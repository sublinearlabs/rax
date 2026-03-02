mod decode;
mod ecall;
mod elf;
mod execute;
mod host_io;
mod instr_execute;
pub mod ir;
pub mod jit;
mod loader;
mod memory;
mod runner;
mod util;
mod vm;

pub use host_io::HostIO;
pub use loader::init_from_elf;
pub use runner::Runner;
pub use vm::VM;
