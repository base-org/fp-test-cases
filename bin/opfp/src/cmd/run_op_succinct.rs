//! Run OP Succinct Subcommand

use std::fs::File;

use clap::Parser;
use color_eyre::{eyre::eyre, Result};
use op_succinct_client_utils::boot::BootInfoStruct;
use sp1_sdk::ProverClient;

use crate::cmd::run_op_program::ProgramStats;

use super::run_common::RunCommon;

/// ELF binary for the `range` exe.
pub const RANGE_ELF: &[u8] = include_bytes!("../elf/range-elf");

/// CLI arguments for the `run-op-succinct` subcommand of `opfp`.
#[derive(Parser, Clone, Debug)]
pub struct RunOpSuccinct {
    /// Common arguments.
    #[clap(flatten)]
    pub common: RunCommon,
}

impl RunOpSuccinct {
    /// Runs the `run-op-succinct` subcommand.
    pub async fn run(&self) -> Result<()> {
        let prover = ProverClient::builder().cpu().build();

        let file = File::open(&self.common.fixture)?;
        let stdin = serde_json::from_reader(file)?;
        let start = std::time::Instant::now();

        let (mut public_values, execution_report) = prover
            .execute(RANGE_ELF, &stdin)
            .run()
            .map_err(|err| eyre!("{err}"))?;

        let runtime = start.elapsed().as_millis();

        let _ = public_values.read::<BootInfoStruct>();
        let memory_used = public_values.read::<usize>();

        println!("{execution_report}");

        let stats = ProgramStats {
            runtime,
            instructions: Some(execution_report.total_instruction_count()),
            pages: None,
            memory_used: Some(memory_used as u64),
            num_preimage_requests: None,
            total_preimage_size: None,
        };

        if let Some(output) = &self.common.output {
            let file = std::fs::File::create(output)?;
            serde_json::to_writer_pretty(file, &stats)?;
        }

        Ok(())
    }
}
