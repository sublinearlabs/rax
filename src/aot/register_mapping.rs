use crate::aot::registers::{RiscvRegister, X86Gpr, X86Xmm};

/// Errors that can occur during register mapping validation.
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
    XmmShared {
        reg: X86Xmm,
        lane: XmmLane,
    },
    /// Map to a full 128-bit x86 XMM register (exclusive ownership).
    XmmExclusive(X86Xmm),
}

/// High or Low 64-bit lane within a 128-bit XMM register.
#[derive(Copy, Clone)]
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
