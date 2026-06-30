pub mod ecall;
mod elf;
mod execute;
pub mod host_io;
mod instr_execute;
mod loader;
mod memory;
mod runner;
mod vm;

pub use host_io::HostIO;
pub use loader::{init_from_elf, init_from_elf_with_tracer};
pub use runner::Runner;
pub use vm::VM;

// Re-exports for JIT/AOT that need pub(crate) items from the interpreter:
pub(crate) use vm::{
    VM_EXIT_CODE_OFFSET, VM_FCSR_OFFSET, VM_FREGS_OFFSET, VM_HALTED_OFFSET, VM_PC_OFFSET,
    VM_REGS_OFFSET, VM_RESERVATION_OFFSET,
};
pub(crate) use ecall::handle_ecall;
pub(crate) use memory::MemoryDefault;
