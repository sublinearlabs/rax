//! Ethereum fetch command

use crate::fetcher::EthFetcher;
use crate::types::BlockData;
use colored::*;
use riscv::cli::common::{print_header, print_info, CliError, CliResult};
use std::io::Write;

/// Block fetch result data
#[derive(Debug)]
pub struct FetchResult {
    pub block_number: u64,
    pub block_hash: String,
    pub state_root: String,
    pub tx_count: usize,
    pub timestamp: u64,
    pub miner: String,
    pub gas_used: u64,
    pub gas_limit: u64,
}

/// Execute the fetch command
pub fn execute_fetch(
    block: &str,
    rpc_url: Option<&str>,
    format: &str,
    output: Option<&str>,
) -> CliResult<()> {
    print_header("ETH CLI - Block Fetcher");

    // Validate RPC URL
    let rpc_url = rpc_url
        .ok_or_else(|| CliError::new("RPC URL is required. Use --rpc-url <URL>".to_string()))?;

    // Create fetcher
    print_info("Creating Ethereum RPC client...");
    let fetcher = EthFetcher::new(rpc_url)
        .map_err(|e| CliError::new(format!("Failed to create EthFetcher: {}", e)))?;

    // Parse block number
    print_info(&format!("Parsing block: {}", block));
    let block_number: u64 = block.parse().map_err(|_| {
        CliError::new(format!(
            "Invalid block number '{}'. Expected a decimal number.",
            block
        ))
    })?;

    // Fetch block data asynchronously
    print_info(&format!("Fetching block {} from RPC...", block_number));
    let block_data = tokio::runtime::Runtime::new()
        .map_err(|e| CliError::new(format!("Failed to create async runtime: {}", e)))?
        .block_on(async {
            fetcher
                .fetch_block_data(block_number)
                .await
                .map_err(|e| CliError::new(format!("Failed to fetch block: {}", e)))
        })?;

    // Extract summary information
    let result = extract_fetch_result(&block_data)?;

    // Format output as string
    let output_text = format_fetch_output(format, &result)?;

    // Display to stdout
    println!("{}", output_text);

    // Save to file if requested
    if let Some(file_path) = output {
        write_fetch_to_file(file_path, &output_text)?;
        print_info(&format!("Fetch output written to: {}", file_path));
    }

    Ok(())
}

/// Extract key information from block data
fn extract_fetch_result(block_data: &BlockData) -> CliResult<FetchResult> {
    // Extract fields from raw_block JSON
    let raw = &block_data.raw_block;

    let timestamp = raw["timestamp"]
        .as_str()
        .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0);

    let miner = raw["miner"].as_str().unwrap_or("unknown").to_string();

    let gas_used = raw["gasUsed"]
        .as_str()
        .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0);

    let gas_limit = raw["gasLimit"]
        .as_str()
        .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0);

    Ok(FetchResult {
        block_number: block_data.block_number,
        block_hash: block_data.block_hash.to_string(),
        state_root: block_data.state_root.to_string(),
        tx_count: block_data.transactions.len(),
        timestamp,
        miner,
        gas_used,
        gas_limit,
    })
}

/// Format fetch output based on format type
fn format_fetch_output(format: &str, result: &FetchResult) -> CliResult<String> {
    match format {
        "text" => format_fetch_text(result),
        "json" => format_fetch_json(result),
        "csv" => format_fetch_csv(result),
        _ => Err(CliError::new(format!(
            "Unknown output format: '{}'. Use: text, json, csv",
            format
        ))),
    }
}

/// Format fetch output as human-readable text
fn format_fetch_text(result: &FetchResult) -> CliResult<String> {
    let mut output = String::new();

    output.push_str(&format!("\n{}\n", "Block Information".bold()));
    output.push_str(&format!("{}\n", "-".repeat(80)));
    output.push_str(&format!("  Block Number:        {}\n", result.block_number));
    output.push_str(&format!("  Block Hash:          {}\n", result.block_hash));
    output.push_str(&format!("  State Root:          {}\n", result.state_root));
    output.push_str(&format!("  Transactions:        {}\n", result.tx_count));
    output.push_str(&format!("  Timestamp:           {}\n", result.timestamp));
    output.push_str(&format!("  Miner:               {}\n", result.miner));
    output.push_str(&format!("  Gas Used:            {}\n", result.gas_used));
    output.push_str(&format!("  Gas Limit:           {}\n", result.gas_limit));
    output.push_str(&format!("{}\n", "-".repeat(80)));

    Ok(output)
}

/// Format fetch output as JSON
fn format_fetch_json(result: &FetchResult) -> CliResult<String> {
    let json = serde_json::json!({
        "block_number": result.block_number,
        "block_hash": result.block_hash,
        "state_root": result.state_root,
        "transaction_count": result.tx_count,
        "timestamp": result.timestamp,
        "miner": result.miner,
        "gas_used": result.gas_used,
        "gas_limit": result.gas_limit,
    });

    Ok(serde_json::to_string_pretty(&json).unwrap())
}

/// Format fetch output as CSV
fn format_fetch_csv(result: &FetchResult) -> CliResult<String> {
    let mut output = String::new();
    output.push_str(
        "block_number,block_hash,state_root,transaction_count,timestamp,miner,gas_used,gas_limit\n",
    );
    output.push_str(&format!(
        "{},{},{},{},{},{},{},{}\n",
        result.block_number,
        result.block_hash,
        result.state_root,
        result.tx_count,
        result.timestamp,
        result.miner,
        result.gas_used,
        result.gas_limit
    ));
    Ok(output)
}

/// Write fetch output to a file
fn write_fetch_to_file(file_path: &str, content: &str) -> CliResult<()> {
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
