//! Verify Transaction Execution
//!
//! Tests the verification of transaction execution against on-chain receipts.
//! Ensures our execution results match what the network recorded.

use alloy_primitives::B256;
use riscv::eth::{
    BlockTracer,
    types::{TxResult, TxTrace},
};
use serde_json::json;

/// Test: Verify a single transaction execution
#[test]
fn test_verify_tx_execution_matching() {
    // Create a mock transaction trace from our execution
    let tx_trace = TxTrace {
        tx_index: 0,
        tx_hash: B256::from_slice(&[1; 32]),
        state_changes: vec![],
        result: TxResult {
            success: true,
            gas_used: 21000,
            output: None,
        },
    };

    // Create a mock receipt (what was on-chain)
    let receipt = json!({
        "transactionHash": "0x0101010101010101010101010101010101010101010101010101010101010101",
        "status": "0x1",          // Success (1 = success, 0 = failure)
        "gasUsed": "0x5208",      // 21000 in hex
        "logs": [],
        "blockNumber": "0x123456",
    });

    // Verify: should match
    let result = BlockTracer::verify_tx_execution(&tx_trace, &receipt, 0).unwrap();

    assert_eq!(result.tx_index, 0);
    assert!(result.status_match, "Status should match");
    assert!(result.gas_match, "Gas should match");
    assert!(
        result.details.mismatch_reason.is_none(),
        "Should have no mismatches"
    );

    println!("✓ Matching transaction verified successfully");
}

/// Test: Detect status mismatch
#[test]
fn test_verify_tx_execution_status_mismatch() {
    // Our execution says success
    let tx_trace = TxTrace {
        tx_index: 0,
        tx_hash: B256::from_slice(&[2; 32]),
        state_changes: vec![],
        result: TxResult {
            success: true, // We think it succeeded
            gas_used: 21000,
            output: None,
        },
    };

    // But receipt says it failed
    let receipt = json!({
        "status": "0x0",    // Failed on-chain!
        "gasUsed": "0x5208",
    });

    let result = BlockTracer::verify_tx_execution(&tx_trace, &receipt, 0).unwrap();

    assert!(!result.status_match, "Status should NOT match");
    assert!(result.gas_match, "Gas still matches");
    assert!(
        result.details.mismatch_reason.is_some(),
        "Should have mismatch reason"
    );
    assert!(
        result
            .details
            .mismatch_reason
            .as_ref()
            .unwrap()
            .contains("Status"),
        "Reason should mention Status"
    );

    println!("✓ Status mismatch detected correctly");
}

/// Test: Detect gas mismatch
#[test]
fn test_verify_tx_execution_gas_mismatch() {
    // Our execution used different gas
    let tx_trace = TxTrace {
        tx_index: 1,
        tx_hash: B256::from_slice(&[3; 32]),
        state_changes: vec![],
        result: TxResult {
            success: true,
            gas_used: 25000, // We used more gas
            output: None,
        },
    };

    // Receipt says less gas was used
    let receipt = json!({
        "status": "0x1",
        "gasUsed": "0x5208",  // 21000 - we used 25000
    });

    let result = BlockTracer::verify_tx_execution(&tx_trace, &receipt, 1).unwrap();

    assert!(result.status_match, "Status should match");
    assert!(!result.gas_match, "Gas should NOT match");
    assert!(
        result.details.mismatch_reason.is_some(),
        "Should have mismatch reason"
    );
    assert!(
        result
            .details
            .mismatch_reason
            .as_ref()
            .unwrap()
            .contains("Gas"),
        "Reason should mention Gas"
    );

    println!("✓ Gas mismatch detected correctly");
}

/// Test: Verify multiple transactions
#[test]
fn test_verify_block_execution() {
    use riscv::eth::types::BlockTrace;

    // Create a trace with 3 transactions
    let block_trace = BlockTrace {
        block_number: 123456,
        block_hash: B256::from_slice(&[0xFF; 32]),
        state_root: B256::from_slice(&[0xAA; 32]),
        transactions: vec![
            TxTrace {
                tx_index: 0,
                tx_hash: B256::from_slice(&[1; 32]),
                state_changes: vec![],
                result: TxResult {
                    success: true,
                    gas_used: 21000,
                    output: None,
                },
            },
            TxTrace {
                tx_index: 1,
                tx_hash: B256::from_slice(&[2; 32]),
                state_changes: vec![],
                result: TxResult {
                    success: true,
                    gas_used: 50000,
                    output: None,
                },
            },
            TxTrace {
                tx_index: 2,
                tx_hash: B256::from_slice(&[3; 32]),
                state_changes: vec![],
                result: TxResult {
                    success: false,
                    gas_used: 21000,
                    output: None,
                },
            },
        ],
        initial_state: vec![],
    };

    // Create corresponding receipts
    let receipts = vec![
        json!({
            "transactionHash": "0x0101010101010101010101010101010101010101010101010101010101010101",
            "status": "0x1",
            "gasUsed": "0x5208",  // 21000
        }),
        json!({
            "transactionHash": "0x0202020202020202020202020202020202020202020202020202020202020202",
            "status": "0x1",
            "gasUsed": "0xc350",  // 50000
        }),
        json!({
            "transactionHash": "0x0303030303030303030303030303030303030303030303030303030303030303",
            "status": "0x0",      // Failed
            "gasUsed": "0x5208",
        }),
    ];

    let results = BlockTracer::verify_block_execution(&block_trace, &receipts).unwrap();

    assert_eq!(results.len(), 3, "Should have 3 verification results");

    // Check each result
    for result in &results {
        assert!(
            result.status_match,
            "Tx {} status should match",
            result.tx_index
        );
        assert!(result.gas_match, "Tx {} gas should match", result.tx_index);
        assert!(result.details.mismatch_reason.is_none());
    }

    println!("✓ All block transactions verified successfully");
}

/// Test: Reject mismatched transaction counts
#[test]
fn test_verify_block_execution_count_mismatch() {
    use riscv::eth::types::BlockTrace;

    let block_trace = BlockTrace {
        block_number: 123456,
        block_hash: B256::from_slice(&[0xFF; 32]),
        state_root: B256::from_slice(&[0xAA; 32]),
        transactions: vec![TxTrace {
            tx_index: 0,
            tx_hash: B256::from_slice(&[1; 32]),
            state_changes: vec![],
            result: TxResult {
                success: true,
                gas_used: 21000,
                output: None,
            },
        }],
        initial_state: vec![],
    };

    // Only provide one receipt but trace has one tx
    let receipts = vec![
        json!({"status": "0x1", "gasUsed": "0x5208"}),
        json!({"status": "0x1", "gasUsed": "0x5208"}), // Extra receipt!
    ];

    let error = BlockTracer::verify_block_execution(&block_trace, &receipts);
    assert!(error.is_err(), "Should error on count mismatch");
    assert!(
        error.unwrap_err().to_string().contains("mismatch"),
        "Error should mention mismatch"
    );

    println!("✓ Transaction count mismatch rejected correctly");
}
