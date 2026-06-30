use crate::aot::registers::X86Gpr;

use super::core::{MapError, MapTarget};

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
pub(super) fn validate_mapping(mapping: &[MapTarget; 32]) -> Result<Vec<X86Gpr>, MapError> {
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
