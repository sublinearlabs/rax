use alloy_primitives::{keccak256, Address, B256, U256};
use anyhow::Result;
use std::collections::HashMap;

use alloy_rlp::Encodable;
use revm::primitives::{AccountInfo, Bytecode, TxEnv};
use revm::Evm;
use revm::InMemoryDB;

use super::types::{AccountData, BlockData, BlockTrace, StateChange, TxResult, TxTrace};
use super::utils::{get_chain_config, parse_hex_u256, parse_hex_u64, EMPTY_CODE_HASH};

/// Generates execution traces for Ethereum blocks
pub struct BlockTracer;

/// Transaction data parsed from JSON RPC response
/// Extracts basic transaction information from JSON transaction objects
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
    /// Parse a transaction from JSON object (from eth_getBlockByNumber)
    fn from_json(tx: &serde_json::Value) -> Result<Self> {
        use std::str::FromStr;

        // Extract 'from' address
        let from_str = tx["from"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'from' in transaction"))?;
        let from = Address::from_str(from_str)
            .map_err(|e| anyhow::anyhow!("Invalid 'from' address: {}", e))?;

        // Extract optional 'to' address
        let to = if let Some(to_str) = tx["to"].as_str() {
            if to_str.is_empty() || to_str == "0x" {
                None
            } else {
                Some(
                    Address::from_str(to_str)
                        .map_err(|e| anyhow::anyhow!("Invalid 'to' address: {}", e))?,
                )
            }
        } else {
            None
        };

        // Extract nonce
        let nonce_str = tx["nonce"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'nonce' in transaction"))?;
        let nonce = u64::from_str_radix(nonce_str.strip_prefix("0x").unwrap_or(nonce_str), 16)
            .map_err(|e| anyhow::anyhow!("Invalid 'nonce': {}", e))?;

        // Extract value
        let value_str = tx["value"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'value' in transaction"))?;
        let value = U256::from_str_radix(value_str.strip_prefix("0x").unwrap_or(value_str), 16)
            .map_err(|e| anyhow::anyhow!("Invalid 'value': {}", e))?;

        // Extract gas limit
        let gas_str = tx["gas"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'gas' in transaction"))?;
        let gas_limit = u64::from_str_radix(gas_str.strip_prefix("0x").unwrap_or(gas_str), 16)
            .map_err(|e| anyhow::anyhow!("Invalid 'gas': {}", e))?;

        // Extract gas price - use gasPrice for legacy, maxFeePerGas for EIP-1559
        let gas_price = if let Some(max_fee_str) = tx["maxFeePerGas"].as_str() {
            U256::from_str_radix(max_fee_str.strip_prefix("0x").unwrap_or(max_fee_str), 16)
                .map_err(|e| anyhow::anyhow!("Invalid 'maxFeePerGas': {}", e))?
        } else if let Some(gas_price_str) = tx["gasPrice"].as_str() {
            U256::from_str_radix(
                gas_price_str.strip_prefix("0x").unwrap_or(gas_price_str),
                16,
            )
            .map_err(|e| anyhow::anyhow!("Invalid 'gasPrice': {}", e))?
        } else {
            anyhow::bail!("Missing both 'gasPrice' and 'maxFeePerGas' in transaction")
        };

        // Extract input data
        let input_str = tx["input"]
            .as_str()
            .or_else(|| tx["data"].as_str())
            .unwrap_or("0x");
        let input =
            hex::decode(input_str.strip_prefix("0x").unwrap_or(input_str)).unwrap_or_default();

        // Extract optional chain ID
        let chain_id = tx["chainId"]
            .as_str()
            .and_then(|s| u64::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).ok());

        Ok(Self {
            from,
            to,
            nonce,
            value,
            gas_limit,
            gas_price,
            input,
            chain_id,
        })
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
            // Compute code hash for the account
            let code_hash = if account_data.code.is_empty() {
                EMPTY_CODE_HASH
            } else {
                keccak256(&account_data.code)
            };

            initial_state.push((
                *addr,
                super::types::AccountState {
                    nonce: account_data.nonce,
                    balance: account_data.balance,
                    code_hash,
                },
            ));
        }

        // Step 2: Execute each transaction
        for (tx_index, tx_json) in block_data.transactions.iter().enumerate() {
            // Attempt to execute transaction
            match Self::execute_transaction(&mut state_db, tx_json, tx_index) {
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
                EMPTY_CODE_HASH
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
        tx_json: &serde_json::Value,
        tx_index: usize,
    ) -> Result<TxTrace> {
        // Step 1: Decode transaction from JSON
        let decoded = match DecodedTransaction::from_json(tx_json) {
            Ok(tx) => tx,
            Err(e) => {
                eprintln!("  TX {}: Failed to decode JSON - {}", tx_index, e);
                return Err(e);
            }
        };

        eprintln!(
            "  TX {}: from={}, to={:?}, nonce={}, gas_limit={}, data_len={}",
            tx_index,
            decoded.from,
            decoded.to,
            decoded.nonce,
            decoded.gas_limit,
            decoded.input.len()
        );

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
        let result = match evm.transact() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("  TX {}: Execution error: {}", tx_index, e);
                return Err(e.into());
            }
        };

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
        // Extract transaction hash from JSON if available, otherwise compute from content
        let tx_hash = if let Some(hash_str) = tx_json.get("hash").and_then(|h| h.as_str()) {
            use std::str::FromStr;
            B256::from_str(hash_str).unwrap_or_default()
        } else {
            // Fallback: compute deterministic hash
            let content = format!("{:?}", tx_json);
            keccak256(content.as_bytes())
        };

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

    /// Generate execution witness data compatible with alloy-rpc-types-debug::ExecutionWitness
    ///
    /// Creates a structured witness containing:
    /// - Full block header and body
    /// - Chain configuration
    /// - State witness (codes, keys, state proofs)
    ///
    /// This matches the structure expected by exec-block and can be serialized to JSON
    pub fn generate_witness(
        block_data: &BlockData,
        block_trace: &BlockTrace,
    ) -> Result<serde_json::Value> {
        // 1. Build block header from actual block data
        let block_json = &block_data.raw_block;
        let header = serde_json::json!({
            "number": block_data.block_number,
            "state_root": block_json["stateRoot"].as_str().unwrap_or("0x0"),
            "parent_hash": block_json["parentHash"].as_str().unwrap_or("0x0000000000000000000000000000000000000000000000000000000000000000"),
            "timestamp": parse_hex_u64(block_json["timestamp"].as_str().unwrap_or("0x0")),
            "gas_used": parse_hex_u64(block_json["gasUsed"].as_str().unwrap_or("0x0")),
            "gas_limit": parse_hex_u64(block_json["gasLimit"].as_str().unwrap_or("0x0")),
            "beneficiary": block_json["miner"].as_str().unwrap_or("0x0000000000000000000000000000000000000000"),
            "difficulty": parse_hex_u256(block_json["difficulty"].as_str().unwrap_or("0x0")),
            "mix_hash": block_json["mixHash"].as_str().unwrap_or("0x0000000000000000000000000000000000000000000000000000000000000000"),
            "nonce": block_json["nonce"].as_str().unwrap_or("0x0000000000000000"),
            "ommers_hash": block_json["sha3Uncles"].as_str().unwrap_or("0x1dcc4de8dec75d7aab85b567b6ccd41ad312451ca908e6d1e0a0d94c51ad5c3d"),
            "transactions_root": block_json["transactionsRoot"].as_str().unwrap_or("0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"),
            "receipts_root": block_json["receiptsRoot"].as_str().unwrap_or("0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"),
            "logs_bloom": block_json["logsBloom"].as_str().unwrap_or(&format!("0x{}", "0".repeat(512))),
            "extra_data": block_json["extraData"].as_str().unwrap_or("0x"),
            "base_fee_per_gas": parse_hex_u64(block_json["baseFeePerGas"].as_str().unwrap_or("0x1")),
        });

        // 2. Build transactions array from actual transaction data
        let mut transactions = Vec::new();

        for (tx_index, tx_json) in block_data.transactions.iter().enumerate() {
            // Extract actual transaction data from JSON
            let nonce = parse_hex_u64(tx_json["nonce"].as_str().unwrap_or("0x0"));
            let gas_price = tx_json["gasPrice"]
                .as_str()
                .or_else(|| tx_json["maxFeePerGas"].as_str())
                .unwrap_or("0x1");
            let gas_limit = parse_hex_u64(tx_json["gas"].as_str().unwrap_or("0x5208"));
            let to_addr = tx_json["to"]
                .as_str()
                .unwrap_or("0x0000000000000000000000000000000000000000");
            let value = tx_json["value"].as_str().unwrap_or("0x0");

            // Extract input data from transaction JSON
            let input_hex = if let Some(input_val) = tx_json.get("input") {
                if let Some(input_str) = input_val.as_str() {
                    input_str.to_string()
                } else if input_val.is_object() || input_val.is_array() {
                    // If it's an object/array, serialize it to hex
                    let serialized =
                        serde_json::to_string(input_val).unwrap_or_else(|_| "{}".to_string());
                    format!("0x{}", hex::encode(serialized.as_bytes()))
                } else {
                    "0x".to_string()
                }
            } else if let Some(data_val) = tx_json.get("data") {
                if let Some(data_str) = data_val.as_str() {
                    data_str.to_string()
                } else {
                    "0x".to_string()
                }
            } else {
                "0x".to_string()
            };

            // Extract signature data from transaction
            let v = tx_json["v"].as_str().unwrap_or("0x1");
            let r = tx_json["r"].as_str().unwrap_or("0x0");
            let s = tx_json["s"].as_str().unwrap_or("0x0");

            // Extract max priority fee for EIP-1559
            let max_priority_fee = tx_json["maxPriorityFeePerGas"]
                .as_str()
                .and_then(|s| u64::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).ok())
                .unwrap_or(0);

            // Parse gas_price as integer
            let gas_price_int =
                u64::from_str_radix(gas_price.strip_prefix("0x").unwrap_or(&gas_price), 16)
                    .unwrap_or(0);

            transactions.push(serde_json::json!({
                "signature": {
                    "r": r,
                    "s": s,
                    "v": v,
                    "yParity": if v == "0x0" || v == "0x1c" { "0x0" } else { "0x1" },
                },
                "transaction": {
                    "Eip1559": {
                        "access_list": [],
                        "chain_id": block_data.raw_block["chainId"]
                            .as_str()
                            .and_then(|s| u64::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).ok())
                            .unwrap_or(1),
                        "nonce": nonce,
                        "gas_limit": gas_limit,
                        "max_fee_per_gas": gas_price_int,
                        "max_priority_fee_per_gas": max_priority_fee,
                        "to": to_addr,
                        "value": value,
                        "input": input_hex,
                    }
                }
            }));
        }

        // 3. Build block body
        let body = serde_json::json!({
            "transactions": transactions,
            "ommers": [],
        });

        // 4. Build chain config (Ethereum mainnet)
        let chain_config = get_chain_config();

        // 5. Extract bytecodes from all accounts
        let codes = extract_bytecodes(&block_data.accounts);

        // 6. Extract storage keys from all accounts
        let keys = extract_storage_keys(&block_data.accounts);

        // 7. Build state proofs from block trace using alloy-trie
        let state_proofs = Self::build_state_proofs_from_trace(block_trace, &block_data.accounts)?;

        // 8. Assemble complete ExecutionWitness structure
        let witness_json = serde_json::json!({
            "block": {
                "header": header,
                "body": body,
            },
            "chain_config": chain_config,
            "witness": {
                "codes": codes,
                "keys": keys,
                "state": state_proofs,
                "headers": [],
            }
        });

        Ok(witness_json)
    }

    /// Build state proofs from block trace
    ///
    /// Constructs RLP-encoded state entries from the state changes during block execution.
    /// This provides the witness data needed for verification.
    fn build_state_proofs_from_trace(
        block_trace: &BlockTrace,
        accounts: &HashMap<Address, AccountData>,
    ) -> Result<Vec<serde_json::Value>> {
        let mut state_proofs = Vec::new();

        // Collect all addresses that had state changes
        let mut changed_addresses = std::collections::HashSet::new();
        for tx_trace in &block_trace.transactions {
            for state_change in &tx_trace.state_changes {
                changed_addresses.insert(state_change.address);
            }
        }

        // For each address that changed, encode its account state
        for address in changed_addresses {
            if let Some(account_data) = accounts.get(&address) {
                // Compute the code hash
                let code_hash = if account_data.code.is_empty() {
                    EMPTY_CODE_HASH
                } else {
                    keccak256(&account_data.code)
                };

                // Encode account: (nonce, balance, code_hash)
                let mut account_bytes = Vec::new();
                account_data.nonce.encode(&mut account_bytes);
                account_data.balance.encode(&mut account_bytes);
                code_hash.encode(&mut account_bytes);

                // Store as hex string
                state_proofs.push(serde_json::Value::String(format!(
                    "0x{}",
                    hex::encode(&account_bytes)
                )));
            }
        }

        // Also add storage slot changes as proof entries
        for tx_trace in &block_trace.transactions {
            for state_change in &tx_trace.state_changes {
                for (slot, _old_value, new_value) in &state_change.storage_changes {
                    // Encode storage slot entry: (slot, value)
                    let mut slot_bytes = Vec::new();
                    slot.encode(&mut slot_bytes);
                    new_value.encode(&mut slot_bytes);

                    state_proofs.push(serde_json::Value::String(format!(
                        "0x{}",
                        hex::encode(&slot_bytes)
                    )));
                }
            }
        }

        Ok(state_proofs)
    }
}

/// Helper: Extract all bytecodes from accounts
fn extract_bytecodes(accounts: &HashMap<Address, super::types::AccountData>) -> Vec<String> {
    accounts
        .values()
        .filter(|acc| !acc.code.is_empty())
        .map(|acc| format!("0x{}", hex::encode(&acc.code)))
        .collect()
}

/// Helper: Extract all storage keys from accounts
fn extract_storage_keys(accounts: &HashMap<Address, super::types::AccountData>) -> Vec<String> {
    let mut keys = Vec::new();
    for account in accounts.values() {
        for key in account.storage.keys() {
            keys.push(format!("0x{:064x}", key));
        }
    }
    keys.sort();
    keys.dedup();
    keys
}
