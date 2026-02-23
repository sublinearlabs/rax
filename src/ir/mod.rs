mod builder;
mod interpreter;
mod ir;

pub use builder::IrBuilder;
pub use interpreter::execute_ir;
pub use ir::*;
