use crate::aot::registers::{RiscvRegister, X86Gpr, X86Xmm};

use super::core::{MapError, MapTarget, RegisterMapping, XmmLane};

/// Errors that can occur while incrementally building a `RegisterMapping`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BuildError {
    /// Attempted to modify a register that is fixed by invariant (`x0`).
    LockedRegister(RiscvRegister),
    /// One or more non-`x0` RISC-V registers were not assigned a mapping.
    UnassignedRegisters(Vec<RiscvRegister>),
    /// Finalized mapping failed structural validation.
    Validation(MapError),
}

/// Builder for hand-authored RISC-V to x86 register mappings.
///
/// # Invariants
/// - `x0` is always fixed to `MapTarget::ConstZero`
/// - `build()` requires every non-`x0` register to be explicitly assigned
#[derive(Debug)]
pub(crate) struct RegisterMappingBuilder {
    mapping: [Option<MapTarget>; 32],
}

impl RegisterMappingBuilder {
    /// Creates a new mapping builder with only `x0` pre-assigned.
    ///
    /// # Returns
    ///
    /// A builder where `x0` is mapped to `ConstZero` and all other registers
    /// are unassigned.
    pub(crate) fn new() -> Self {
        let mut mapping = [None; 32];
        mapping[0] = Some(MapTarget::ConstZero);
        Self { mapping }
    }

    /// Assigns a RISC-V register to an x86 GPR target.
    ///
    /// # Errors
    ///
    /// Returns `BuildError::LockedRegister` if `riscv` is `x0`.
    pub(crate) fn map_gpr(
        &mut self,
        riscv: RiscvRegister,
        gpr: X86Gpr,
    ) -> Result<&mut Self, BuildError> {
        self.set_mapping(riscv, MapTarget::Gpr(gpr))?;
        Ok(self)
    }

    /// Assigns a RISC-V register to a shared 64-bit lane in an x86 XMM register.
    ///
    /// # Errors
    ///
    /// Returns `BuildError::LockedRegister` if `riscv` is `x0`.
    pub(crate) fn map_xmm_shared(
        &mut self,
        riscv: RiscvRegister,
        xmm: X86Xmm,
        lane: XmmLane,
    ) -> Result<&mut Self, BuildError> {
        self.set_mapping(riscv, MapTarget::XmmShared { reg: xmm, lane })?;
        Ok(self)
    }

    /// Assigns a RISC-V register to an exclusive x86 XMM register.
    ///
    /// # Errors
    ///
    /// Returns `BuildError::LockedRegister` if `riscv` is `x0`.
    pub(crate) fn map_xmm_exclusive(
        &mut self,
        riscv: RiscvRegister,
        xmm: X86Xmm,
    ) -> Result<&mut Self, BuildError> {
        self.set_mapping(riscv, MapTarget::XmmExclusive(xmm))?;
        Ok(self)
    }

    /// Finalizes the mapping and validates all invariants.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any non-`x0` register is still unassigned
    /// - The mapping fails validation (e.g. collisions or invalid `ConstZero` usage)
    ///
    /// # Returns
    ///
    /// On success, returns the finalized `RegisterMapping` and the list of
    /// unused x86 GPRs available for temporary allocation.
    pub(crate) fn build(self) -> Result<(RegisterMapping, Vec<X86Gpr>), BuildError> {
        let mut missing = Vec::new();
        for idx in 1..32 {
            if self.mapping[idx].is_none() {
                missing.push(
                    RiscvRegister::from_index(idx).expect("index within riscv register range"),
                );
            }
        }

        if !missing.is_empty() {
            return Err(BuildError::UnassignedRegisters(missing));
        }

        let mut finalized = [MapTarget::ConstZero; 32];
        for (idx, item) in self.mapping.into_iter().enumerate() {
            finalized[idx] = item.expect("all registers must be assigned before finalization");
        }

        RegisterMapping::init(finalized).map_err(BuildError::Validation)
    }

    /// Internal helper for setting one register mapping slot.
    fn set_mapping(&mut self, riscv: RiscvRegister, target: MapTarget) -> Result<(), BuildError> {
        if matches!(riscv, RiscvRegister::Zero) {
            return Err(BuildError::LockedRegister(riscv));
        }
        self.mapping[riscv as usize] = Some(target);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fully_assign_builder(builder: &mut RegisterMappingBuilder) {
        for idx in 1..32 {
            let reg = RiscvRegister::from_index(idx).expect("valid riscv reg index");
            let lane_idx = idx - 1;
            let xmm = X86Xmm::from_index(lane_idx / 2).expect("valid xmm index");
            let lane = if lane_idx % 2 == 0 {
                XmmLane::Low
            } else {
                XmmLane::High
            };
            builder
                .map_xmm_shared(reg, xmm, lane)
                .expect("builder assignment should succeed");
        }
    }

    #[test]
    fn builder_reports_unassigned_registers() {
        let mut builder = RegisterMapping::builder();
        builder
            .map_gpr(RiscvRegister::Ra, X86Gpr::Rax)
            .expect("mapping should succeed");

        let err = builder
            .build()
            .expect_err("build should fail when registers are unassigned");
        match err {
            BuildError::UnassignedRegisters(v) => assert!(!v.is_empty()),
            _ => panic!("expected unassigned registers error"),
        }
    }

    #[test]
    fn builder_rejects_mapping_x0() {
        let mut builder = RegisterMapping::builder();
        let err = builder
            .map_gpr(RiscvRegister::Zero, X86Gpr::Rax)
            .expect_err("x0 mapping must be rejected");
        assert_eq!(err, BuildError::LockedRegister(RiscvRegister::Zero));
    }

    #[test]
    fn builder_builds_when_all_registers_are_assigned() {
        let mut builder = RegisterMapping::builder();
        fully_assign_builder(&mut builder);
        builder
            .map_gpr(RiscvRegister::Ra, X86Gpr::Rax)
            .expect("mapping should succeed");
        builder
            .map_gpr(RiscvRegister::Sp, X86Gpr::Rbx)
            .expect("mapping should succeed");

        let (mapping, unused) = builder
            .build()
            .expect("builder should produce valid mapping");
        assert_eq!(
            mapping.get(&RiscvRegister::Ra),
            &MapTarget::Gpr(X86Gpr::Rax)
        );
        assert!(unused.contains(&X86Gpr::Rcx));
    }
}
