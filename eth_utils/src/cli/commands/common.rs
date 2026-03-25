//! Common utilities for Ethereum CLI commands

use crate::fetcher::EthFetcher;
use crate::types::BlockData;
use riscv::cli::common::{print_info, CliError, CliResult};

/// Fetch block data from Ethereum RPC with unified error handling
pub fn fetch_block_data(block: &str, rpc_url: &str) -> CliResult<BlockData> {
    // Validate RPC URL
    if rpc_url.is_empty() {
        return Err(CliError::new(
            "RPC URL is required. Use --rpc-url <URL>".to_string(),
        ));
    }

    // Parse block number
    print_info(&format!("Parsing block: {}", block));
    let block_number: u64 = block.parse().map_err(|_| {
        CliError::new(format!(
            "Invalid block number '{}'. Expected a decimal number.",
            block
        ))
    })?;

    // Create fetcher
    print_info("Creating Ethereum RPC client...");
    let fetcher = EthFetcher::new(rpc_url)
        .map_err(|e| CliError::new(format!("Failed to create EthFetcher: {}", e)))?;

    // Fetch block data asynchronously
    print_info(&format!("Fetching block {} from RPC...", block_number));
    tokio::runtime::Runtime::new()
        .map_err(|e| CliError::new(format!("Failed to create async runtime: {}", e)))?
        .block_on(async {
            fetcher
                .fetch_block_data(block_number)
                .await
                .map_err(|e| CliError::new(format!("Failed to fetch block: {}", e)))
        })
}
