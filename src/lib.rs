pub mod aot;
mod decode;
pub mod elfgen;
pub mod interpreter;
pub mod ir;
pub mod jit;
mod util;

pub use interpreter::{HostIO, Runner, VM};
pub use interpreter::init_from_elf;
