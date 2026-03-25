//! Ethereum CLI commands implementation

pub mod common;
pub mod fetch;
pub mod generate_witness;

pub use fetch::execute_fetch;
pub use generate_witness::execute_generate_witness;
