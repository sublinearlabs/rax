use alloy_primitives::{Address, B256, U256};
use anyhow::Result;
use std::collections::HashMap;

use super::types::{AccountData, BlockData};

/// Fetches block data from Ethereum RPC
pub struct EthFetcher {
    rpc_url: String,
}

impl EthFetcher {
    /// Create a new fetcher with the given RPC URL
    pub fn new(rpc_url: &str) -> Result<Self> {
        Ok(Self {
            rpc_url: rpc_url.to_string(),
        })
    }

    /// Fetch block data from Ethereum mainnet
    pub async fn fetch_block_data(&self, block_number: u64) -> Result<BlockData> {
        // TODO: Implement block fetching
        // 1. Get block header (block_hash, state_root, etc)
        // 2. Get all transactions in block (RLP encoded)
        // 3. Get account state at block start
        //    - For each account touched by the block:
        //      - nonce, balance
        //      - code (if contract)
        //      - storage keys accessed
        // 4. Validate block structure

        todo!("Fetch block data from RPC: {}", self.rpc_url)
    }

    /// Get the RPC URL being used
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetcher_creation() {
        let fetcher = EthFetcher::new("https://eth-mainnet.g.alchemy.com/v2/demo");
        assert!(fetcher.is_ok());
        assert_eq!(
            fetcher.unwrap().rpc_url(),
            "https://eth-mainnet.g.alchemy.com/v2/demo"
        );
    }
}
