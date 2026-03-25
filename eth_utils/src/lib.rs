//! Ethereum library crate
//!
//! Provides Ethereum-specific functionality for block fetching and tracing

pub mod cli;
pub mod fetcher;
pub mod tracer;
pub mod types;
pub mod utils;

pub use fetcher::EthFetcher;
pub use tracer::BlockTracer;
pub use types::BlockTrace;
pub use utils::{get_chain_config, parse_hex_u256, parse_hex_u64, EMPTY_CODE_HASH};
