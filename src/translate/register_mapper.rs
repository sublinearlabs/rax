//! Register mapping trait for flexible register allocation strategies
//!
//! This trait defines the interface for register allocation without being
//! tied to a specific implementation. It allows different strategies to be
//! plugged in without modifying calling code.

use crate::aot::register_mapping::{RegisterLocation, RegisterMapping, RiscvRegister};
use crate::translate::x86_insn::Operand;

/// Trait for register mapping and allocation strategies
pub trait RegisterMapper: Clone {
    /// Get the RegisterLocation for a given RISC-V register index (0-31)
    fn get_register_location(&self, risc_v_reg: u8) -> RegisterLocation;

    /// Convert a RegisterLocation to an x86-64 Operand
    /// This handles strategy-specific conversions (e.g., GPR-only vs mixed)
    fn location_to_operand(&self, location: RegisterLocation) -> Result<Operand, String>;

    /// Create a new instance with default configuration for this strategy
    /// Takes the memory address where other registers would be stored in memory
    fn new(mem_addr: u64) -> Self
    where
        Self: Sized;
}

impl RegisterMapper for RegisterMapping {
    fn get_register_location(&self, risc_v_reg: u8) -> RegisterLocation {
        let reg = RiscvRegister::new(risc_v_reg);
        self[reg]
    }

    fn location_to_operand(&self, location: RegisterLocation) -> Result<Operand, String> {
        match location {
            RegisterLocation::ZERO => {
                // Zero register maps to 0 immediate value
                Ok(Operand::Immediate(0))
            }
            RegisterLocation::GPR(gpr_index) => {
                // Convert GPR index (0-15) to x86 register
                let x86_reg = match gpr_index {
                    0 => crate::translate::x86_insn::X86Register::RAX,
                    1 => crate::translate::x86_insn::X86Register::RCX,
                    2 => crate::translate::x86_insn::X86Register::RDX,
                    3 => crate::translate::x86_insn::X86Register::RBX,
                    4 => crate::translate::x86_insn::X86Register::RSP,
                    5 => crate::translate::x86_insn::X86Register::RBP,
                    6 => crate::translate::x86_insn::X86Register::RSI,
                    7 => crate::translate::x86_insn::X86Register::RDI,
                    8 => crate::translate::x86_insn::X86Register::R8,
                    9 => crate::translate::x86_insn::X86Register::R9,
                    10 => crate::translate::x86_insn::X86Register::R10,
                    11 => crate::translate::x86_insn::X86Register::R11,
                    12 => crate::translate::x86_insn::X86Register::R12,
                    13 => crate::translate::x86_insn::X86Register::R13,
                    14 => crate::translate::x86_insn::X86Register::R14,
                    15 => crate::translate::x86_insn::X86Register::R15,
                    _ => return Err(format!("Invalid GPR index: {}", gpr_index)),
                };
                Ok(Operand::Register(x86_reg))
            }
            RegisterLocation::XMM(xmm_index, _sub_index) => {
                // XMM registers not yet supported in Operand enum
                Err(format!("XMM register {} not yet supported", xmm_index))
            }
            RegisterLocation::MEM(address) => {
                // Spilled registers are stored at absolute addresses in the BSS segment
                // Use absolute memory addressing instead of RBP-relative
                Ok(Operand::AbsoluteAddress(address))
            }
        }
    }

    // Takes the memory address where other registers would be stored in memory
    fn new(mem_addr: u64) -> Self
    where
        Self: Sized,
    {
        // Create a mapping respecting RISC-V and x86-64 calling conventions:
        // RISC-V caller-saved → x86-64 caller-saved (RAX, RCX, RDX, RSI, RDI, R8-R10)
        // RISC-V callee-saved → x86-64 callee-saved (RBX, RBP, R15)
        // Overflow → memory spilling
        // Reserved scratch → R11, R12, R13, R14 (used for complex instructions and temporary operations)
        //
        // x86-64 registers: RAX(0), RCX(1), RDX(2), RBX(3), RSP(4), RBP(5), RSI(6), RDI(7), R8(8), R9(9), R10(10), R11(11), R12(12), R13(13), R14(14), R15(15)
        let map = [
            RegisterLocation::ZERO,                // x0: hardwired zero
            RegisterLocation::GPR(0),              // x1(ra): rax (caller-saved, return addr)
            RegisterLocation::GPR(4),              // x2(sp): rsp (stack pointer)
            RegisterLocation::GPR(3),              // x3(gp): rbx (callee-saved, rarely used)
            RegisterLocation::MEM(mem_addr), // x4(tp): memory (callee-saved, rarely used, freed for scratch)
            RegisterLocation::MEM(mem_addr + 144), // x5(t0): memory (freed R11 for scratch register)
            RegisterLocation::GPR(10),             // x6(t1): r10 (caller-saved temp)
            RegisterLocation::GPR(9),              // x7(t2): r9 (caller-saved temp)
            RegisterLocation::GPR(5),              // x8(s0/fp): rbp (frame pointer, callee-saved)
            RegisterLocation::MEM(mem_addr + 8), // x9(s1): memory (callee-saved, freed R13 for scratch)
            RegisterLocation::GPR(7),            // x10(a0): rdi (arg0, caller-saved)
            RegisterLocation::GPR(6),            // x11(a1): rsi (arg1, caller-saved)
            RegisterLocation::GPR(2),            // x12(a2): rdx (arg2, caller-saved)
            RegisterLocation::GPR(1),            // x13(a3): rcx (arg3, caller-saved)
            RegisterLocation::GPR(8),            // x14(a4): r8 (arg4, caller-saved)
            RegisterLocation::GPR(15),           // x15(a5): r15 (caller-saved)
            RegisterLocation::MEM(mem_addr + 16), // x16(a6): memory (overflow)
            RegisterLocation::MEM(mem_addr + 24), // x17(a7): memory (overflow)
            RegisterLocation::MEM(mem_addr + 32), // x18(s2): memory (callee-saved)
            RegisterLocation::MEM(mem_addr + 40), // x19(s3): memory (callee-saved)
            RegisterLocation::MEM(mem_addr + 48), // x20(s4): memory (callee-saved)
            RegisterLocation::MEM(mem_addr + 56), // x21(s5): memory (callee-saved)
            RegisterLocation::MEM(mem_addr + 64), // x22(s6): memory (callee-saved)
            RegisterLocation::MEM(mem_addr + 72), // x23(s7): memory (callee-saved)
            RegisterLocation::MEM(mem_addr + 80), // x24(s8): memory (callee-saved)
            RegisterLocation::MEM(mem_addr + 88), // x25(s9): memory (callee-saved)
            RegisterLocation::MEM(mem_addr + 96), // x26(s10): memory (callee-saved)
            RegisterLocation::MEM(mem_addr + 104), // x27(s11): memory (callee-saved)
            RegisterLocation::MEM(mem_addr + 112), // x28(t3): memory (caller-saved temp, spilled)
            RegisterLocation::MEM(mem_addr + 120), // x29(t4): memory (freed R14 for scratch)
            RegisterLocation::MEM(mem_addr + 128), // x30(t5): memory (spilled temp)
            RegisterLocation::MEM(mem_addr + 136), // x31(t6): memory (spilled temp)
        ];

        RegisterMapping::new(map)
    }
}
