//! From OP Succinct Subcommand

use std::fs::File;

use clap::Parser;
use color_eyre::{eyre::eyre, Result};
use op_succinct_host_utils::{
    fetcher::{CacheMode, OPSuccinctDataFetcher, RunContext},
    get_proof_stdin, start_server_and_native_client, ProgramType,
};
use tracing::info;

use super::from_common::FromComon;

/// CLI arguments for the `from-op-succinct` subcommand of `opfp`.
#[derive(Parser, Clone, Debug)]
pub struct FromOpSuccinct {
    /// The start L2 block number to validate.
    #[clap(long, help = "L2 block number to validate")]
    pub l2_start_block: u64,
    /// The end L2 block number to validate.
    #[clap(long, help = "L2 block number to validate")]
    pub l2_end_block: u64,
    /// Common arguments.
    #[clap(flatten)]
    pub common: FromComon,
}

impl FromOpSuccinct {
    /// Runs the from-op-succinct subcommand.
    pub async fn run(&self) -> Result<()> {
        let data_fetcher = OPSuccinctDataFetcher::new_with_rollup_config(RunContext::Dev)
            .await
            .map_err(|err| eyre!(Box::new(err)))?;

        let host_args = data_fetcher
            .get_host_args(
                self.l2_start_block,
                self.l2_end_block,
                ProgramType::Multi,
                CacheMode::KeepCache,
            )
            .await
            .map_err(|err| eyre!("{err}"))?;

        let oracle = start_server_and_native_client(host_args)
            .await
            .map_err(|err| eyre!("{err}"))?;
        let stdin = get_proof_stdin(oracle).map_err(|err| eyre!("{err}"))?;

        let mut file = File::create(&self.common.output)?;
        info!("Persist fixture to {:?}", self.common.output);
        serde_json::to_writer(&mut file, &stdin)?;

        Ok(())
    }
}
