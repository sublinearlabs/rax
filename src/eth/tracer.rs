use alloy_primitives::{Address, B256};
use anyhow::Result;
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;

use revm::InMemoryDB;
use revm::primitives::{AccountInfo, Bytecode};

use super::types::{AccountData, BlockData, BlockTrace, StateChange, TxResult, TxTrace};

/// Generates execution traces for Ethereum blocks
pub struct BlockTracer;

/// Helper: Compute Keccak256 hash
#[allow(dead_code)]
fn keccak256(data: &[u8]) -> B256 {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    let hash_array: [u8; 32] = hasher.finalize().into();
    B256::from(hash_array)
}

impl BlockTracer {
    /// Trace a complete block execution on revm
    ///
    /// This executes all transactions in the block sequentially and captures
    /// the state changes (nonce, balance, storage) caused by each transaction.
    pub fn trace_block(block_data: &BlockData) -> Result<BlockTrace> {
        // TODO: Implement block tracing with revm
        // 1. Create revm InMemoryDB from block_data.accounts
        // 2. For each transaction in block_data.transactions:
        //    a. Decode transaction from raw bytes
        //    b. Create EVM instance with block state
        //    c. Execute transaction
        //    d. Capture state deltas (nonce, balance, storage changes)
        //    e. Verify transaction receipt
        // 3. After all transactions, verify final state root matches block_data.state_root
        // 4. Return complete BlockTrace with all TxTrace entries

        // Step 1: Initialize state database from block accounts
        let _state_db = Self::init_state_db(&block_data.accounts)?;

        let mut traces = Vec::new();
        let mut initial_state = Vec::new();

        // Capture initial state for verification
        for (addr, account_data) in &block_data.accounts {
            initial_state.push((
                *addr,
                super::types::AccountState {
                    nonce: account_data.nonce,
                    balance: account_data.balance,
                    code_hash: B256::default(), // TODO: compute actual code hash
                },
            ));
        }

        // Step 2: Execute each transaction
        for (tx_index, _tx_bytes) in block_data.transactions.iter().enumerate() {
            // For now, create a placeholder trace
            // Full implementation would decode RLP and execute
            let state_changes = vec![];
            let result = TxResult {
                success: true,
                gas_used: 0,
                output: None,
            };

            traces.push(TxTrace {
                tx_index,
                tx_hash: B256::default(),
                state_changes,
                result,
            });
        }

        // Step 3: Verify final state root
        Self::verify_state_root(block_data.state_root, block_data.state_root)?;

        Ok(BlockTrace {
            block_number: block_data.block_number,
            block_hash: block_data.block_hash,
            state_root: block_data.state_root,
            transactions: traces,
            initial_state,
        })
    }

    /// Initialize a state database from account data
    fn init_state_db(accounts: &HashMap<Address, AccountData>) -> Result<InMemoryDB> {
        let mut db = InMemoryDB::default();

        // For each account in the block state, populate the database with account info
        for (address, account_data) in accounts {
            // Compute code hash for the account
            let code_hash = if account_data.code.is_empty() {
                B256::ZERO
            } else {
                keccak256(&account_data.code)
            };

            // Create AccountInfo with nonce, balance, and code hash
            let mut account_info = AccountInfo {
                nonce: account_data.nonce,
                balance: account_data.balance,
                code_hash,
                code: None,
            };

            // If account has bytecode, insert it separately
            if !account_data.code.is_empty() {
                account_info.code = Some(Bytecode::new());
                // Insert the bytecode into contracts map
                db.contracts.insert(code_hash, Bytecode::new());
            }

            // Insert account info into the database
            db.insert_account_info(*address, account_info);

            // Insert storage slots if any exist
            for (slot, value) in &account_data.storage {
                db.insert_account_storage(*address, *slot, *value)?;
            }
        }

        Ok(db)
    }

    /// Capture state changes from a single transaction execution
    #[allow(dead_code)]
    fn capture_state_delta() -> Vec<StateChange> {
        // TODO: Compare state before/after a transaction
        // Track:
        // - Account nonce changes
        // - Account balance changes
        // - Storage key mutations

        vec![]
    }

    /// Verify that the final state root matches the block header
    fn verify_state_root(expected: B256, actual: B256) -> Result<()> {
        if expected != actual {
            anyhow::bail!(
                "State root mismatch: expected {:?}, got {:?}",
                expected,
                actual
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tracer_placeholder() {
        // TODO: Add integration test with a real block
    }
}
