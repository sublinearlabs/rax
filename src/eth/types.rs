use alloy_primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete trace of a block execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTrace {
    pub block_number: u64,
    pub block_hash: B256,
    pub state_root: B256,
    pub transactions: Vec<TxTrace>,
    pub initial_state: Vec<(Address, AccountState)>,
}

/// Trace of a single transaction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxTrace {
    pub tx_index: usize,
    pub tx_hash: B256,
    pub state_changes: Vec<StateChange>,
    pub result: TxResult,
}

/// State changes caused by a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub address: Address,
    pub nonce_change: Option<(u64, u64)>,     // (before, after)
    pub balance_change: Option<(U256, U256)>, // (before, after)
    pub storage_changes: Vec<(U256, U256, U256)>, // (key, old_value, new_value)
}

/// Account state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    pub nonce: u64,
    pub balance: U256,
    pub code_hash: B256,
}

/// Result of transaction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxResult {
    pub success: bool,
    pub gas_used: u64,
    pub output: Option<Vec<u8>>,
}

/// Block data fetched from Ethereum RPC
#[derive(Debug, Clone)]
pub struct BlockData {
    pub block_number: u64,
    pub block_hash: B256,
    pub state_root: B256,
    pub transactions: Vec<Vec<u8>>, // RLP encoded
    pub accounts: HashMap<Address, AccountData>,
}

/// Account data at block start
#[derive(Debug, Clone)]
pub struct AccountData {
    pub nonce: u64,
    pub balance: U256,
    pub code: Vec<u8>,
    pub storage: HashMap<U256, U256>,
}

/// Result of verifying a single transaction against its receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxVerificationResult {
    pub tx_index: usize,
    pub tx_hash: B256,
    /// Did execution status match receipt status?
    pub status_match: bool,
    /// Did gas used match receipt gas used?
    pub gas_match: bool,
    /// Details of any mismatches
    pub details: VerificationDetails,
}

/// Detailed verification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationDetails {
    pub our_status: bool,
    pub receipt_status: bool,
    pub our_gas_used: u64,
    pub receipt_gas_used: u64,
    pub mismatch_reason: Option<String>,
}

/// Result of verifying block state root
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRootVerification {
    pub block_number: u64,
    /// State root from on-chain block header
    pub on_chain_state_root: B256,
    /// State root computed from our execution
    pub our_computed_state_root: B256,
    /// Do they match?
    pub matches: bool,
    /// Error details if verification failed
    pub error: Option<String>,
}
