//! Verify Ethereum block by executing witness on RISC-V VM
//!
//! This command delegates to eth-cli for witness generation, then executes on RISC-V VM.

use crate::cli::common::{check_file_exists, print_header, print_info, CliError, CliResult};
use crate::{init_from_elf, trace::NoopTracer, Runner, VM};
use colored::*;
use std::fs;
use std::io::Write;
use std::process::Command;

/// Block verification result data
#[derive(Debug)]
pub struct VerifyBlockResult {
    pub block_number: u64,
    pub block_hash: String,
    pub transactions_traced: usize,
    pub witness_size_bytes: usize,
    pub vm_exit_code: u64,
    pub vm_cycles: u64,
    pub verification_passed: bool,
}

/// Execute the verify-block command
///
/// This command:
/// 1. Uses eth-cli to fetch and generate witness for an Ethereum block
/// 2. Executes the witness on the provided RISC-V verifier binary
pub fn execute_verify_block(
    block: &str,
    binary: &str,
    rpc_url: &str,
    witness: Option<&str>,
    format: &str,
    output: Option<&str>,
) -> CliResult<()> {
    print_header("RISC-V CLI - Block Verifier");

    // Validate binary file exists
    check_file_exists(binary)?;

    // Create witness file path - use provided path or default temp file
    let witness_file = witness
        .map(|p| p.to_string())
        .unwrap_or_else(|| format!("/tmp/witness_{}", block));

    // Check if witness file already exists
    let (witness_bytes, witness_json) = if fs::metadata(&witness_file).is_ok() {
        print_info(&format!("Using existing witness file: {}", witness_file));

        // Read hex-encoded witness file
        let hex_data = fs::read_to_string(&witness_file)
            .map_err(|e| CliError::new(format!("Failed to read witness file: {}", e)))?;

        // Decode from hex
        let witness_bytes = hex::decode(&hex_data)
            .map_err(|e| CliError::new(format!("Failed to decode witness hex: {}", e)))?;

        // Parse JSON to extract metadata
        let witness_json: serde_json::Value = serde_json::from_slice(&witness_bytes)
            .map_err(|e| CliError::new(format!("Failed to parse witness JSON: {}", e)))?;

        (witness_bytes, witness_json)
    } else {
        // Call eth-cli to generate witness
        print_info(&format!(
            "Generating witness for block {} via eth-cli...",
            block
        ));

        let eth_cli_status = Command::new("eth-cli")
            .args(&["generate-witness", block])
            .args(&["--rpc-url", rpc_url])
            .args(&["--output", &witness_file])
            .status()
            .map_err(|e| {
                CliError::new(format!(
                    "Failed to run eth-cli: {}. Make sure eth-cli is in your PATH.",
                    e
                ))
            })?;

        if !eth_cli_status.success() {
            return Err(CliError::new(
                "eth-cli witness generation failed".to_string(),
            ));
        }

        // Read the hex-encoded witness file
        print_info("Reading witness from file...");
        let hex_data = fs::read_to_string(&witness_file)
            .map_err(|e| CliError::new(format!("Failed to read witness file: {}", e)))?;

        // Decode from hex
        let witness_bytes = hex::decode(&hex_data)
            .map_err(|e| CliError::new(format!("Failed to decode witness hex: {}", e)))?;

        // Parse JSON to extract metadata
        let witness_json: serde_json::Value = serde_json::from_slice(&witness_bytes)
            .map_err(|e| CliError::new(format!("Failed to parse witness JSON: {}", e)))?;

        print_info(&format!("Witness saved to: {}", witness_file));

        (witness_bytes, witness_json)
    };

    // Extract block information from witness JSON
    let block_number = witness_json["block"]["header"]["number"]
        .as_u64()
        .unwrap_or(0);
    let block_hash = witness_json["block"]["header"]["hash"]
        .as_str()
        .or_else(|| witness_json["block"]["header"]["state_root"].as_str())
        .unwrap_or("unknown")
        .to_string();
    let transactions_traced = witness_json["block"]["body"]["transactions"]
        .as_array()
        .map(|arr| arr.len())
        .unwrap_or(0);

    // Load RISC-V VM with binary
    print_info(&format!("Loading RISC-V verifier binary: {}", binary));

    // TODO: move to full tracer once tracing is finalized
    let mut vm: VM<NoopTracer> = init_from_elf(binary);

    // Create runner and feed witness as input
    print_info("Executing witness on RISC-V VM...");
    let mut runner = Runner::new();
    runner.set_input_stream(witness_bytes.clone());

    runner.run_with_timing(&mut vm);

    let exit_code = vm.exit_code();
    let cycles = runner.cycles();

    // Determine if verification passed (exit code 0)
    let verification_passed = exit_code == 0;

    // Create result
    let result = VerifyBlockResult {
        block_number,
        block_hash,
        transactions_traced,
        witness_size_bytes: witness_bytes.len(),
        vm_exit_code: exit_code,
        vm_cycles: cycles,
        verification_passed,
    };

    // Format output - note: elapsed time will be included in runner's output
    let output_text = format_output(format, &result)?;

    // Display to stdout
    println!("{}", output_text);

    // Save to file if requested
    if let Some(file_path) = output {
        write_output_to_file(file_path, &output_text)?;
        print_info(&format!("Verification output written to: {}", file_path));
    }

    // Witness file is always kept for potential reuse
    print_info(&format!("Witness file available at: {}", witness_file));

    Ok(())
}

/// Format output based on requested format
fn format_output(format: &str, result: &VerifyBlockResult) -> CliResult<String> {
    match format {
        "text" => {
            let status = if result.verification_passed {
                "✓ PASSED".green().bold()
            } else {
                "✗ FAILED".red().bold()
            };

            let mut output = String::new();
            output.push_str(&format!("\n{}\n", "Block Verification Results".bold()));
            output.push_str(&format!("{}\n", "-".repeat(70)));
            output.push_str(&format!("  Status:                {}\n", status));
            output.push_str(&format!(
                "  Block Number:         {}\n",
                result.block_number
            ));
            output.push_str(&format!("  Block Hash:           {}\n", result.block_hash));
            output.push_str(&format!(
                "  Transactions Traced:  {}\n",
                result.transactions_traced
            ));
            output.push_str(&format!(
                "  Witness Size:         {} bytes\n",
                result.witness_size_bytes
            ));
            output.push_str(&format!(
                "  VM Exit Code:         {}\n",
                result.vm_exit_code
            ));
            output.push_str(&format!("  VM Cycles:            {}\n", result.vm_cycles));
            output.push_str(&format!("{}\n", "-".repeat(70)));
            Ok(output)
        }
        "json" => {
            let json = serde_json::json!({
                "status": if result.verification_passed { "passed" } else { "failed" },
                "block_number": result.block_number,
                "block_hash": result.block_hash,
                "transactions_traced": result.transactions_traced,
                "witness_size_bytes": result.witness_size_bytes,
                "vm_exit_code": result.vm_exit_code,
                "vm_cycles": result.vm_cycles,
            });
            Ok(serde_json::to_string_pretty(&json)
                .map_err(|e| CliError::new(format!("Failed to format JSON: {}", e)))?)
        }
        "csv" => {
            let status = if result.verification_passed {
                "passed"
            } else {
                "failed"
            };
            let output = format!(
                "status,block_number,block_hash,transactions_traced,witness_size_bytes,vm_exit_code,vm_cycles\n{},{},{},{},{},{},{}",
                status,
                result.block_number,
                result.block_hash,
                result.transactions_traced,
                result.witness_size_bytes,
                result.vm_exit_code,
                result.vm_cycles,
            );
            Ok(output)
        }
        _ => Err(CliError::new(format!(
            "Unknown output format: '{}'. Expected text, json, or csv",
            format
        ))),
    }
}

/// Write verification output to file
fn write_output_to_file(file_path: &str, content: &str) -> CliResult<()> {
    let mut file = std::fs::File::create(file_path).map_err(|e| {
        CliError::new(format!(
            "Failed to create output file '{}': {}",
            file_path, e
        ))
    })?;

    file.write_all(content.as_bytes())
        .map_err(|e| CliError::new(format!("Failed to write to file '{}': {}", file_path, e)))?;

    Ok(())
}
