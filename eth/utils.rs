/// Utility functions and constants for Ethereum operations

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
