//! RISC-V to x86-64 AOT Compiler
//!
//! This module contains the complete AOT (Ahead-Of-Time) compilation infrastructure
//! for translating RISC-V binaries to x86-64 ELF executables.

pub mod register_config;
pub mod translator;
pub mod x86_emitter;
pub mod x86_insn;

// Re-export commonly used types
pub use register_config::{RegisterAllocationConfig, RegisterAllocationStrategy};
pub use translator::{RiscvToX86Translator, TranslationContext, TranslationPhase};
pub use x86_emitter::X86Emitter;
pub use x86_insn::{Operand, X86Instruction, X86Register};
