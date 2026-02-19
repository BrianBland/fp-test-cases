use anyhow::Result;
use clap::Parser;
use num_format::{Locale, ToFormattedString};
use op_succinct_client_utils::precompiles::cycle_tracker::keys;
use op_succinct_host_utils::{
    fetcher::OPSuccinctDataFetcher, host::OPSuccinctHost,
    witness_generation::WitnessGenerator,
};
use op_succinct_proof_utils::{get_range_elf_embedded, initialize_host};
use serde::{Deserialize, Serialize};
use sp1_sdk::{ProverClient, SP1Stdin};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "v5-baseline", about = "Run OP Succinct ecpairing test through SP1 v5")]
struct Args {
    /// L2 start block (exclusive — the range is (start, end])
    #[arg(long)]
    l2_start_block: u64,

    /// L2 end block (inclusive)
    #[arg(long)]
    l2_end_block: u64,

    /// Output JSON file path
    #[arg(long)]
    output: PathBuf,

    /// Path to a pre-serialized SP1Stdin fixture. If provided, skips witness generation.
    #[arg(long)]
    fixture: Option<PathBuf>,
}

/// Stats output format, compatible with jackson's OpSuccinctStats.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpSuccinctStats {
    runtime_ms: u128,
    total_instruction_count: u64,
    oracle_verify_instruction_count: u64,
    derivation_instruction_count: u64,
    block_execution_instruction_count: u64,
    blob_verification_instruction_count: u64,
    total_sp1_gas: u64,
    bn_pair_cycles: u64,
    bn_add_cycles: u64,
    bn_mul_cycles: u64,
    kzg_eval_cycles: u64,
    ec_recover_cycles: u64,
    p256_verify_cycles: u64,
    syscall_counts: BTreeMap<String, u64>,
    opcode_counts: BTreeMap<String, u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let sp1_stdin = if let Some(fixture_path) = &args.fixture {
        info!("Loading SP1Stdin fixture from {:?}", fixture_path);
        let json = std::fs::read_to_string(fixture_path)?;
        serde_json::from_str::<SP1Stdin>(&json)?
    } else {
        info!(
            "Generating witness for blocks ({}, {}]",
            args.l2_start_block, args.l2_end_block
        );
        generate_witness(args.l2_start_block, args.l2_end_block).await?
    };

    info!("Executing through SP1 v5 prover...");
    let start = std::time::Instant::now();

    let prover = ProverClient::builder().mock().build();
    let (_, report) = prover
        .execute(get_range_elf_embedded(), &sp1_stdin)
        .calculate_gas(true)
        .deferred_proof_verification(false)
        .run()?;

    let runtime_ms = start.elapsed().as_millis();

    let get_cycles = |key: &str| -> u64 { *report.cycle_tracker.get(key).unwrap_or(&0) };

    let syscall_counts: BTreeMap<String, u64> = report
        .syscall_counts
        .iter()
        .filter(|(_, &count)| count > 0)
        .map(|(code, &count)| (format!("{code}"), count))
        .collect();

    let opcode_counts: BTreeMap<String, u64> = report
        .opcode_counts
        .iter()
        .filter(|(_, &count)| count > 0)
        .map(|(op, &count)| (format!("{op}"), count))
        .collect();

    let stats = OpSuccinctStats {
        runtime_ms,
        total_instruction_count: report.total_instruction_count(),
        oracle_verify_instruction_count: get_cycles("oracle-verify"),
        derivation_instruction_count: get_cycles("payload-derivation"),
        block_execution_instruction_count: get_cycles("block-execution"),
        blob_verification_instruction_count: get_cycles("blob-verification"),
        total_sp1_gas: report.gas.unwrap_or(0),
        bn_pair_cycles: get_cycles(keys::BN_PAIR),
        bn_add_cycles: get_cycles(keys::BN_ADD),
        bn_mul_cycles: get_cycles(keys::BN_MUL),
        kzg_eval_cycles: get_cycles(keys::KZG_EVAL),
        ec_recover_cycles: get_cycles(keys::EC_RECOVER),
        p256_verify_cycles: get_cycles(keys::P256_VERIFY),
        syscall_counts,
        opcode_counts,
    };

    info!("Execution completed in {}ms", runtime_ms);
    info!(
        "Total instructions: {}",
        stats.total_instruction_count.to_formatted_string(&Locale::en)
    );
    info!(
        "Total SP1 gas: {}",
        stats.total_sp1_gas.to_formatted_string(&Locale::en)
    );
    info!(
        "BN pair cycles: {}",
        stats.bn_pair_cycles.to_formatted_string(&Locale::en)
    );
    info!(
        "BN add cycles: {}",
        stats.bn_add_cycles.to_formatted_string(&Locale::en)
    );
    info!(
        "BN mul cycles: {}",
        stats.bn_mul_cycles.to_formatted_string(&Locale::en)
    );
    info!(
        "Total syscalls: {}",
        report.total_syscall_count().to_formatted_string(&Locale::en)
    );

    if let Some(parent) = args.output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(&args.output)?;
    serde_json::to_writer_pretty(file, &stats)?;
    info!("Wrote execution stats to {:?}", args.output);

    Ok(())
}

/// Generate SP1Stdin witness using lincoln's host infrastructure.
/// Requires environment variables: L1_RPC, L2_RPC, L1_BEACON_RPC, L2_NODE_RPC
async fn generate_witness(l2_start_block: u64, l2_end_block: u64) -> Result<SP1Stdin> {
    let data_fetcher = OPSuccinctDataFetcher::new_with_rollup_config().await?;
    let host = initialize_host(Arc::new(data_fetcher));

    let host_args = host
        .fetch(l2_start_block, l2_end_block, None, false)
        .await?;

    info!("Running witness generation...");
    let witness = host.run(&host_args).await?;

    let stdin = host.witness_generator().get_sp1_stdin(witness)?;
    info!("Witness generation complete");

    Ok(stdin)
}
