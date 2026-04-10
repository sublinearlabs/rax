mod aot;
mod decode;
mod ecall;
mod elf;
pub mod elf_gen;
mod execute;
mod host_io;
mod instr_execute;
pub mod ir;
pub mod jit;
mod loader;
mod memory;
mod runner;
pub mod trace;
mod util;
mod vm;

pub use host_io::HostIO;
pub use loader::{init_from_elf, init_from_elf_with_tracer};
pub use runner::Runner;
pub use vm::VM;

pub mod cli;
