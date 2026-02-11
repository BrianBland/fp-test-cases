//! Run OP Succinct Subcommand
//!
//! Executes an SP1Stdin fixture through the SP1 CPU prover and extracts
//! execution statistics including per-precompile cycle counts.

use clap::{ArgAction, Parser};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use num_format::{Locale, ToFormattedString};
use op_succinct_client_utils::precompiles::cycle_tracker::keys;
use op_succinct_proof_utils::get_range_elf_embedded;
use serde::{Deserialize, Serialize};
use sp1_sdk::blocking::{CpuProver, Prover};
use sp1_sdk::{Elf, ExecutionReport, SP1PublicValues, SP1Stdin};
use std::path::PathBuf;
use tracing::{info, trace};

/// The logging target to use for [tracing].
const TARGET: &str = "run-op-succinct";

/// CLI arguments for the `run-op-succinct` subcommand of `opfp`.
#[derive(Parser, Clone, Debug)]
pub struct RunOpSuccinct {
    /// Path to the SP1Stdin fixture file (JSON, produced by from-op-succinct).
    #[clap(short, long)]
    pub fixture: PathBuf,
    /// Output file path for the execution stats (JSON).
    #[clap(long)]
    pub output: PathBuf,
    /// Verbosity level (0-4)
    #[arg(long, short, action = ArgAction::Count)]
    pub v: u8,
}

/// Execution statistics from running an SP1 program through the CPU prover.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpSuccinctStats {
    pub runtime_ms: u128,
    pub total_instruction_count: u64,
    pub oracle_verify_instruction_count: u64,
    pub derivation_instruction_count: u64,
    pub block_execution_instruction_count: u64,
    pub blob_verification_instruction_count: u64,
    pub total_sp1_gas: u64,
    pub bn_pair_cycles: u64,
    pub bn_add_cycles: u64,
    pub bn_mul_cycles: u64,
    pub kzg_eval_cycles: u64,
    pub ec_recover_cycles: u64,
    pub p256_verify_cycles: u64,
}

impl OpSuccinctStats {
    /// Extract stats from an SP1 ExecutionReport.
    pub fn from_report(report: &ExecutionReport, runtime_ms: u128) -> Self {
        let get_cycles = |key: &str| -> u64 { *report.cycle_tracker.get(key).unwrap_or(&0) };

        Self {
            runtime_ms,
            total_instruction_count: report.total_instruction_count(),
            oracle_verify_instruction_count: get_cycles("oracle-verify"),
            derivation_instruction_count: get_cycles("payload-derivation"),
            block_execution_instruction_count: get_cycles("block-execution"),
            blob_verification_instruction_count: get_cycles("blob-verification"),
            total_sp1_gas: report.gas().unwrap_or(0),
            bn_pair_cycles: get_cycles(keys::BN_PAIR),
            bn_add_cycles: get_cycles(keys::BN_ADD),
            bn_mul_cycles: get_cycles(keys::BN_MUL),
            kzg_eval_cycles: get_cycles(keys::KZG_EVAL),
            ec_recover_cycles: get_cycles(keys::EC_RECOVER),
            p256_verify_cycles: get_cycles(keys::P256_VERIFY),
        }
    }
}

impl RunOpSuccinct {
    /// Runs the `run-op-succinct` subcommand.
    ///
    /// 1. Deserializes the SP1Stdin fixture from JSON.
    /// 2. Runs the SP1 CPU prover in a blocking task (it creates its own tokio runtime).
    /// 3. Extracts execution statistics from the report.
    /// 4. Writes the stats to the output file.
    pub async fn run(&self) -> Result<()> {
        trace!(target: TARGET, "Running OP Succinct fixture: {:?}", self.fixture);

        let fixture_json = std::fs::read_to_string(&self.fixture)
            .map_err(|e| eyre!("Failed to read fixture file: {}", e))?;
        let sp1_stdin: SP1Stdin = serde_json::from_str(&fixture_json)
            .map_err(|e| eyre!("Failed to deserialize SP1Stdin: {}", e))?;

        // CpuProver::new() creates its own tokio runtime internally,
        // so we must run it in spawn_blocking to avoid nested runtime panic.
        info!(target: TARGET, "Executing through SP1 CPU prover...");
        let start = std::time::Instant::now();

        let result: std::result::Result<(SP1PublicValues, ExecutionReport), _> =
            tokio::task::spawn_blocking(move || {
                let prover = CpuProver::new();
                prover
                    .execute(Elf::Static(get_range_elf_embedded()), sp1_stdin)
                    .calculate_gas(true)
                    .deferred_proof_verification(false)
                    .run()
            })
            .await
            .map_err(|e| eyre!("spawn_blocking join error: {}", e))?;
        let (_, report) = result.map_err(|e| eyre!("SP1 execution failed: {}", e))?;

        let runtime_ms = start.elapsed().as_millis();
        let stats = OpSuccinctStats::from_report(&report, runtime_ms);

        info!(target: TARGET, "Execution completed in {}ms", runtime_ms);
        info!(target: TARGET, "Total instructions: {}",
            stats.total_instruction_count.to_formatted_string(&Locale::en));
        info!(target: TARGET, "Total SP1 gas: {}",
            stats.total_sp1_gas.to_formatted_string(&Locale::en));
        info!(target: TARGET, "BN pair cycles: {}",
            stats.bn_pair_cycles.to_formatted_string(&Locale::en));
        info!(target: TARGET, "BN add cycles: {}",
            stats.bn_add_cycles.to_formatted_string(&Locale::en));
        info!(target: TARGET, "BN mul cycles: {}",
            stats.bn_mul_cycles.to_formatted_string(&Locale::en));
        info!(target: TARGET, "KZG eval cycles: {}",
            stats.kzg_eval_cycles.to_formatted_string(&Locale::en));
        info!(target: TARGET, "EC recover cycles: {}",
            stats.ec_recover_cycles.to_formatted_string(&Locale::en));
        info!(target: TARGET, "P256 verify cycles: {}",
            stats.p256_verify_cycles.to_formatted_string(&Locale::en));

        if let Some(parent) = self.output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(&self.output)?;
        serde_json::to_writer_pretty(file, &stats)?;
        info!(target: TARGET, "Wrote execution stats to {:?}", self.output);

        Ok(())
    }
}
