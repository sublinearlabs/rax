//! x86-64 instruction definitions and operand types
//!
//! This module defines the x86-64 instruction set used for RISC-V translation.
//! It provides the types needed to represent x86-64 instructions, registers, and operands.

use std::fmt;

/// x86-64 general-purpose registers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum X86Register {
    RAX,
    RBX,
    RCX,
    RDX,
    RSI,
    RDI,
    RSP,
    RBP,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl X86Register {
    /// Get the register code (0-15) for encoding
    pub fn code(&self) -> u8 {
        match self {
            X86Register::RAX => 0,
            X86Register::RCX => 1,
            X86Register::RDX => 2,
            X86Register::RBX => 3,
            X86Register::RSP => 4,
            X86Register::RBP => 5,
            X86Register::RSI => 6,
            X86Register::RDI => 7,
            X86Register::R8 => 8,
            X86Register::R9 => 9,
            X86Register::R10 => 10,
            X86Register::R11 => 11,
            X86Register::R12 => 12,
            X86Register::R13 => 13,
            X86Register::R14 => 14,
            X86Register::R15 => 15,
        }
    }

    /// Check if this register requires a REX prefix (R8-R15)
    pub fn needs_rex(&self) -> bool {
        matches!(
            self,
            X86Register::R8
                | X86Register::R9
                | X86Register::R10
                | X86Register::R11
                | X86Register::R12
                | X86Register::R13
                | X86Register::R14
                | X86Register::R15
        )
    }
}

impl fmt::Display for X86Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                X86Register::RAX => "rax",
                X86Register::RBX => "rbx",
                X86Register::RCX => "rcx",
                X86Register::RDX => "rdx",
                X86Register::RSI => "rsi",
                X86Register::RDI => "rdi",
                X86Register::RSP => "rsp",
                X86Register::RBP => "rbp",
                X86Register::R8 => "r8",
                X86Register::R9 => "r9",
                X86Register::R10 => "r10",
                X86Register::R11 => "r11",
                X86Register::R12 => "r12",
                X86Register::R13 => "r13",
                X86Register::R14 => "r14",
                X86Register::R15 => "r15",
            }
        )
    }
}
