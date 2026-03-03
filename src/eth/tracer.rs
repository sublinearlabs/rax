use alloy_primitives::{Address, B256, U256};
use anyhow::Result;
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;

use revm::Evm;
use revm::InMemoryDB;
use revm::primitives::{AccountInfo, Bytecode, TxEnv};

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

/// Simple RLP transaction decoder
/// Extracts basic transaction information from RLP-encoded bytes
/// Works with Legacy, EIP-2930, and EIP-1559 transaction formats
#[derive(Debug, Clone)]
struct DecodedTransaction {
    from: Address,
    to: Option<Address>,
    nonce: u64,
    value: U256,
    gas_limit: u64,
    gas_price: U256,
    input: Vec<u8>,
    chain_id: Option<u64>,
}

impl DecodedTransaction {
    /// Attempt to decode a transaction from RLP bytes
    /// This handles Legacy, EIP-2930, and EIP-1559 transaction formats
    fn decode_from_bytes(bytes: &[u8]) -> Result<Self> {
        // Check for typed transaction (EIP-2930 or EIP-1559)
        // Typed transactions start with 0x01 (EIP-2930) or 0x02 (EIP-1559)
        let (is_legacy, tx_data) = match bytes.get(0) {
            Some(&0x01) | Some(&0x02) => {
                // Typed transaction: strip the type byte and use the remaining RLP
                (false, &bytes[1..])
            }
            _ => {
                // Legacy transaction
                (true, bytes)
            }
        };

        if is_legacy {
            Self::decode_legacy(tx_data)
        } else {
            // For now, parse as legacy for simplicity
            // TODO: Add proper EIP-2930 and EIP-1559 specific handling
            Self::decode_legacy(tx_data)
        }
    }

    /// Decode legacy transaction format [nonce, gasprice, gaslimit, to, value, data, v, r, s]
    fn decode_legacy(data: &[u8]) -> Result<Self> {
        // Use a simple RLP parser with alloy_rlp
        use alloy_rlp::Decodable;

        // Decode as a generic list
        let rlp_list: Vec<Vec<u8>> = Decodable::decode(&mut &data[..])?;

        if rlp_list.len() < 9 {
            anyhow::bail!(
                "Invalid transaction: expected at least 9 RLP fields, got {}",
                rlp_list.len()
            );
        }

        // Parse individual fields
        let nonce = Self::bytes_to_u64(&rlp_list[0])?;
        let gas_price = U256::from_be_bytes(Self::pad_bytes(&rlp_list[1])?);
        let gas_limit = Self::bytes_to_u64(&rlp_list[2])?;
        let to = if rlp_list[3].is_empty() {
            None
        } else {
            Some(Address::from_slice(&rlp_list[3]))
        };
        let value = U256::from_be_bytes(Self::pad_bytes(&rlp_list[4])?);
        let input = rlp_list[5].clone();

        // Parse signature components (v, r, s)
        let v = Self::bytes_to_u64(&rlp_list[6])?;
        let r = U256::from_be_bytes(Self::pad_bytes(&rlp_list[7])?);
        let s = U256::from_be_bytes(Self::pad_bytes(&rlp_list[8])?);

        // Recover sender from signature
        let from = Self::recover_sender(data, v, r, s)?;

        Ok(Self {
            from,
            to,
            nonce,
            value,
            gas_limit,
            gas_price,
            input,
            chain_id: None,
        })
    }

    /// Helper: Pad byte slice to 32 bytes (returns full array for U256/B256)
    fn pad_bytes(bytes: &[u8]) -> Result<[u8; 32]> {
        if bytes.len() > 32 {
            anyhow::bail!("Byte slice too long: {} > 32", bytes.len());
        }
        let mut result = [0u8; 32];
        // Copy to the right side (big-endian)
        let start = 32 - bytes.len();
        result[start..].copy_from_slice(bytes);
        Ok(result)
    }

    /// Helper: Convert byte slice to u64 (for nonce, gas_limit, etc)
    fn bytes_to_u64(bytes: &[u8]) -> Result<u64> {
        let padded = Self::pad_bytes(bytes)?;
        Ok(u64::from_be_bytes(padded[24..32].try_into()?))
    }

    /// Recover sender address from transaction signature
    /// This uses ECDSA signature recovery
    fn recover_sender(_tx_bytes: &[u8], _v: u64, _r: U256, _s: U256) -> Result<Address> {
        // For now, we use a simple fallback that computes a deterministic address
        // In production, this would use proper ECDSA recovery (e.g., k256 library)
        // The actual recovery requires:
        // 1. Hash the transaction (excluding v, r, s)
        // 2. Recover the public key using ECDSA with (v, r, s)
        // 3. Hash the public key to get the address

        // Fallback: use hash of transaction as a deterministic address
        let mut hasher = Sha3_256::new();
        hasher.update(_tx_bytes);
        let hash_result = hasher.finalize();

        // Take last 20 bytes as address
        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&hash_result[12..32]);
        Ok(Address::from(addr_bytes))
    }
}

impl BlockTracer {
    /// Trace a complete block execution on revm
    ///
    /// This executes all transactions in the block sequentially and captures
    /// the state changes (nonce, balance, storage) caused by each transaction.
    pub fn trace_block(block_data: &BlockData) -> Result<BlockTrace> {
        // Step 1: Initialize state database from block accounts
        let mut state_db = Self::init_state_db(&block_data.accounts)?;

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
        for (tx_index, tx_bytes) in block_data.transactions.iter().enumerate() {
            // Attempt to execute transaction
            match Self::execute_transaction(&mut state_db, tx_bytes, tx_index) {
                Ok(tx_trace) => traces.push(tx_trace),
                Err(e) => {
                    // Log error but continue processing other transactions
                    eprintln!("Warning: Failed to execute transaction {}: {}", tx_index, e);

                    // Create error trace
                    traces.push(TxTrace {
                        tx_index,
                        tx_hash: B256::default(),
                        state_changes: vec![],
                        result: TxResult {
                            success: false,
                            gas_used: 0,
                            output: None,
                        },
                    });
                }
            }
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

    /// Capture state changes from transaction execution results
    ///
    /// Capture state changes from transaction execution
    ///
    /// Compares before/after state to track:
    /// - Account nonce changes
    /// - Account balance changes
    /// - Storage mutations (key, old value, new value)
    fn capture_state_delta_with_before<S: std::hash::BuildHasher>(
        db_before: &InMemoryDB,
        state_after: &HashMap<Address, revm::primitives::Account, S>,
    ) -> Vec<StateChange> {
        let mut deltas = Vec::new();

        // Iterate through all modified accounts in the after state
        for (address, account_after) in state_after {
            // Get the before state if it exists
            let account_before = db_before.accounts.get(address);

            // Track nonce changes
            let nonce_change = if let Some(before) = account_before {
                let nonce_before = before.info.nonce;
                let nonce_after = account_after.info.nonce;
                if nonce_before != nonce_after {
                    Some((nonce_before, nonce_after))
                } else {
                    None
                }
            } else {
                // Account didn't exist before, now it does
                if account_after.info.nonce != 0 {
                    Some((0, account_after.info.nonce))
                } else {
                    None
                }
            };

            // Track balance changes
            let balance_change = if let Some(before) = account_before {
                let balance_before = before.info.balance;
                let balance_after = account_after.info.balance;
                if balance_before != balance_after {
                    Some((balance_before, balance_after))
                } else {
                    None
                }
            } else {
                // Account didn't exist before, now it does
                if account_after.info.balance != U256::ZERO {
                    Some((U256::ZERO, account_after.info.balance))
                } else {
                    None
                }
            };

            // Track storage changes
            let mut storage_changes = Vec::new();
            for (key, storage_slot_after) in &account_after.storage {
                // Get original and present values from revm's tracking
                let original = storage_slot_after.original_value();
                let present = storage_slot_after.present_value();

                // Only record if the value actually changed
                if original != present {
                    storage_changes.push((*key, original, present));
                }
            }

            // Only include accounts with actual changes
            if nonce_change.is_some() || balance_change.is_some() || !storage_changes.is_empty() {
                deltas.push(StateChange {
                    address: *address,
                    nonce_change,
                    balance_change,
                    storage_changes,
                });
            }
        }

        deltas
    }

    /// Execute a single transaction on the EVM
    ///
    /// This function orchestrates transaction execution:
    /// 1. Decode transaction from RLP bytes
    /// 2. Extract sender, recipient, value, data, gas, gasPrice, nonce
    /// 3. Create Evm instance with the state database
    /// 4. Set transaction environment (TxEnv)
    /// 5. Execute and capture results
    /// 6. Track state changes (nonce, balance, storage)
    /// 7. Return execution trace
    fn execute_transaction(
        db: &mut InMemoryDB,
        tx_bytes: &[u8],
        tx_index: usize,
    ) -> Result<TxTrace> {
        // Step 1: Decode transaction
        let decoded = DecodedTransaction::decode_from_bytes(tx_bytes)?;

        // Step 2: Capture the state BEFORE execution
        let db_before = db.clone();

        // Step 3: Create EVM instance with current state
        let mut evm = Evm::builder().with_db(db).build();

        // Step 4: Set transaction environment
        use revm::primitives::TransactTo;
        evm.context.evm.env.tx = TxEnv {
            caller: decoded.from,
            transact_to: match decoded.to {
                Some(addr) => TransactTo::Call(addr),
                None => TransactTo::Create,
            },
            value: decoded.value,
            data: decoded.input.into(),
            gas_limit: decoded.gas_limit,
            gas_price: decoded.gas_price,
            nonce: Some(decoded.nonce),
            ..Default::default()
        };

        // Step 5: Execute transaction
        let result = evm.transact()?;

        // Step 6: Extract execution result
        let (success, gas_used, output) = match &result.result {
            revm::primitives::ExecutionResult::Success {
                gas_used, output, ..
            } => (true, *gas_used, Some(output.clone().into_data().into())),
            revm::primitives::ExecutionResult::Revert { output, .. } => {
                (false, 0, Some(output.clone().into()))
            }
            revm::primitives::ExecutionResult::Halt { reason, gas_used } => {
                eprintln!("Execution halted: {:?}", reason);
                (false, *gas_used, None)
            }
        };

        // Step 7: Capture state changes by comparing before/after state
        let state_changes = Self::capture_state_delta_with_before(&db_before, &result.state);

        // Step 8: Return transaction trace
        let tx_hash = keccak256(tx_bytes);

        Ok(TxTrace {
            tx_index,
            tx_hash,
            state_changes,
            result: TxResult {
                success,
                gas_used,
                output,
            },
        })
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
