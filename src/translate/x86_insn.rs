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

/// x86-64 operand types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    /// Register operand
    Register(X86Register),

    /// Immediate (constant) operand - 64-bit signed
    Immediate(i64),

    /// Memory operand: [base + offset]
    Memory { base: X86Register, offset: i32 },

    /// Label operand (for jumps and calls)
    Label(String),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Register(r) => write!(f, "{}", r),
            Operand::Immediate(imm) => write!(f, "{}", imm),
            Operand::Memory { base, offset } => {
                if *offset == 0 {
                    write!(f, "[{}]", base)
                } else if *offset > 0 {
                    write!(f, "[{} + {}]", base, offset)
                } else {
                    write!(f, "[{} - {}]", base, -offset)
                }
            }
            Operand::Label(name) => write!(f, "{}", name),
        }
    }
}

/// x86-64 instruction types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum X86Instruction {
    // Data movement
    Mov { src: Operand, dst: Operand },
    Movsx { src: Operand, dst: Operand }, // Sign-extend move
    Movzx { src: Operand, dst: Operand }, // Zero-extend move
    Lea { src: Operand, dst: Operand },   // Load effective address

    // Arithmetic
    Add { src: Operand, dst: Operand },
    Sub { src: Operand, dst: Operand },
    Adc { src: Operand, dst: Operand },  // Add with carry
    Sbb { src: Operand, dst: Operand },  // Subtract with borrow
    Inc { dst: Operand },                // Increment by 1
    Dec { dst: Operand },                // Decrement by 1
    Neg { dst: Operand },                // Two's complement negation
    Imul { src: Operand, dst: Operand }, // Signed multiply
    Idiv { src: Operand },               // Signed divide
    Mul { src: Operand },                // Unsigned multiply
    Div { src: Operand },                // Unsigned divide
    Cwd,                                 // Sign extend AX to DX:AX
    Cdq,                                 // Sign extend EAX to EDX:EAX
    Cqo,                                 // Sign extend RAX to RDX:RAX

    // Bitwise operations
    And { src: Operand, dst: Operand },
    Or { src: Operand, dst: Operand },
    Xor { src: Operand, dst: Operand },
    Not { dst: Operand },
    Xchg { src: Operand, dst: Operand }, // Atomic exchange

    // Shifts
    Shl { src: Operand, dst: Operand }, // Logical left shift
    Shr { src: Operand, dst: Operand }, // Logical right shift
    Sar { src: Operand, dst: Operand }, // Arithmetic right shift

    // Comparison
    Cmp { src: Operand, dst: Operand },
    Test { src: Operand, dst: Operand },

    // Conditional move
    Cmove { src: Operand, dst: Operand }, // Conditional move if equal
    Cmovne { src: Operand, dst: Operand }, // Conditional move if not equal
    Cmovl { src: Operand, dst: Operand }, // Conditional move if less
    Cmovle { src: Operand, dst: Operand }, // Conditional move if less or equal
    Cmovg { src: Operand, dst: Operand }, // Conditional move if greater
    Cmovge { src: Operand, dst: Operand }, // Conditional move if greater or equal

    // Set on condition
    Sete { dst: Operand },  // Set if equal
    Setne { dst: Operand }, // Set if not equal
    Setl { dst: Operand },  // Set if less
    Setle { dst: Operand }, // Set if less or equal
    Setg { dst: Operand },  // Set if greater
    Setge { dst: Operand }, // Set if greater or equal

    // Atomic operations
    Xadd { src: Operand, dst: Operand },    // Exchange and add
    Cmpxchg { src: Operand, dst: Operand }, // Compare and exchange

    // Control flow
    Jmp { target: String }, // Unconditional jump
    Je { target: String },  // Jump if equal
    Jne { target: String }, // Jump if not equal
    Jl { target: String },  // Jump if less
    Jle { target: String }, // Jump if less or equal
    Jg { target: String },  // Jump if greater
    Jge { target: String }, // Jump if greater or equal
    Jbe { target: String }, // Jump if below or equal (unsigned)
    Ja { target: String },  // Jump if above (unsigned)

    // Function calls
    Call { target: String }, // Call function
    Ret,                     // Return from function

    // Stack operations
    Push { src: Operand },
    Pop { dst: Operand },

    // Labels (pseudo-instruction)
    Label { name: String },

    // No-op
    Nop,
}

impl fmt::Display for X86Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            X86Instruction::Mov { src, dst } => write!(f, "mov {}, {}", dst, src),
            X86Instruction::Movsx { src, dst } => write!(f, "movsx {}, {}", dst, src),
            X86Instruction::Movzx { src, dst } => write!(f, "movzx {}, {}", dst, src),
            X86Instruction::Lea { src, dst } => write!(f, "lea {}, {}", dst, src),
            X86Instruction::Add { src, dst } => write!(f, "add {}, {}", dst, src),
            X86Instruction::Sub { src, dst } => write!(f, "sub {}, {}", dst, src),
            X86Instruction::Adc { src, dst } => write!(f, "adc {}, {}", dst, src),
            X86Instruction::Sbb { src, dst } => write!(f, "sbb {}, {}", dst, src),
            X86Instruction::Inc { dst } => write!(f, "inc {}", dst),
            X86Instruction::Dec { dst } => write!(f, "dec {}", dst),
            X86Instruction::Neg { dst } => write!(f, "neg {}", dst),
            X86Instruction::Imul { src, dst } => write!(f, "imul {}, {}", dst, src),
            X86Instruction::Idiv { src } => write!(f, "idiv {}", src),
            X86Instruction::Mul { src } => write!(f, "mul {}", src),
            X86Instruction::Div { src } => write!(f, "div {}", src),
            X86Instruction::Cwd => write!(f, "cwd"),
            X86Instruction::Cdq => write!(f, "cdq"),
            X86Instruction::Cqo => write!(f, "cqo"),
            X86Instruction::And { src, dst } => write!(f, "and {}, {}", dst, src),
            X86Instruction::Or { src, dst } => write!(f, "or {}, {}", dst, src),
            X86Instruction::Xor { src, dst } => write!(f, "xor {}, {}", dst, src),
            X86Instruction::Not { dst } => write!(f, "not {}", dst),
            X86Instruction::Xchg { src, dst } => write!(f, "xchg {}, {}", dst, src),
            X86Instruction::Shl { src, dst } => write!(f, "shl {}, {}", dst, src),
            X86Instruction::Shr { src, dst } => write!(f, "shr {}, {}", dst, src),
            X86Instruction::Sar { src, dst } => write!(f, "sar {}, {}", dst, src),
            X86Instruction::Cmp { src, dst } => write!(f, "cmp {}, {}", dst, src),
            X86Instruction::Test { src, dst } => write!(f, "test {}, {}", dst, src),
            X86Instruction::Cmove { src, dst } => write!(f, "cmove {}, {}", dst, src),
            X86Instruction::Cmovne { src, dst } => write!(f, "cmovne {}, {}", dst, src),
            X86Instruction::Cmovl { src, dst } => write!(f, "cmovl {}, {}", dst, src),
            X86Instruction::Cmovle { src, dst } => write!(f, "cmovle {}, {}", dst, src),
            X86Instruction::Cmovg { src, dst } => write!(f, "cmovg {}, {}", dst, src),
            X86Instruction::Cmovge { src, dst } => write!(f, "cmovge {}, {}", dst, src),
            X86Instruction::Sete { dst } => write!(f, "sete {}", dst),
            X86Instruction::Setne { dst } => write!(f, "setne {}", dst),
            X86Instruction::Setl { dst } => write!(f, "setl {}", dst),
            X86Instruction::Setle { dst } => write!(f, "setle {}", dst),
            X86Instruction::Setg { dst } => write!(f, "setg {}", dst),
            X86Instruction::Setge { dst } => write!(f, "setge {}", dst),
            X86Instruction::Xadd { src, dst } => write!(f, "xadd {}, {}", dst, src),
            X86Instruction::Cmpxchg { src, dst } => write!(f, "cmpxchg {}, {}", dst, src),
            X86Instruction::Jmp { target } => write!(f, "jmp {}", target),
            X86Instruction::Je { target } => write!(f, "je {}", target),
            X86Instruction::Jne { target } => write!(f, "jne {}", target),
            X86Instruction::Jl { target } => write!(f, "jl {}", target),
            X86Instruction::Jle { target } => write!(f, "jle {}", target),
            X86Instruction::Jg { target } => write!(f, "jg {}", target),
            X86Instruction::Jge { target } => write!(f, "jge {}", target),
            X86Instruction::Jbe { target } => write!(f, "jbe {}", target),
            X86Instruction::Ja { target } => write!(f, "ja {}", target),
            X86Instruction::Call { target } => write!(f, "call {}", target),
            X86Instruction::Ret => write!(f, "ret"),
            X86Instruction::Push { src } => write!(f, "push {}", src),
            X86Instruction::Pop { dst } => write!(f, "pop {}", dst),
            X86Instruction::Label { name } => write!(f, "{}:", name),
            X86Instruction::Nop => write!(f, "nop"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_codes() {
        assert_eq!(X86Register::RAX.code(), 0);
        assert_eq!(X86Register::RCX.code(), 1);
        assert_eq!(X86Register::R8.code(), 8);
        assert_eq!(X86Register::R15.code(), 15);
    }

    #[test]
    fn test_register_needs_rex() {
        assert!(!X86Register::RAX.needs_rex());
        assert!(!X86Register::RBX.needs_rex());
        assert!(X86Register::R8.needs_rex());
        assert!(X86Register::R15.needs_rex());
    }

    #[test]
    fn test_register_display() {
        assert_eq!(X86Register::RAX.to_string(), "rax");
        assert_eq!(X86Register::R12.to_string(), "r12");
    }

    #[test]
    fn test_operand_display() {
        let reg = Operand::Register(X86Register::RAX);
        assert_eq!(reg.to_string(), "rax");

        let imm = Operand::Immediate(42);
        assert_eq!(imm.to_string(), "42");

        let mem = Operand::Memory {
            base: X86Register::RBP,
            offset: -8,
        };
        assert_eq!(mem.to_string(), "[rbp - 8]");
    }

    #[test]
    fn test_instruction_display() {
        let mov = X86Instruction::Mov {
            src: Operand::Immediate(5),
            dst: Operand::Register(X86Register::RAX),
        };
        assert_eq!(mov.to_string(), "mov rax, 5");

        let add = X86Instruction::Add {
            src: Operand::Register(X86Register::RBX),
            dst: Operand::Register(X86Register::RAX),
        };
        assert_eq!(add.to_string(), "add rax, rbx");

        let jmp = X86Instruction::Jmp {
            target: "loop_start".to_string(),
        };
        assert_eq!(jmp.to_string(), "jmp loop_start");
    }

    #[test]
    fn test_instruction_equality() {
        let mov1 = X86Instruction::Mov {
            src: Operand::Immediate(5),
            dst: Operand::Register(X86Register::RAX),
        };
        let mov2 = X86Instruction::Mov {
            src: Operand::Immediate(5),
            dst: Operand::Register(X86Register::RAX),
        };
        assert_eq!(mov1, mov2);
    }
}
