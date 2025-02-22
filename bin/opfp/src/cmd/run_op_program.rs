//! Run Op Program Subcommand

use alloy_primitives::hex::ToHexExt;
use alloy_primitives::U64;
use clap::Parser;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use fp_test_fixtures::{ChainDefinition, FaultProofFixture};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, path::PathBuf};
use tracing::info;

use super::run_common::RunCommon;
use super::util::{RollupConfig, VersionedState};

/// The logging target to use for [tracing].
const TARGET: &str = "run-op-program";

/// CLI arguments for the `run-op-program` subcommand of `opfp`.
#[derive(Parser, Clone, Debug)]
pub struct RunOpProgram {
    /// Path to the op-program binary
    #[clap(short, long, help = "Path to the op-program binary")]
    pub op_program: PathBuf,
    /// Optional path to the cannon binary
    #[clap(short, long, help = "Path to the cannon binary")]
    pub cannon: Option<PathBuf>,
    /// Optional cannon state
    #[clap(long, help = "Path to the cannon state")]
    pub cannon_state: Option<PathBuf>,
    /// Optional cannon metadata
    #[clap(long, help = "Path to the cannon metadata")]
    pub cannon_meta: Option<PathBuf>,
    /// Common arguments.
    #[clap(flatten)]
    pub common: RunCommon,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProgramStats {
    pub runtime: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_used: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_preimage_requests: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_preimage_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CannonOutput {
    pub step: u64,
}

#[derive(Debug, Deserialize)]
struct CannonDebug {
    pub pages: u64,
    pub memory_used: U64,
    pub num_preimage_requests: u64,
    pub total_preimage_size: u64,
}

impl RunOpProgram {
    /// Runs the `run-op-program` subcommand.
    pub async fn run(&self) -> Result<()> {
        let fixture = std::fs::read_to_string(&self.common.fixture)
            .map_err(|e| eyre!("Failed to read fixture file: {}", e))?;
        let fixture: FaultProofFixture = serde_json::from_str(&fixture)
            .map_err(|e| eyre!("Failed to parse fixture file: {}", e))?;

        let dirname = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis()
            .to_string();
        let data_dir = env::temp_dir().join("run-op-program").join(dirname);
        if data_dir.exists() {
            std::fs::remove_dir_all(&data_dir)?;
        }
        std::fs::create_dir_all(&data_dir)?;

        let op_program_command =
            OpProgramCommand::new(self.op_program.clone(), fixture, data_dir.clone());

        match self.cannon.as_ref() {
            Some(cannon) => {
                let cannon_command = CannonCommand::new(
                    cannon.clone(),
                    self.cannon_state
                        .clone()
                        .ok_or(eyre!("Missing cannon state"))?,
                    self.cannon_meta
                        .clone()
                        .ok_or(eyre!("Missing cannon meta"))?,
                    op_program_command,
                );
                cannon_command.prepare().await?;
                let stats = cannon_command.run().await?;
                info!(target: TARGET, "Cannon stats: {:?}", stats);

                if let Some(output) = &self.common.output {
                    let file = std::fs::File::create(output)?;
                    serde_json::to_writer_pretty(file, &stats)?;
                }
            }
            None => {
                op_program_command.prepare().await?;
                let stats = op_program_command.run().await?;
                info!(target: TARGET, "op-program stats: {:?}", stats);

                if let Some(output) = &self.common.output {
                    let file = std::fs::File::create(output)?;
                    serde_json::to_writer_pretty(file, &stats)?;
                }
            }
        }

        std::fs::remove_dir_all(&data_dir)?;

        Ok(())
    }
}

/// The command to run the op-program within cannon.
#[derive(Debug)]
pub struct CannonCommand {
    /// The path to the cannon binary.
    pub cannon: PathBuf,
    /// The path to the cannon state file.
    pub state: PathBuf,
    /// The path to the cannon metadata file.
    pub meta: PathBuf,
    /// The path to the cannon output file.
    pub output: PathBuf,
    /// The path to the cannon debug output file.
    pub debug: PathBuf,
    /// The op-program command to run within cannon.
    pub op_program: OpProgramCommand,
}

impl CannonCommand {
    pub fn new(
        cannon: PathBuf,
        state: PathBuf,
        meta: PathBuf,
        op_program: OpProgramCommand,
    ) -> Self {
        let output = op_program.data_dir.join("cannon-output.bin");
        let debug = op_program.data_dir.join("cannon-debug.json");

        Self {
            cannon,
            state,
            meta,
            output,
            debug,
            op_program,
        }
    }

    pub async fn prepare(&self) -> Result<()> {
        self.op_program.prepare().await?;

        Ok(())
    }

    pub async fn run(&self) -> Result<ProgramStats> {
        let start = std::time::Instant::now();

        let result = Command::new(&self.cannon).args(self.args()).status();

        if result.is_err() {
            return Err(eyre!("Failed to execute cannon binary"));
        }

        let runtime = start.elapsed().as_millis();

        let data =
            std::fs::read(&self.output).map_err(|e| eyre!("Failed to read output file: {}", e))?;

        let versioned_state = VersionedState::try_from(data)
            .map_err(|e| eyre!("Failed to decode versioned state: {}", e))?;
        let output: CannonOutput = CannonOutput {
            step: versioned_state.single_threaded_fpvmstate.step,
        };

        let debug_output = std::fs::read_to_string(&self.debug)
            .map_err(|e| eyre!("Failed to read debug output file: {}", e))?;
        let debug_output: CannonDebug = serde_json::from_str(&debug_output)?;

        let stats = ProgramStats {
            runtime,
            instructions: Some(output.step),
            pages: Some(debug_output.pages),
            memory_used: Some(debug_output.memory_used.to()),
            num_preimage_requests: Some(debug_output.num_preimage_requests),
            total_preimage_size: Some(debug_output.total_preimage_size),
        };

        Ok(stats)
    }

    pub fn args(&self) -> Vec<String> {
        let mut args = vec![
            "run".to_string(),
            "--info-at".to_string(),
            "%10000000".to_string(),
            "--input".to_string(),
            self.state.to_str().unwrap().to_string(),
            "--meta".to_string(),
            self.meta.to_str().unwrap().to_string(),
            "--output".to_string(),
            self.output.to_str().unwrap().to_string(),
            "--debug-info".to_string(),
            self.debug.to_str().unwrap().to_string(),
            "--".to_string(),
            self.op_program.op_program.to_str().unwrap().to_string(),
        ];
        args.extend(self.op_program.args());
        args.push("--server".to_string());
        args
    }
}

/// The command to run the op-program.
#[derive(Debug)]
pub struct OpProgramCommand {
    /// The path to the op-program binary.
    pub op_program: PathBuf,
    /// The fixture to run the op-program with.
    pub fixture: FaultProofFixture,
    /// The directory to store the input data for the op-program.
    pub data_dir: PathBuf,
}

impl OpProgramCommand {
    pub fn new(op_program: PathBuf, fixture: FaultProofFixture, data_dir: PathBuf) -> Self {
        Self {
            op_program,
            fixture,
            data_dir,
        }
    }

    pub async fn prepare(&self) -> Result<()> {
        if let ChainDefinition::Unnamed(rollup_config, genesis) =
            &self.fixture.inputs.chain_definition
        {
            // Write the genesis file to the temp directory.
            let genesis_file = self.data_dir.join("genesis.json");
            let file = std::fs::File::create(&genesis_file)?;
            serde_json::to_writer_pretty(file, &genesis)?;

            // Write the rollup config to the temp directory.
            let rollup_config_file = self.data_dir.join("rollup_config.json");
            let file = std::fs::File::create(&rollup_config_file)?;
            let cfg: RollupConfig = rollup_config.into();
            serde_json::to_writer_pretty(file, &cfg)?;
        }

        for (key, value) in &self.fixture.witness_data {
            let key_hex = key.encode_hex();

            let (dirname, filename) = key_hex.split_at(4);
            let dirname = self.data_dir.join(dirname);
            std::fs::create_dir_all(&dirname)?;

            let file = dirname.join(format!("{}.txt", filename));
            std::fs::write(file, value.encode_hex())?;
        }

        Ok(())
    }

    pub async fn run(&self) -> Result<ProgramStats> {
        let start = std::time::Instant::now();

        let result = Command::new(&self.op_program).args(self.args()).status();

        if result.is_err() {
            return Err(eyre!("Failed to execute op-program binary"));
        }

        let runtime = start.elapsed().as_millis();

        Ok(ProgramStats {
            runtime,
            ..ProgramStats::default()
        })
    }

    pub fn args(&self) -> Vec<String> {
        let mut args = vec![
            "--l1.head".to_string(),
            self.fixture.inputs.l1_head.to_string(),
            "--l2.head".to_string(),
            self.fixture.inputs.l2_head.to_string(),
            "--l2.outputroot".to_string(),
            self.fixture.inputs.l2_output_root.encode_hex_with_prefix(),
            "--l2.blocknumber".to_string(),
            self.fixture.inputs.l2_block_number.to_string(),
            "--l2.claim".to_string(),
            self.fixture.inputs.l2_claim.encode_hex_with_prefix(),
            "--log.format".to_string(),
            "terminal".to_string(),
            "--datadir".to_string(),
            self.data_dir.to_str().unwrap().to_string(),
            "--data.format".to_string(),
            "directory".to_string(),
            "--l2.custom".to_string(),
        ];
        match &self.fixture.inputs.chain_definition {
            ChainDefinition::Named(name) => {
                args.push("--network".to_string());
                args.push(name.to_string());
            }
            ChainDefinition::Unnamed(_, _) => {
                let data_dir = self.data_dir.clone();
                args.push("--l2.genesis".to_string());
                args.push(data_dir.join("genesis.json").to_str().unwrap().to_string());
                args.push("--rollup.config".to_string());
                args.push(
                    data_dir
                        .join("rollup_config.json")
                        .to_str()
                        .unwrap()
                        .to_string(),
                );
            }
        }
        args
    }
}
