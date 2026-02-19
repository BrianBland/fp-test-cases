//! Run Op Program Subcommand
//!
//! NOTE: This command is not functional with the current dependency set.
//! It exists only to maintain CLI structure. Use `run-op-succinct` instead.

use clap::{ArgAction, Parser};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use std::path::PathBuf;

/// CLI arguments for the `run-op-program` subcommand of `opfp`.
#[derive(Parser, Clone, Debug)]
pub struct RunOpProgram {
    /// Path to the op-program binary
    #[clap(short, long, help = "Path to the op-program binary")]
    pub op_program: PathBuf,
    /// Path to the fixture file
    #[clap(short, long, help = "Path to the fixture file")]
    pub fixture: PathBuf,
    /// Optional path to the cannon binary
    #[clap(short, long, help = "Path to the cannon binary")]
    pub cannon: Option<PathBuf>,
    /// Optional cannon state
    #[clap(long, help = "Path to the cannon state")]
    pub cannon_state: Option<PathBuf>,
    /// Optional cannon metadata
    #[clap(long, help = "Path to the cannon metadata")]
    pub cannon_meta: Option<PathBuf>,
    /// Optional output file path
    #[clap(long, help = "Path to the output file")]
    pub output: Option<PathBuf>,
    /// Verbosity level (0-4)
    #[arg(long, short, help = "Verbosity level (0-4)", action = ArgAction::Count)]
    pub v: u8,
}

impl RunOpProgram {
    /// Runs the `run-op-program` subcommand.
    ///
    /// This command is deprecated in favor of `run-op-succinct`.
    pub async fn run(&self) -> Result<()> {
        Err(eyre!(
            "run-op-program is not supported with the current dependency set. \
             Use run-op-succinct instead."
        ))
    }
}
