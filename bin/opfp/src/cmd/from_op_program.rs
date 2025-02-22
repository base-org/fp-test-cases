//! From Op Program Subcommand

use alloy_primitives::hex::FromHex;
use alloy_primitives::BlockHash;
use alloy_primitives::{hex::ToHexExt, B256};
use clap::Parser;
use color_eyre::{eyre::eyre, Result};
use fp_test_fixtures::{
    self, ChainDefinition, FaultProofFixture, FaultProofInputs, FaultProofStatus, Genesis,
};
use kona_derive::traits::ChainProvider;
use kona_providers_alloy::{AlloyChainProvider, AlloyL2ChainProvider};
use maili_protocol::BatchValidationProvider;
use maili_registry::ROLLUP_CONFIGS;
use reqwest::Url;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    env,
    io::{stderr, stdout},
    path::PathBuf,
};
use tracing::{debug, error, info, trace};

use crate::cmd::util::RollupConfig;

use super::from_common::FromComon;
use super::util::{RollupProvider, SafeHeadResponse};

/// The logging target to use for [tracing].
const TARGET: &str = "from-op-program";

/// CLI arguments for the `from-op-program` subcommand of `opfp`.
#[derive(Parser, Clone, Debug)]
pub struct FromOpProgram {
    /// The path to the op-program binary.
    #[clap(short, long, help = "Path to the op-program binary")]
    pub op_program: PathBuf,
    /// The L2 block number to validate.
    #[clap(long, help = "L2 block number to validate")]
    pub l2_block: u64,
    /// Optional L1 block number which can derive the given L2 block.
    #[clap(
        long,
        help = "Optional L1 block number which can derive the given L2 block"
    )]
    pub l1_block: Option<u64>,
    /// An RPC URL to fetch L1 block data from.
    #[clap(long, help = "RPC url to fetch L1 block data from")]
    pub l1_rpc_url: String,
    /// An L2 RPC URL to validate span batches.
    #[clap(long, help = "L2 RPC URL to validate span batches")]
    pub l2_rpc_url: String,
    /// A beacon client to fetch blob data from.
    #[clap(long, help = "Beacon client url to fetch blob data from")]
    pub beacon_url: String,
    /// A rollup client to fetch derivation data from.
    #[clap(long, help = "Rollup client url to fetch derivation data from")]
    pub rollup_url: String,
    /// Optional chain name.
    #[clap(long, help = "Optional chain name")]
    pub chain_name: Option<String>,
    /// Optional path to the rollup config file.
    #[clap(long, help = "Optional path to the rollup config file")]
    pub rollup_path: Option<PathBuf>,
    /// Optional path to the genesis file.
    #[clap(long, help = "Optional path to the genesis file")]
    pub genesis_path: Option<PathBuf>,
    /// Common arguments.
    #[clap(flatten)]
    pub common: FromComon,
}

impl FromOpProgram {
    /// Runs the from-op-program subcommand.
    pub async fn run(&self) -> Result<()> {
        trace!(target: TARGET, "Producing fault proof fixture for L2 block {}", self.l2_block);

        let inputs = self.fault_proof_inputs().await?;
        debug!(target: TARGET, "Using the following fault proof inputs: {:?}", inputs);

        let dirname = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis()
            .to_string();
        let data_dir = env::temp_dir().join("from-op-program").join(dirname);
        std::fs::create_dir_all(&data_dir)?;

        let input_dir = data_dir.join("input");
        if input_dir.exists() {
            std::fs::remove_dir_all(&input_dir)?;
            std::fs::create_dir(&input_dir)?;
        } else {
            std::fs::create_dir(&input_dir)?;
        }
        info!(target: TARGET, "Created input temp directory: {:?}", input_dir);

        let output_dir = data_dir.join("output");
        if output_dir.exists() {
            std::fs::remove_dir_all(&output_dir)?;
        }
        info!(target: TARGET, "Created output temp directory: {:?}", output_dir);

        let mut command = std::process::Command::new(&self.op_program);
        match &inputs.chain_definition {
            ChainDefinition::Named(name) => {
                command.arg("--network").arg(name);
            }
            ChainDefinition::Unnamed(rollup_config, genesis) => {
                // Copy the genesis file to the temp directory.
                let genesis_file = input_dir.join("genesis.json");
                let file = std::fs::File::create(&genesis_file)?;
                serde_json::to_writer_pretty(file, &genesis)?;

                // Write the rollup config to the temp directory.
                let rollup_config_file = input_dir.join("rollup_config.json");
                let file = std::fs::File::create(&rollup_config_file)?;
                let cfg: RollupConfig = rollup_config.into();
                serde_json::to_writer_pretty(file, &cfg)?;

                command
                    .arg("--l2.genesis")
                    .arg(
                        genesis_file
                            .to_str()
                            .ok_or(eyre!("Failed to convert genesis file path to string"))?,
                    )
                    .arg("--rollup.config")
                    .arg(
                        rollup_config_file
                            .to_str()
                            .ok_or(eyre!("Failed to convert rollup config file path to string"))?,
                    );
            }
        }
        // Execute the op-program binary.
        let status = command
            .arg("--l1")
            .arg(self.l1_rpc_url.clone())
            .arg("--l2")
            .arg(self.l2_rpc_url.clone())
            .arg("--l1.beacon")
            .arg(self.beacon_url.clone())
            .arg("--l1.head")
            .arg(inputs.l1_head.encode_hex_with_prefix())
            .arg("--l2.head")
            .arg(inputs.l2_head.to_string())
            .arg("--l2.outputroot")
            .arg(inputs.l2_output_root.encode_hex_with_prefix())
            .arg("--l2.blocknumber")
            .arg(inputs.l2_block_number.to_string())
            .arg("--l2.claim")
            .arg(inputs.l2_claim.encode_hex_with_prefix())
            .arg("--log.format")
            .arg("terminal")
            .arg("--l2.custom")
            .arg("--datadir")
            .arg(
                output_dir
                    .to_str()
                    .ok_or(eyre!("Failed to convert output directory path to string"))?,
            )
            .arg("--data.format")
            .arg("directory")
            .stdout(stdout())
            .stderr(stderr())
            .status()
            .map_err(|e| eyre!(e))?;

        if !status.success() {
            error!(target: TARGET, "Failed to execute op-program binary");
            return Err(eyre!("Failed to execute op-program binary"));
        }

        let mut witness_data = BTreeMap::new();

        // Parse the output of the op-program binary and populate the witness data.
        output_dir.read_dir()?.try_for_each(|entry| -> Result<()> {
            let entry = entry?;
            let prefix = entry.path();
            debug!(target: TARGET, "Found dir: {:?}", prefix);
            if !prefix.is_dir() {
                return Ok(());
            }
            prefix.read_dir()?.try_for_each(|entry| -> Result<()> {
                let entry = entry?;
                let filename = entry.path();
                debug!(target: TARGET, "Found file: {:?}", filename);

                let contents = std::fs::read_to_string(&filename)?;
                debug!(target: TARGET, "File contents: {}", contents);

                let key_prefix = prefix
                    .file_name()
                    .ok_or(eyre!("Failed to get directory name"))?
                    .to_os_string()
                    .into_string()
                    .map_err(|_| eyre!("Failed to convert directory name into string"))?;

                // strip the .txt suffix from the file path
                let key: String = key_prefix
                    + filename
                        .file_name()
                        .ok_or(eyre!("Failed to get file name"))?
                        .to_str()
                        .ok_or(eyre!("Failed to convert file name to string"))?
                        .split('.')
                        .next()
                        .ok_or(eyre!("Failed to strip file extension"))?;

                debug!(target: TARGET, "Key: {}", key);

                let key: B256 = FromHex::from_hex(key)?;
                let witness = FromHex::from_hex(contents)?;

                witness_data.insert(key, witness);
                Ok(())
            })
        })?;

        let fixture = FaultProofFixture {
            inputs,
            expected_status: FaultProofStatus::Valid,
            witness_data,
        };
        info!(target: TARGET, "Successfully built fault proof test fixture");

        // Write the fault proof fixture to the specified output location.
        let file = std::fs::File::create(&self.common.output)?;
        serde_json::to_writer_pretty(file, &fixture)?;
        info!(target: TARGET, "Wrote fault proof fixture to: {:?}", self.common.output);

        Ok(())
    }

    /// Returns a new [AlloyChainProvider] using the l1 rpc url.
    pub fn l1_provider(&self) -> Result<AlloyChainProvider> {
        Ok(AlloyChainProvider::new_http(self.l1_rpc_url()?))
    }

    /// Returns a new [AlloyL2ChainProvider] using the l2 rpc url.
    pub fn l2_provider(
        &self,
        cfg: Arc<maili_genesis::RollupConfig>,
    ) -> Result<AlloyL2ChainProvider> {
        Ok(AlloyL2ChainProvider::new_http(self.l2_rpc_url()?, cfg))
    }

    /// Returns a new [RollupProvider] using the rollup rpc url.
    pub fn rollup_provider(&self) -> Result<RollupProvider> {
        Ok(RollupProvider::new_http(self.rollup_url()?))
    }

    /// Gets the rollup config from the l2 rpc url.
    pub async fn rollup_config(&self) -> Result<super::util::RollupConfig> {
        if let Some(path) = &self.rollup_path {
            let file = std::fs::File::open(path)?;
            let cfg: super::util::RollupConfig = serde_json::from_reader(file)?;
            return Ok(cfg);
        }

        let mut l2_provider =
            AlloyL2ChainProvider::new_http(self.l2_rpc_url()?, Arc::new(Default::default()));
        let l2_chain_id = l2_provider.chain_id().await.map_err(|e| eyre!(e))?;
        let cfg = ROLLUP_CONFIGS
            .get(&l2_chain_id)
            .ok_or_else(|| eyre!("No rollup config found for L2 chain ID: {}", l2_chain_id))?;

        Ok(cfg.into())
    }

    /// Returns the l1 rpc url from CLI or environment variable.
    pub fn l1_rpc_url(&self) -> Result<Url> {
        Url::parse(&self.l1_rpc_url).map_err(|e| eyre!(e))
    }

    /// Returns the l2 rpc url from CLI or environment variable.
    pub fn l2_rpc_url(&self) -> Result<Url> {
        Url::parse(&self.l2_rpc_url).map_err(|e| eyre!(e))
    }

    /// Returns the rollup rpc url from CLI or environment variable.
    pub fn rollup_url(&self) -> Result<Url> {
        Url::parse(&self.rollup_url).map_err(|e| eyre!(e))
    }

    /// Returns the beacon url from CLI or environment variable.
    pub fn beacon_url(&self) -> String {
        self.beacon_url.clone()
    }

    async fn fault_proof_inputs(&self) -> Result<FaultProofInputs> {
        let cfg = self.rollup_config().await?;

        let rollup_provider = self.rollup_provider()?;

        let claim_output = rollup_provider.output_at_block(self.l2_block).await?;
        let parent_output = rollup_provider.output_at_block(self.l2_block - 1).await?;

        let chain_definition: ChainDefinition;

        if let Some(genesis_path) = &self.genesis_path {
            let genesis_file = std::fs::File::open(genesis_path)?;
            let genesis: Genesis = serde_json::from_reader(genesis_file)?;
            chain_definition = ChainDefinition::Unnamed(cfg.into(), genesis);
        } else {
            chain_definition = ChainDefinition::Named(
                self.chain_name
                    .clone()
                    .ok_or_else(|| eyre!("Missing chain name"))?,
            );
        }

        let l1_head: BlockHash;

        if let Some(l1_block) = self.l1_block {
            l1_head = self
                .l1_provider()?
                .block_info_by_number(l1_block)
                .await
                .map_err(|_| eyre!("Failed to fetch L1 block info"))?
                .hash;
        } else {
            let next_safe_head = self.find_next_safe_head().await?;
            l1_head = next_safe_head.l1_block.hash;
        }

        Ok(FaultProofInputs {
            l1_head,
            l2_head: parent_output.block_ref.hash,
            l2_output_root: parent_output.output_root,
            l2_block_number: claim_output.block_ref.number,
            l2_claim: claim_output.output_root,
            chain_definition,
        })
    }

    async fn find_next_safe_head(&self) -> Result<SafeHeadResponse> {
        let cfg = self.rollup_config().await?;
        let mut l2_provider = self.l2_provider(Arc::new(cfg.into()))?;

        let l2_block_info = l2_provider
            .l2_block_info_by_number(self.l2_block)
            .await
            .map_err(|_| eyre!("Failed to fetch L2 block info"))?;
        let mut l1_block_num = l2_block_info.l1_origin.number;

        let rollup_provider = self.rollup_provider()?;

        let skip_size = 32;
        for _ in 0..10 {
            l1_block_num += skip_size;
            let next_safe_head = rollup_provider.safe_head_at_block(l1_block_num).await?;
            if next_safe_head.safe_head.number >= self.l2_block {
                return Ok(next_safe_head);
            }
        }
        Err(eyre!("No next safe head found"))
    }
}
