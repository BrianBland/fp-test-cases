//! From OP Succinct Subcommand
//!
//! Generates an SP1Stdin fixture by fetching witness data through the
//! OP Succinct host pipeline (ETH-DA / SingleChainOPSuccinctHost).

use alloy_primitives::B256;
use clap::{ArgAction, Parser};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use op_succinct_ethereum_host_utils::host::SingleChainOPSuccinctHost;
use op_succinct_host_utils::fetcher::OPSuccinctDataFetcher;
use op_succinct_host_utils::host::OPSuccinctHost;
use op_succinct_host_utils::witness_generation::WitnessGenerator;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, trace};

const TARGET: &str = "from-op-succinct";

/// CLI arguments for the `from-op-succinct` subcommand of `opfp`.
///
/// Requires the following environment variables:
///   - `L1_RPC`        — L1 execution RPC endpoint
///   - `L1_BEACON_RPC` — L1 beacon (consensus) RPC endpoint
///   - `L2_RPC`        — L2 execution RPC endpoint
///   - `L2_NODE_RPC`   — L2 rollup node RPC endpoint
#[derive(Parser, Clone, Debug)]
pub struct FromOpSuccinct {
    /// L2 start block (inclusive). Typically `end_block - 1`.
    #[clap(long)]
    pub l2_start_block: u64,
    /// L2 end block (inclusive) — the block whose execution we benchmark.
    #[clap(long)]
    pub l2_end_block: u64,
    /// SP1Stdin fixture output path (JSON).
    #[clap(long)]
    pub output: PathBuf,
    /// L1 head block hash. Bypasses automatic L1 head estimation (which
    /// requires SafeDB or caps at finalized L1). Useful for devnets where
    /// L1 finality lags behind batch posting.
    #[clap(long)]
    pub l1_head: Option<B256>,
    /// Verbosity level (0-4)
    #[arg(long, short, action = ArgAction::Count)]
    pub v: u8,
}

impl FromOpSuccinct {
    /// Fetch witness data via the OP Succinct host pipeline and write an SP1Stdin fixture.
    pub async fn run(&self) -> Result<()> {
        trace!(target: TARGET, "Generating OP Succinct fixture for L2 blocks {} -> {}",
            self.l2_start_block, self.l2_end_block);

        info!(target: TARGET, "Initializing OPSuccinctDataFetcher from environment...");
        let fetcher = OPSuccinctDataFetcher::new_with_rollup_config()
            .await
            .map_err(|e| eyre!("Failed to create OPSuccinctDataFetcher: {}", e))?;

        let host = SingleChainOPSuccinctHost::new(Arc::new(fetcher));

        info!(target: TARGET, "Fetching host args for blocks {} -> {}...",
            self.l2_start_block, self.l2_end_block);
        let host_args = host
            .fetch(self.l2_start_block, self.l2_end_block, self.l1_head, true)
            .await
            .map_err(|e| eyre!("Failed to fetch host args: {}", e))?;

        info!(target: TARGET, "Running host to generate witness...");
        let witness = host
            .run(&host_args)
            .await
            .map_err(|e| eyre!("Failed to run host: {}", e))?;

        let sp1_stdin = host
            .witness_generator()
            .get_sp1_stdin(witness)
            .map_err(|e| eyre!("Failed to create SP1Stdin: {}", e))?;

        if let Some(parent) = self.output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(&sp1_stdin)
            .map_err(|e| eyre!("Failed to serialize SP1Stdin: {}", e))?;
        std::fs::write(&self.output, json)?;

        info!(target: TARGET, "Wrote SP1Stdin fixture to {:?}", self.output);
        Ok(())
    }
}
