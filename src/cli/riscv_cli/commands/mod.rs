//! RISC-V CLI commands implementation

pub mod run;
pub mod trace;

pub use run::execute_run;
pub use trace::execute_trace;
