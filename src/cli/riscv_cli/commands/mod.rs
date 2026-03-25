//! RISC-V CLI commands implementation

pub mod run;
pub mod verify_block;

pub use run::execute_run;
pub use verify_block::execute_verify_block;
