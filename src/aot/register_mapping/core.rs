use crate::aot::registers::{RiscvRegister, X86Gpr, X86Xmm};

use super::builder::RegisterMappingBuilder;
use super::validate::validate_mapping;

/// Errors that can occur during register mapping validation.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum MapError {
    /// Attempted to map a RISC-V register other than x0 to ConstZero.
    ConstZeroRequiresX0,
    /// x0 (the zero register) was not mapped to ConstZero.
    X0RequiresConstZero,
    /// Two RISC-V registers mapped to the same x86 GPR.
    GprCollision,
    /// Two RISC-V registers mapped to the same XMM register (or overlapping lanes).
    XmmCollision,
}

/// Maps RISC-V registers to x86 locations.
///
/// # Invariants
/// - RISC-V x0 must map to `MapTarget::ConstZero`
/// - No two RISC-V registers may map to overlapping x86 locations (GPR or XMM lane)
#[derive(Debug)]
pub(crate) struct RegisterMapping {
    mapping: [MapTarget; 32],
}

/// Validated mapping handoff containing both register assignments and temp pool.
///
/// This type couples `RegisterMapping` with the derived set of unused x86 GPRs
/// so downstream users cannot accidentally pass desynchronized values.
#[derive(Debug)]
pub(crate) struct MappingPlan {
    reg_map: RegisterMapping,
    unused_gprs: Vec<X86Gpr>,
}

impl MappingPlan {
    /// Consumes the plan and returns its coupled components.
    ///
    /// # Returns
    ///
    /// The validated `RegisterMapping` and the corresponding list of unused
    /// x86 GPRs available for temporary allocation.
    pub(crate) fn into_parts(self) -> (RegisterMapping, Vec<X86Gpr>) {
        (self.reg_map, self.unused_gprs)
    }
}

impl RegisterMapping {
    /// Creates a builder for constructing a `RegisterMapping` by hand.
    pub(crate) fn builder() -> RegisterMappingBuilder {
        RegisterMappingBuilder::new()
    }

    /// Initializes a new `RegisterMapping` from a fixed mapping array.
    ///
    /// Validates the mapping to ensure no overlaps and that x0 maps to `ConstZero`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A RISC-V register other than x0 maps to `ConstZero`
    /// - x0 does not map to `ConstZero`
    /// - Two RISC-V registers map to the same x86 GPR
    /// - Two RISC-V registers map to the same XMM register (or overlapping lanes)
    ///
    /// # Returns
    ///
    /// On success, returns a `MappingPlan` containing the validated
    /// `RegisterMapping` and the corresponding unused x86 GPR set.
    pub(crate) fn init(mapping: [MapTarget; 32]) -> Result<MappingPlan, MapError> {
        let unused_gprs = validate_mapping(&mapping)?;
        Ok(MappingPlan {
            reg_map: Self { mapping },
            unused_gprs,
        })
    }

    /// Returns the hand-authored default register mapping plan.
    ///
    /// This mapping is intentionally explicit and validated through `init()`
    /// so invariants and derived temp registers remain coupled in `MappingPlan`.
    pub(crate) fn default_plan() -> MappingPlan {
        let mapping = [
            // x0 (zero)
            MapTarget::ConstZero,
            // x1 (ra)
            MapTarget::Gpr(X86Gpr::Rbx),
            // x2 (sp)
            MapTarget::Gpr(X86Gpr::Rsp),
            // x3 (gp)
            MapTarget::XmmShared {
                reg: X86Xmm::Xmm12,
                lane: XmmLane::Low,
            },
            // x4 (tp)
            MapTarget::XmmShared {
                reg: X86Xmm::Xmm12,
                lane: XmmLane::High,
            },
            // x5 (t0)
            MapTarget::Gpr(X86Gpr::R14),
            // x6 (t1)
            MapTarget::Gpr(X86Gpr::R15),
            // x7 (t2)
            MapTarget::Gpr(X86Gpr::Rbp),
            // x8 (s0/fp)
            MapTarget::XmmExclusive(X86Xmm::Xmm1),
            // x9 (s1)
            MapTarget::XmmExclusive(X86Xmm::Xmm2),
            // x10 (a0)
            MapTarget::Gpr(X86Gpr::Rdi),
            // x11 (a1)
            MapTarget::Gpr(X86Gpr::Rsi),
            // x12 (a2)
            MapTarget::Gpr(X86Gpr::Rdx),
            // x13 (a3)
            MapTarget::Gpr(X86Gpr::R10),
            // x14 (a4)
            MapTarget::Gpr(X86Gpr::R8),
            // x15 (a5)
            MapTarget::Gpr(X86Gpr::R9),
            // x16 (a6)
            MapTarget::XmmExclusive(X86Xmm::Xmm3),
            // x17 (a7)
            MapTarget::Gpr(X86Gpr::Rax),
            // x18 (s2)
            MapTarget::XmmExclusive(X86Xmm::Xmm4),
            // x19 (s3)
            MapTarget::XmmExclusive(X86Xmm::Xmm5),
            // x20 (s4)
            MapTarget::XmmExclusive(X86Xmm::Xmm6),
            // x21 (s5)
            MapTarget::XmmExclusive(X86Xmm::Xmm7),
            // x22 (s6)
            MapTarget::XmmExclusive(X86Xmm::Xmm8),
            // x23 (s7)
            MapTarget::XmmExclusive(X86Xmm::Xmm9),
            // x24 (s8)
            MapTarget::XmmExclusive(X86Xmm::Xmm10),
            // x25 (s9)
            MapTarget::XmmExclusive(X86Xmm::Xmm11),
            // x26 (s10)
            MapTarget::XmmShared {
                reg: X86Xmm::Xmm13,
                lane: XmmLane::Low,
            },
            // x27 (s11)
            MapTarget::XmmShared {
                reg: X86Xmm::Xmm13,
                lane: XmmLane::High,
            },
            // x28 (t3)
            MapTarget::XmmShared {
                reg: X86Xmm::Xmm14,
                lane: XmmLane::Low,
            },
            // x29 (t4)
            MapTarget::XmmShared {
                reg: X86Xmm::Xmm14,
                lane: XmmLane::High,
            },
            // x30 (t5)
            MapTarget::XmmShared {
                reg: X86Xmm::Xmm15,
                lane: XmmLane::Low,
            },
            // x31 (t6)
            MapTarget::XmmShared {
                reg: X86Xmm::Xmm15,
                lane: XmmLane::High,
            },
        ];

        Self::init(mapping)
            .expect("default register mapping must remain valid and collision-free")
    }

    /// Returns the `MapTarget` for a given `RiscvRegister`.
    ///
    /// # Panics
    ///
    /// Panics if `reg` has an invalid discriminant (should not happen for valid `RiscvRegister` values).
    pub(crate) fn get(&self, reg: &RiscvRegister) -> &MapTarget {
        &self.mapping[*reg as usize]
    }
}

/// Possible mapping targets on the x86 hardware.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum MapTarget {
    /// Virtual register representing the constant value 0.
    ///
    /// Only RISC-V x0 may be mapped to this target.
    ///
    /// **Optimization note:** This register need not be materialized on physical x86
    /// hardware. Instead, handle it at translation time (e.g., treat reads as constant 0,
    /// ignore writes) to produce more efficient assembly output.
    ConstZero,
    /// Map to a specific x86 general-purpose register.
    Gpr(X86Gpr),
    /// Map to a specific 64-bit lane of an x86 XMM register (shared with another RISC-V register).
    ///
    /// Two RISC-V registers can share different lanes of the same XMM register.
    XmmShared { reg: X86Xmm, lane: XmmLane },
    /// Map to a full 128-bit x86 XMM register (exclusive ownership).
    XmmExclusive(X86Xmm),
}

/// High or Low 64-bit lane within a 128-bit XMM register.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum XmmLane {
    /// Lower 64 bits of the XMM register.
    Low = 0,
    /// Upper 64 bits of the XMM register.
    High = 1,
}
