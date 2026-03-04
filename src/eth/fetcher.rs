use alloy_primitives::{Address, B256, U256};
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;

use super::types::{AccountData, BlockData};

/// Fetches block data from Ethereum RPC via JSON-RPC
pub struct EthFetcher {
    rpc_url: String,
    client: reqwest::Client,
}

impl EthFetcher {
    /// Create a new fetcher with the given RPC URL
    pub fn new(rpc_url: &str) -> Result<Self> {
        Ok(Self {
            rpc_url: rpc_url.to_string(),
            client: reqwest::Client::new(),
        })
    }

    /// Fetch block data from Ethereum mainnet
    pub async fn fetch_block_data(&self, block_number: u64) -> Result<BlockData> {
        // Fetch block header with transactions
        let block_hex = format!("0x{:x}", block_number);

        let block = self
            .json_rpc_call::<serde_json::Value>(
                "eth_getBlockByNumber",
                &[json!(block_hex), json!(true)],
            )
            .await?;

        let block = block.ok_or_else(|| anyhow::anyhow!("Block {} not found", block_number))?;

        // Extract block data
        let block_hash = B256::from_str(
            block["hash"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing block hash"))?,
        )
        .map_err(|e| anyhow::anyhow!("Invalid block hash: {}", e))?;

        let state_root = B256::from_str(
            block["stateRoot"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing state root"))?,
        )
        .map_err(|e| anyhow::anyhow!("Invalid state root: {}", e))?;

        // Extract transactions
        let mut transactions = Vec::new();
        if let Some(txs) = block["transactions"].as_array() {
            for tx in txs {
                // Store raw transaction data (we'll encode to RLP later)
                if let Some(tx_str) = tx["input"].as_str().or(tx["data"].as_str()) {
                    // For now, store the raw bytes from input field
                    let tx_bytes = hex::decode(tx_str.strip_prefix("0x").unwrap_or(tx_str))?;
                    transactions.push(tx_bytes);
                }
            }
        }

        // Extract touched addresses and fetch their state
        let accounts = self
            .fetch_touched_accounts(block_number - 1, &block)
            .await?;

        Ok(BlockData {
            block_number,
            block_hash,
            state_root,
            transactions,
            accounts,
        })
    }

    /// Fetch state of all accounts touched by the block
    async fn fetch_touched_accounts(
        &self,
        block_number: u64,
        block: &serde_json::Value,
    ) -> Result<HashMap<Address, AccountData>> {
        let mut accounts = HashMap::new();
        let mut touched_addresses = std::collections::HashSet::new();

        // Extract addresses from transactions
        if let Some(txs) = block["transactions"].as_array() {
            for tx in txs {
                if let Some(from_str) = tx["from"].as_str() {
                    touched_addresses.insert(
                        Address::from_str(from_str)
                            .map_err(|e| anyhow::anyhow!("Invalid from address: {}", e))?,
                    );
                }
                if let Some(to_str) = tx["to"].as_str() {
                    if to_str != "0x" {
                        touched_addresses.insert(
                            Address::from_str(to_str)
                                .map_err(|e| anyhow::anyhow!("Invalid to address: {}", e))?,
                        );
                    }
                }
            }
        }

        // Fetch state for each touched address at block start
        let block_hex = format!("0x{:x}", block_number);

        for address in touched_addresses {
            let addr_str = format!("0x{:x}", address);

            // Get nonce
            let nonce_str = self
                .json_rpc_call::<String>(
                    "eth_getTransactionCount",
                    &[json!(addr_str), json!(block_hex)],
                )
                .await?
                .ok_or_else(|| anyhow::anyhow!("Failed to get nonce for {}", address))?;
            let nonce =
                u64::from_str_radix(nonce_str.strip_prefix("0x").unwrap_or(&nonce_str), 16)?;

            // Get balance
            let balance_str = self
                .json_rpc_call::<String>("eth_getBalance", &[json!(addr_str), json!(block_hex)])
                .await?
                .ok_or_else(|| anyhow::anyhow!("Failed to get balance for {}", address))?;
            let balance =
                U256::from_str_radix(balance_str.strip_prefix("0x").unwrap_or(&balance_str), 16)?;

            // Get code
            let code_str = self
                .json_rpc_call::<String>("eth_getCode", &[json!(addr_str), json!(block_hex)])
                .await?
                .ok_or_else(|| anyhow::anyhow!("Failed to get code for {}", address))?;
            let code = hex::decode(code_str.strip_prefix("0x").unwrap_or(&code_str))?;

            accounts.insert(
                address,
                AccountData {
                    nonce,
                    balance,
                    code,
                    storage: HashMap::new(),
                },
            );
        }

        Ok(accounts)
    }

    /// Make a JSON-RPC call to the Ethereum node
    async fn json_rpc_call<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: &[serde_json::Value],
    ) -> Result<Option<T>> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1,
        });

        let response = self
            .client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await?;
        let data: serde_json::Value = response.json().await?;

        if let Some(error) = data.get("error") {
            return Err(anyhow::anyhow!(
                "JSON-RPC error: {}",
                error
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
            ));
        }

        if let Some(result) = data.get("result") {
            if result.is_null() {
                return Ok(None);
            }
            let parsed = serde_json::from_value(result.clone())?;
            Ok(Some(parsed))
        } else {
            Ok(None)
        }
    }

    /// Get the RPC URL being used
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    /// Fetch transaction receipt from the network
    ///
    /// Returns the on-chain receipt for a given transaction hash.
    /// Used for verification against locally computed execution results.
    pub async fn fetch_tx_receipt(&self, tx_hash: B256) -> Result<serde_json::Value> {
        let receipt = self
            .json_rpc_call::<serde_json::Value>(
                "eth_getTransactionReceipt",
                &[json!(format!("{:?}", tx_hash))],
            )
            .await?;

        receipt.ok_or_else(|| anyhow::anyhow!("Receipt not found for tx: {}", tx_hash))
    }

    /// Fetch a block summary from the network
    ///
    /// Returns block data (hash, state root, transaction count) for verification.
    /// For complete block data with account state, use fetch_block_data() instead.
    pub async fn fetch_block_summary(&self, block_number: u64) -> Result<serde_json::Value> {
        let block_hex = format!("0x{:x}", block_number);
        let block = self
            .json_rpc_call::<serde_json::Value>(
                "eth_getBlockByNumber",
                &[json!(block_hex), json!(false)],
            )
            .await?;

        block.ok_or_else(|| anyhow::anyhow!("Block not found: {}", block_number))
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
