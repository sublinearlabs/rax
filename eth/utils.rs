/// Utility functions and constants for Ethereum operations
use alloy_primitives::B256;

/// Keccak256 hash of an empty byte string
/// Used as the code hash for accounts with no bytecode
pub const EMPTY_CODE_HASH: B256 = B256::new([
    0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0,
    0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
]);

/// Get the chain configuration for Ethereum mainnet
pub fn get_chain_config() -> serde_json::Value {
    serde_json::json!({
        "chain_id": 1,
        "homestead_block": 1150000,
        "dao_fork_block": 1920000,
        "dao_fork_support": true,
        "eip150_block": 2463000,
        "eip155_block": 2675000,
        "eip158_block": 2675000,
        "byzantium_block": 4370000,
        "constantinople_block": 7280000,
        "petersburg_block": 7280000,
        "istanbul_block": 9069000,
        "berlin_block": 12965000,
        "london_block": 12965000,
        "merge_netsplit_block": 17034870,
        "shanghai_time": 1681338455,
        "cancun_time": 1710338135,
    })
}
