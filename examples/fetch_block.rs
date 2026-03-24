/// Example: Fetch a live Ethereum block data
///
/// This example demonstrates fetching block data from Ethereum mainnet
/// and printing the block information.
///
/// Usage:
/// ```bash
/// ALCHEMY_API_KEY=your_key cargo run --example fetch_block
/// ```
use eth_utils::EthFetcher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Get API key from environment
    let api_key = std::env::var("ALCHEMY_API_KEY").unwrap_or_else(|_| "demo".to_string());

    let rpc_url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", api_key);

    println!("Fetching block data from: {}", rpc_url);

    let fetcher = EthFetcher::new(&rpc_url)?;

    // Fetch a recent block (adjust this to a real block number)
    // For testing, you might want to use a specific block
    let block_number = 21000000u64; // Example block number

    println!("Fetching block {}...", block_number);

    match fetcher.fetch_block_data(block_number).await {
        Ok(block_data) => {
            println!("\n✓ Successfully fetched block data!");
            println!("  Block Number: {}", block_data.block_number);
            println!("  Block Hash: {:?}", block_data.block_hash);
            println!("  State Root: {:?}", block_data.state_root);
            println!("  Transactions: {}", block_data.transactions.len());
            println!("  Touched Accounts: {}", block_data.accounts.len());

            // Print first few touched accounts
            println!("\n  First 5 touched accounts:");
            for (i, (addr, account)) in block_data.accounts.iter().take(5).enumerate() {
                println!(
                    "    {}. {} - nonce: {}, balance: {}",
                    i + 1,
                    addr,
                    account.nonce,
                    account.balance
                );
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to fetch block: {}", e);
            eprintln!("\nNote: This example requires a valid Alchemy API key.");
            eprintln!("Set the ALCHEMY_API_KEY environment variable:");
            eprintln!("  export ALCHEMY_API_KEY=your_actual_key");
            eprintln!("  cargo run --example fetch_block");
        }
    }

    Ok(())
}
