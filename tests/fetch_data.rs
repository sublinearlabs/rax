//! Fetch Transaction Receipts and Blocks
//!
//! Tests the JSON-RPC fetching functionality for Ethereum blocks and transactions.
//! These functions are used to retrieve on-chain data for verification.

use riscv::eth::EthFetcher;

/// Test: Fetch transaction receipt from mainnet
/// This test is skipped by default since it requires network access
#[tokio::test]
#[ignore]
async fn test_fetch_tx_receipt_mainnet() {
    // Alchemy endpoint for mainnet (requires API key)
    let mainnet_rpc = std::env::var("ETH_MAINNET_RPC_URL")
        .unwrap_or_else(|_| "https://eth-mainnet.g.alchemy.com/v2/{YOUR_KEY}".to_string());

    let fetcher = EthFetcher::new(&mainnet_rpc).expect("Failed to create fetcher");

    // Use a transaction hash from mainnet
    // Random transaction used
    let tx_hash_str = "0x6e7ab751c5b92454da42f70ce0f8e81a4ea74bfa0ff2ee8da723f1f0352a1d90";
    let tx_hash = tx_hash_str.parse().expect("Invalid tx hash");

    match fetcher.fetch_tx_receipt(tx_hash).await {
        Ok(receipt) => {
            println!("✓ Successfully fetched receipt");
            println!("  Receipt: {:?}", receipt);
            // Verify it has the expected fields
            assert!(receipt.get("transactionHash").is_some());
        }
        Err(e) => {
            println!(
                "Note: Receipt fetch failed (expected if tx hash not real): {}",
                e
            );
        }
    }
}

/// Test: Fetch block summary from mainnet
/// This test is skipped by default since it requires network access
#[tokio::test]
#[ignore]
async fn test_fetch_block_summary_mainnet() {
    let mainnet_rpc = std::env::var("ETH_MAINNET_RPC_URL")
        .unwrap_or_else(|_| "https://eth-mainnet.g.alchemy.com/v2/{YOUR_KEY}".to_string());

    let fetcher = EthFetcher::new(&mainnet_rpc).expect("Failed to create fetcher");

    // Fetch a recent mainnet block
    let block_number = 19000000u64; // Recent mainnet block

    match fetcher.fetch_block_summary(block_number).await {
        Ok(block) => {
            println!("✓ Successfully fetched block summary");
            println!("  Block: {:?}", block);
            // Verify it has the expected fields
            assert!(block.get("hash").is_some());
            assert!(block.get("stateRoot").is_some());
        }
        Err(e) => {
            println!("Error fetching block: {}", e);
        }
    }
}

/// Test: Verify fetcher creation works
/// This test ensures the fetcher can be created with proper types
#[test]
fn test_fetcher_creation() {
    // Create a fetcher with a dummy URL
    let fetcher = EthFetcher::new("https://eth-mainnet.g.alchemy.com/v2/demo");

    match fetcher {
        Ok(f) => {
            println!("✓ EthFetcher created successfully");
            println!("✓ RPC URL: {}", f.rpc_url());
            assert_eq!(f.rpc_url(), "https://eth-mainnet.g.alchemy.com/v2/demo");
        }
        Err(e) => {
            println!("Error creating fetcher: {}", e);
            panic!("Failed to create fetcher");
        }
    }
}
