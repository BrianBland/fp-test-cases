//! From Op Program Subcommand
//!
//! NOTE: This command is not functional with the new kona/alloy dependencies.
//! It exists only to maintain CLI structure. Use `from-op-succinct` instead.

use clap::{ArgAction, Parser};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use std::path::PathBuf;

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
    /// The output file for the test fixture.
    #[clap(long, help = "Output file for the test fixture")]
    pub output: PathBuf,
    /// Verbosity level (0-4)
    #[arg(long, short, help = "Verbosity level (0-4)", action = ArgAction::Count)]
    pub v: u8,
}

impl FromOpProgram {
    /// Runs the from-op-program subcommand.
    ///
    /// This command is deprecated in favor of `from-op-succinct`.
    pub async fn run(&self) -> Result<()> {
        Err(eyre!(
            "from-op-program is not supported with the current dependency set. \
             Use from-op-succinct instead."
        ))
    }
}
