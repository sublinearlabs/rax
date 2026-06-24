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
        let mut b = Self::builder();

        // x0 is fixed by builder invariant (ConstZero).

        // Syscall alignment.
        b.map_gpr(RiscvRegister::A0, X86Gpr::Rdi).unwrap();
        b.map_gpr(RiscvRegister::A1, X86Gpr::Rsi).unwrap();
        b.map_gpr(RiscvRegister::A2, X86Gpr::Rdx).unwrap();
        b.map_gpr(RiscvRegister::A3, X86Gpr::R10).unwrap();
        b.map_gpr(RiscvRegister::A4, X86Gpr::R8).unwrap();
        b.map_gpr(RiscvRegister::A5, X86Gpr::R9).unwrap();

        b.map_gpr(RiscvRegister::A7, X86Gpr::Rax).unwrap();

        // Stack pointer alignment.
        b.map_gpr(RiscvRegister::Sp, X86Gpr::Rsp).unwrap();

        // Remaining registers: matched mapping.
        b.map_xmm_exclusive(RiscvRegister::Ra, X86Xmm::Xmm0)
            .unwrap();
        b.map_xmm_shared(RiscvRegister::Gp, X86Xmm::Xmm12, XmmLane::Low)
            .unwrap();
        b.map_xmm_shared(RiscvRegister::Tp, X86Xmm::Xmm12, XmmLane::High)
            .unwrap();
        b.map_gpr(RiscvRegister::T0, X86Gpr::R14).unwrap();
        b.map_gpr(RiscvRegister::T1, X86Gpr::R15).unwrap();
        b.map_gpr(RiscvRegister::T2, X86Gpr::Rbp).unwrap();
        b.map_xmm_exclusive(RiscvRegister::S0, X86Xmm::Xmm1)
            .unwrap();
        b.map_xmm_exclusive(RiscvRegister::S1, X86Xmm::Xmm2)
            .unwrap();
        b.map_xmm_exclusive(RiscvRegister::A6, X86Xmm::Xmm3)
            .unwrap();
        b.map_xmm_exclusive(RiscvRegister::S2, X86Xmm::Xmm4)
            .unwrap();
        b.map_xmm_exclusive(RiscvRegister::S3, X86Xmm::Xmm5)
            .unwrap();
        b.map_xmm_exclusive(RiscvRegister::S4, X86Xmm::Xmm6)
            .unwrap();
        b.map_xmm_exclusive(RiscvRegister::S5, X86Xmm::Xmm7)
            .unwrap();
        b.map_xmm_exclusive(RiscvRegister::S6, X86Xmm::Xmm8)
            .unwrap();
        b.map_xmm_exclusive(RiscvRegister::S7, X86Xmm::Xmm9)
            .unwrap();
        b.map_xmm_exclusive(RiscvRegister::S8, X86Xmm::Xmm10)
            .unwrap();
        b.map_xmm_exclusive(RiscvRegister::S9, X86Xmm::Xmm11)
            .unwrap();
        b.map_xmm_shared(RiscvRegister::S10, X86Xmm::Xmm13, XmmLane::Low)
            .unwrap();
        b.map_xmm_shared(RiscvRegister::S11, X86Xmm::Xmm13, XmmLane::High)
            .unwrap();
        b.map_xmm_shared(RiscvRegister::T3, X86Xmm::Xmm14, XmmLane::Low)
            .unwrap();
        b.map_xmm_shared(RiscvRegister::T4, X86Xmm::Xmm14, XmmLane::High)
            .unwrap();
        b.map_xmm_shared(RiscvRegister::T5, X86Xmm::Xmm15, XmmLane::Low)
            .unwrap();
        b.map_xmm_shared(RiscvRegister::T6, X86Xmm::Xmm15, XmmLane::High)
            .unwrap();

        b.build().unwrap()
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub(crate) enum XmmLane {
    /// Lower 64 bits of the XMM register.
    Low = 0,
    /// Upper 64 bits of the XMM register.
    High = 1,
}

#[cfg(test)]
mod tests {
    use super::RegisterMapping;

    #[test]
    fn default_plan_builds() {
        let _ = RegisterMapping::default_plan();
    }
}
