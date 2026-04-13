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
    fn new() -> Self
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
                // Zero register maps to RAX with immediate 0
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
                // Memory location: use RBP as base with offset
                Ok(Operand::Memory {
                    base: crate::translate::x86_insn::X86Register::RBP,
                    offset: (address & 0xFFFFFFFF) as i32,
                })
            }
        }
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        // Create a default mapping: all RISC-V registers map to GPRs (0-15)
        // For registers beyond 15, spill to memory
        let mut map = [RegisterLocation::ZERO; 32];

        // Map RISC-V x0-x15 to GPR 0-15
        for i in 0..16 {
            map[i] = RegisterLocation::GPR(i as u8);
        }

        // Map RISC-V x16-x31 to memory locations
        for i in 16..32 {
            map[i] = RegisterLocation::MEM(0x1000 + ((i as u64) * 8));
        }

        RegisterMapping::new(map)
    }
}
