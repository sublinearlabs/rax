use crate::aot::registers::{RiscvRegister, X86Gpr, X86Xmm};

/// Errors that can occur during register mapping validation.
#[derive(Debug, PartialEq, Eq)]
enum MapError {
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
struct RegisterMapping {
    mapping: [MapTarget; 32],
}

impl RegisterMapping {
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
    /// On success, returns the `RegisterMapping` and a vector of unused x86 GPRs
    /// that are available for temporary allocation.
    fn init(mapping: [MapTarget; 32]) -> Result<(Self, Vec<X86Gpr>), MapError> {
        let unused_gprs = validate_mapping(&mapping)?;
        Ok((Self { mapping }, unused_gprs))
    }

    /// Returns the `MapTarget` for a given `RiscvRegister`.
    ///
    /// # Panics
    ///
    /// Panics if `reg` has an invalid discriminant (should not happen for valid `RiscvRegister` values).
    fn get(&self, reg: &RiscvRegister) -> &MapTarget {
        &self.mapping[*reg as usize]
    }
}

/// Possible mapping targets on the x86 hardware.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum MapTarget {
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
enum XmmLane {
    /// Lower 64 bits of the XMM register.
    Low = 0,
    /// Upper 64 bits of the XMM register.
    High = 1,
}

/// Validates a register mapping and returns unused x86 GPRs.
///
/// Checks the following invariants:
/// 1. Only RISC-V x0 may map to `MapTarget::ConstZero`
/// 2. RISC-V x0 must map to `MapTarget::ConstZero`
/// 3. No two RISC-V registers map to the same x86 GPR
/// 4. No two RISC-V registers map to the same XMM lane
///
/// # Errors
///
/// Returns an error if any invariant is violated:
/// - `ConstZeroRequiresX0` — a non-x0 RISC-V register maps to `ConstZero`
/// - `X0RequiresConstZero` — x0 does not map to `ConstZero`
/// - `GprCollision` — two RISC-V registers map to the same x86 GPR
/// - `XmmCollision` — two RISC-V registers map to overlapping XMM locations
///
/// # Returns
///
/// On success, returns a vector of x86 GPR registers that are not used by any
/// RISC-V register mapping. These are available for temporary allocation.
fn validate_mapping(mapping: &[MapTarget; 32]) -> Result<Vec<X86Gpr>, MapError> {
    let mut gpr_slots: Vec<Option<usize>> = vec![None; 16];
    let mut xmm_slots: Vec<Option<usize>> = vec![None; 32];

    for (i, target) in mapping.iter().enumerate() {
        match target {
            MapTarget::ConstZero => {
                if i != 0 {
                    return Err(MapError::ConstZeroRequiresX0);
                }
            }

            MapTarget::Gpr(reg) => {
                let gpr_idx = *reg as usize;
                if gpr_slots[gpr_idx].is_some() {
                    return Err(MapError::GprCollision);
                }
                gpr_slots[gpr_idx] = Some(i);
            }

            MapTarget::XmmShared { reg, lane } => {
                let lane_idx = (*reg as usize) * 2 + (*lane as usize);
                if xmm_slots[lane_idx].is_some() {
                    return Err(MapError::XmmCollision);
                }

                xmm_slots[lane_idx] = Some(i);
            }

            MapTarget::XmmExclusive(reg) => {
                let base_idx = (*reg as usize) * 2;
                if xmm_slots[base_idx].is_some() || xmm_slots[base_idx + 1].is_some() {
                    return Err(MapError::XmmCollision);
                }

                xmm_slots[base_idx] = Some(i);
                xmm_slots[base_idx + 1] = Some(i);
            }
        }
    }

    if !matches!(mapping[0], MapTarget::ConstZero) {
        return Err(MapError::X0RequiresConstZero);
    }

    let mut unused_gprs = Vec::new();
    for (idx, slot) in gpr_slots.iter().enumerate() {
        if slot.is_none() {
            unused_gprs.push(X86Gpr::from_index(idx).unwrap());
        }
    }

    Ok(unused_gprs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_mapping() -> [MapTarget; 32] {
        let mut mapping = [MapTarget::XmmExclusive(X86Xmm::Xmm15); 32];
        mapping[0] = MapTarget::ConstZero;

        for (i, slot) in mapping.iter_mut().enumerate().skip(1) {
            let lane_idx = i - 1;
            let reg = X86Xmm::from_index(lane_idx / 2).expect("xmm index must be in range");
            let lane = if lane_idx % 2 == 0 {
                XmmLane::Low
            } else {
                XmmLane::High
            };
            *slot = MapTarget::XmmShared { reg, lane };
        }

        mapping[1] = MapTarget::Gpr(X86Gpr::Rax);
        mapping[2] = MapTarget::Gpr(X86Gpr::Rbx);
        mapping[3] = MapTarget::Gpr(X86Gpr::R10);

        mapping
    }

    #[test]
    fn init_accepts_valid_mapping_and_returns_unused_gprs() {
        let mapping = valid_mapping();
        let (_, unused_gprs) = RegisterMapping::init(mapping).expect("mapping should be valid");

        assert!(!unused_gprs.contains(&X86Gpr::Rax));
        assert!(!unused_gprs.contains(&X86Gpr::Rbx));
        assert!(!unused_gprs.contains(&X86Gpr::R10));
        assert!(unused_gprs.contains(&X86Gpr::Rcx));
    }

    #[test]
    fn init_rejects_const_zero_on_non_x0() {
        let mut mapping = valid_mapping();
        mapping[7] = MapTarget::ConstZero;

        let err = RegisterMapping::init(mapping).expect_err("should reject ConstZero on non-x0");
        assert_eq!(err, MapError::ConstZeroRequiresX0);
    }

    #[test]
    fn init_rejects_x0_not_const_zero() {
        let mut mapping = valid_mapping();
        mapping[0] = MapTarget::Gpr(X86Gpr::Rcx);

        let err = RegisterMapping::init(mapping).expect_err("x0 must be ConstZero");
        assert_eq!(err, MapError::X0RequiresConstZero);
    }

    #[test]
    fn init_rejects_gpr_collision() {
        let mut mapping = valid_mapping();
        mapping[7] = MapTarget::Gpr(X86Gpr::Rax);

        let err = RegisterMapping::init(mapping).expect_err("duplicate GPR should fail");
        assert_eq!(err, MapError::GprCollision);
    }

    #[test]
    fn init_rejects_xmm_shared_lane_collision() {
        let mut mapping = valid_mapping();
        mapping[8] = MapTarget::XmmShared {
            reg: X86Xmm::Xmm1,
            lane: XmmLane::High,
        };

        let err = RegisterMapping::init(mapping).expect_err("duplicate XMM lane should fail");
        assert_eq!(err, MapError::XmmCollision);
    }

    #[test]
    fn init_rejects_xmm_exclusive_colliding_with_shared_lane() {
        let mut mapping = valid_mapping();
        mapping[8] = MapTarget::XmmExclusive(X86Xmm::Xmm1);

        let err = RegisterMapping::init(mapping).expect_err("exclusive XMM should collide");
        assert_eq!(err, MapError::XmmCollision);
    }

    #[test]
    fn init_rejects_xmm_exclusive_collision() {
        let mut mapping = valid_mapping();
        mapping[7] = MapTarget::XmmExclusive(X86Xmm::Xmm1);

        let err = RegisterMapping::init(mapping).expect_err("duplicate exclusive XMM should fail");
        assert_eq!(err, MapError::XmmCollision);
    }

    #[test]
    fn get_returns_target_for_register() {
        let mut mapping = valid_mapping();
        mapping[RiscvRegister::A0 as usize] = MapTarget::Gpr(X86Gpr::Rdi);

        let (reg_map, _) = RegisterMapping::init(mapping).expect("mapping should be valid");
        assert_eq!(
            reg_map.get(&RiscvRegister::A0),
            &MapTarget::Gpr(X86Gpr::Rdi)
        );
    }
}
