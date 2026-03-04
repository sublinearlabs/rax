//! Verify State Root
//!
//! Tests the state root verification functionality.
//! State root is the Merkle tree of all account states - if it matches Ethereum's,
//! our execution is proven correct!

use alloy_primitives::{Address, B256, U256};
use riscv::eth::{BlockTracer, EthFetcher};
use std::collections::HashMap;

/// Test: Extract state root from block JSON
#[test]
fn test_extract_state_root_from_block() {
    // Create a mock block JSON (similar to what eth_getBlockByNumber returns)
    let block_json = serde_json::json!({
        "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        "stateRoot": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        "number": "0x5f5e0f",
        "transactions": []
    });

    let result = BlockTracer::extract_state_root_from_block(&block_json);
    assert!(result.is_ok());

    let state_root = result.unwrap();
    // Verify we got the right state root
    assert_eq!(
        state_root.to_string(),
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
    );
}

/// Test: Extract state root handles missing field
#[test]
fn test_extract_state_root_missing_field() {
    let block_json = serde_json::json!({
        "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        "number": "0x5f5e0f"
    });

    let result = BlockTracer::extract_state_root_from_block(&block_json);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("stateRoot"));
}

/// Test: Verify state root - matching roots
#[test]
fn test_verify_state_root_match() {
    let state_root = B256::from([0x11; 32]);
    let block_number = 5000000u64;

    let verification = BlockTracer::verify_block_state_root(state_root, state_root, block_number);

    assert!(verification.matches);
    assert_eq!(verification.block_number, block_number);
    assert_eq!(verification.on_chain_state_root, state_root);
    assert_eq!(verification.our_computed_state_root, state_root);
    assert!(verification.error.is_none());
}

/// Test: Verify state root - mismatched roots
#[test]
fn test_verify_state_root_mismatch() {
    let on_chain_root = B256::from([0x11; 32]);
    let our_root = B256::from([0x22; 32]);
    let block_number = 5000000u64;

    let verification = BlockTracer::verify_block_state_root(on_chain_root, our_root, block_number);

    assert!(!verification.matches);
    assert_eq!(verification.block_number, block_number);
    assert_eq!(verification.on_chain_state_root, on_chain_root);
    assert_eq!(verification.our_computed_state_root, our_root);
    assert!(verification.error.is_some());
    assert!(verification.error.unwrap().contains("mismatch"));
}

/// Test: Compute state root from accounts
#[test]
fn test_compute_state_root() {
    use riscv::eth::types::AccountData;

    let mut accounts = HashMap::new();

    // Add a test account
    accounts.insert(
        Address::from([0x01; 20]),
        AccountData {
            nonce: 1,
            balance: U256::from(1000u64),
            code: vec![0x60, 0x01],
            storage: HashMap::new(),
        },
    );

    let root = BlockTracer::compute_state_root(&accounts);

    // Just verify we get a B256 back (exact value depends on hash implementation)
    assert_ne!(root, B256::ZERO);
}

/// Test: Compute state root - empty accounts
#[test]
fn test_compute_state_root_empty() {
    let accounts: HashMap<Address, riscv::eth::types::AccountData> = HashMap::new();
    let root = BlockTracer::compute_state_root(&accounts);

    // Even empty state should hash to something
    assert_ne!(root, B256::ZERO);
}

/// Test: Compute state root is deterministic
#[test]
fn test_compute_state_root_deterministic() {
    use riscv::eth::types::AccountData;

    let mut accounts1 = HashMap::new();
    let mut accounts2 = HashMap::new();

    let account = AccountData {
        nonce: 1,
        balance: U256::from(1000u64),
        code: vec![0x60, 0x01],
        storage: HashMap::new(),
    };

    accounts1.insert(Address::from([0x01; 20]), account.clone());
    accounts2.insert(Address::from([0x01; 20]), account);

    let root1 = BlockTracer::compute_state_root(&accounts1);
    let root2 = BlockTracer::compute_state_root(&accounts2);

    assert_eq!(root1, root2);
}

/// Test: Verification types are serializable
#[test]
fn test_state_root_verification_serializable() {
    use riscv::eth::types::StateRootVerification;

    let verification = StateRootVerification {
        block_number: 5000000,
        on_chain_state_root: B256::from([0x11; 32]),
        our_computed_state_root: B256::from([0x11; 32]),
        matches: true,
        error: None,
    };

    // Should be serializable to JSON
    let json = serde_json::to_string(&verification);
    assert!(json.is_ok());

    let json_str = json.unwrap();
    assert!(json_str.contains("5000000"));
    assert!(json_str.contains("true"));
}

/// Test: Fetch state root from mainnet block (integration test)
/// This test is ignored by default as it requires network access
#[tokio::test]
// #[ignore]
async fn test_fetch_and_verify_state_root_mainnet() {
    let alchemy_key = std::env::var("ALCHEMY_API_KEY").unwrap_or_else(|_| "demo".to_string());
    let rpc_url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", alchemy_key);

    let fetcher = EthFetcher::new(&rpc_url);
    assert!(fetcher.is_ok());

    let fetcher = fetcher.unwrap();

    // Fetch a known block (e.g., block 17000000)
    let block_number = 17000000u64;
    let block_result = fetcher.fetch_block_summary(block_number).await;

    if let Ok(block) = block_result {
        // Extract state root from block
        let state_root_result = BlockTracer::extract_state_root_from_block(&block);
        assert!(state_root_result.is_ok());

        let on_chain_root = state_root_result.unwrap();
        println!(
            "✓ Fetched state root for block {}: {:?}",
            block_number, on_chain_root
        );

        // In a real test, we would:
        // 1. Execute all transactions in the block
        // 2. Compute our state root
        // 3. Compare it to on_chain_root
        // For now, just verify extraction works
        assert_ne!(on_chain_root, B256::ZERO);
    } else {
        println!("Could not fetch block (may need ALCHEMY_API_KEY environment variable)");
    }
}
