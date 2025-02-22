//! Common 'From' arguments.

use std::path::PathBuf;

use clap::{ArgAction, Parser};

/// CLI arguments for the `from-op-program` and `fro;-op-succinct` subcommands of `opfp`.
#[derive(Parser, Clone, Debug)]
pub struct FromComon {
    /// The output file for the test fixture.
    #[clap(long, help = "Output file for the test fixture")]
    pub output: PathBuf,
    /// Verbosity level (0-4)
    #[arg(long, short, help = "Verbosity level (0-4)", action = ArgAction::Count)]
    pub v: u8,
}
