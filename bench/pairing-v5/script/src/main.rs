use num_format::{Locale, ToFormattedString};
use serde::{Deserialize, Serialize};
use sp1_sdk::{ProverClient, SP1Stdin};
use std::collections::BTreeMap;

const GUEST_ELF: &[u8] = sp1_build::include_elf!("pairing-guest-v5");

#[derive(Debug, Serialize, Deserialize)]
struct PairingStats {
    num_pairings: u32,
    total_instruction_count: u64,
    per_pairing_instruction_count: u64,
    total_syscall_count: u64,
    total_sp1_gas: u64,
    runtime_ms: u128,
    cycle_tracker: BTreeMap<String, u64>,
    syscall_counts: BTreeMap<String, u64>,
    opcode_counts: BTreeMap<String, u64>,
}

fn main() {
    let n: u32 = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()
        .expect("first argument must be a positive integer (number of pairings)");

    let mut stdin = SP1Stdin::new();
    stdin.write(&n);

    let prover = ProverClient::builder().mock().build();
    let start = std::time::Instant::now();
    let (_, report) = prover
        .execute(GUEST_ELF, &stdin)
        .calculate_gas(true)
        .deferred_proof_verification(false)
        .run()
        .expect("execution failed");
    let runtime_ms = start.elapsed().as_millis();

    let total_instructions = report.total_instruction_count();
    let total_syscalls = report.total_syscall_count();

    println!("=== BN254 Pairing Benchmark (SP1 v5) ===");
    println!("Pairings: {n}");
    println!(
        "Total instructions: {}",
        total_instructions.to_formatted_string(&Locale::en)
    );
    println!(
        "Per-pairing instructions: {}",
        (total_instructions / n as u64).to_formatted_string(&Locale::en)
    );
    println!(
        "Total syscalls: {}",
        total_syscalls.to_formatted_string(&Locale::en)
    );
    println!(
        "Total SP1 gas: {}",
        report.gas.unwrap_or(0).to_formatted_string(&Locale::en)
    );
    println!("Runtime: {runtime_ms}ms");

    println!("\nCycle tracker:");
    let cycle_tracker: BTreeMap<String, u64> = report
        .cycle_tracker
        .iter()
        .map(|(k, &v)| (k.clone(), v))
        .collect();
    for (label, cycles) in &cycle_tracker {
        println!(
            "  {label}: {}",
            cycles.to_formatted_string(&Locale::en)
        );
    }

    let syscall_counts: BTreeMap<String, u64> = report
        .syscall_counts
        .iter()
        .filter(|(_, &c)| c > 0)
        .map(|(code, &c)| (format!("{code}"), c))
        .collect();
    println!("\nSyscall breakdown:");
    for (name, count) in &syscall_counts {
        println!(
            "  {name}: {}",
            count.to_formatted_string(&Locale::en)
        );
    }

    let opcode_counts: BTreeMap<String, u64> = report
        .opcode_counts
        .iter()
        .filter(|(_, &c)| c > 0)
        .map(|(op, &c)| (format!("{op}"), c))
        .collect();
    println!("\nOpcode breakdown:");
    for (name, count) in &opcode_counts {
        println!(
            "  {name}: {}",
            count.to_formatted_string(&Locale::en)
        );
    }

    let stats = PairingStats {
        num_pairings: n,
        total_instruction_count: total_instructions,
        per_pairing_instruction_count: total_instructions / n as u64,
        total_syscall_count: total_syscalls,
        total_sp1_gas: report.gas.unwrap_or(0),
        runtime_ms,
        cycle_tracker,
        syscall_counts,
        opcode_counts,
    };

    let output_path = format!("output/pairing-v5-n{n}.json");
    std::fs::create_dir_all("output").expect("failed to create output directory");
    let file = std::fs::File::create(&output_path).expect("failed to create output file");
    serde_json::to_writer_pretty(file, &stats).expect("failed to write stats JSON");
    println!("\nWrote stats to {output_path}");
}
