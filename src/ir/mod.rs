mod builder;
mod interpreter;
mod ir;
pub mod lower;

pub use builder::IrBuilder;
pub use interpreter::execute_ir;
pub use ir::*;
