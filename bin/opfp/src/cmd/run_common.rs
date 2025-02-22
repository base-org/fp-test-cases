//! Common 'Run' arguments.

use std::path::PathBuf;

use clap::{ArgAction, Parser};

/// CLI arguments for the `run-op-program` subcommand of `opfp`.
#[derive(Parser, Clone, Debug)]
pub struct RunCommon {
    /// Path to the fixture file
    #[clap(short, long, help = "Path to the fixture file")]
    pub fixture: PathBuf,
    /// Optional output file path
    #[clap(long, help = "Path to the output file")]
    pub output: Option<PathBuf>,
    /// Verbosity level (0-4)
    #[arg(long, short, help = "Verbosity level (0-4)", action = ArgAction::Count)]
    pub v: u8,
}
