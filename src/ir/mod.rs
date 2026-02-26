// Builder API (construction + validation)
mod builder;
// Interpreter execution
mod interpreter;
// Lowering from decoded instructions
pub mod lower;
// CFG structures (Op/Terminator/Block/IrFunction)
mod cfg;
// Display impl + IR formatting tests
mod display;
// Pure/effect op enums + widths/sign
mod ops;
// Base types and IDs
mod types;

pub use builder::IrBuilder;
pub use cfg::*;
pub use interpreter::execute_ir;
pub use ops::*;
pub use types::*;
