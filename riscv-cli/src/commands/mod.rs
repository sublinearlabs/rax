//! RISC-V CLI commands implementation

pub mod compile;
pub mod run;

pub use compile::execute_compile;
pub use run::execute_run;
