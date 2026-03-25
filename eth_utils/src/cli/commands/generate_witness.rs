//! Ethereum generate-witness command - execution witness generation

use super::common::fetch_block_data;
use crate::BlockTracer;
use colored::*;
use riscv::cli::common::{print_header, print_info, CliError, CliResult};
use std::io::Write;

/// Witness generation result data
#[derive(Debug)]
pub struct GenerateWitnessResult {
    pub block_number: u64,
    pub block_hash: String,
    pub transactions_traced: usize,
    pub state_changes: usize,
    pub witness_generated: bool,
    pub witness_size_bytes: usize,
}

/// Execute the generate-witness command
pub fn execute_generate_witness(
    block: &str,
    rpc_url: Option<&str>,
    format: &str,
    output: Option<&str>,
) -> CliResult<()> {
    print_header("ETH CLI - Execution Witness Generator");

    // Validate RPC URL
    let rpc_url = rpc_url
        .ok_or_else(|| CliError::new("RPC URL is required. Use --rpc-url <URL>".to_string()))?;

    // Fetch block data
    let block_data = fetch_block_data(block, rpc_url)?;

    // Trace block execution
    print_info("Tracing block execution...");
    let block_trace = BlockTracer::trace_block(&block_data)
        .map_err(|e| CliError::new(format!("Failed to trace block: {}", e)))?;

    // Generate witness
    print_info("Generating execution witness...");
    let witness_json = BlockTracer::generate_witness(&block_data, &block_trace)
        .map_err(|e| CliError::new(format!("Failed to generate witness: {}", e)))?;

    // Create result
    let witness_str = serde_json::to_string(&witness_json).unwrap_or_default();
    let result = GenerateWitnessResult {
        block_number: block_data.block_number,
        block_hash: block_data.block_hash.to_string(),
        transactions_traced: block_trace.transactions.len(),
        state_changes: block_trace
            .transactions
            .iter()
            .map(|tx| tx.state_changes.len())
            .sum(),
        witness_generated: true,
        witness_size_bytes: witness_str.len(),
    };

    // Format output as string
    let output_text = format_witness_output(format, &result, &witness_json)?;

    // Display to stdout
    println!("{}", output_text);

    // Save to file if requested
    if let Some(file_path) = output {
        write_witness_to_file(file_path, &witness_json)?;
    }

    Ok(())
}

/// Format witness output based on format type
fn format_witness_output(
    format: &str,
    result: &GenerateWitnessResult,
    witness_json: &serde_json::Value,
) -> CliResult<String> {
    match format {
        "text" => format_witness_text(result),
        "json" => format_witness_json(witness_json),
        "csv" => format_witness_csv(result),
        _ => Err(CliError::new(format!(
            "Unknown output format: '{}'. Use: text, json, csv",
            format
        ))),
    }
}

/// Format witness output as human-readable text
fn format_witness_text(result: &GenerateWitnessResult) -> CliResult<String> {
    let mut output = String::new();

    output.push_str(&format!("\n{}\n", "Execution Witness Generated".bold()));
    output.push_str(&format!("{}\n", "-".repeat(80)));
    output.push_str(&format!("  Block Number:        {}\n", result.block_number));
    output.push_str(&format!("  Block Hash:          {}\n", result.block_hash));
    output.push_str(&format!(
        "  Transactions Traced: {}\n",
        result.transactions_traced
    ));
    output.push_str(&format!(
        "  State Changes:       {}\n",
        result.state_changes
    ));
    output.push_str(&format!(
        "  Witness Generated:   {}\n",
        if result.witness_generated {
            "yes"
        } else {
            "no"
        }
    ));
    output.push_str(&format!(
        "  Witness Size:        {} bytes\n",
        result.witness_size_bytes
    ));
    output.push_str(&format!("{}\n", "-".repeat(80)));

    Ok(output)
}

/// Format witness output as JSON
fn format_witness_json(witness_json: &serde_json::Value) -> CliResult<String> {
    Ok(serde_json::to_string_pretty(witness_json).unwrap_or_default())
}

/// Format witness output as CSV
fn format_witness_csv(result: &GenerateWitnessResult) -> CliResult<String> {
    let mut output = String::new();
    output.push_str("block_number,block_hash,transactions_traced,state_changes,witness_generated,witness_size_bytes\n");
    output.push_str(&format!(
        "{},{},{},{},{},{}\n",
        result.block_number,
        result.block_hash,
        result.transactions_traced,
        result.state_changes,
        result.witness_generated,
        result.witness_size_bytes
    ));
    Ok(output)
}

/// Write witness to file as hex-encoded bytes
/// This allows direct use as input to RISC-V VM
fn write_witness_to_file(file_path: &str, witness_json: &serde_json::Value) -> CliResult<()> {
    // Serialize witness JSON to bytes
    let witness_bytes = serde_json::to_vec(witness_json)
        .map_err(|e| CliError::new(format!("Failed to serialize witness: {}", e)))?;

    // Hex-encode the bytes
    let witness_hex = hex::encode(&witness_bytes);

    // Write to file
    let mut file = std::fs::File::create(file_path).map_err(|e| {
        CliError::new(format!(
            "Failed to create output file '{}': {}",
            file_path, e
        ))
    })?;

    file.write_all(witness_hex.as_bytes())
        .map_err(|e| CliError::new(format!("Failed to write to file '{}': {}", file_path, e)))?;

    print_info(&format!("Witness saved (hex-encoded) to: {}", file_path));
    Ok(())
}
