use crate::aot::registers::{RiscvRegister, X86Gpr, X86Xmm};

enum MapError {
    OnlyX0MapsToConstZero,
    X0MustMapToConstZero,
    GPRCollision,
    XMMCollision,
}

/// Maps RiscvRegisters to x86 Locations
struct RegisterMapping {
    mapping: [MapTarget; 32],
}

impl RegisterMapping {
    /// Initializes a new RegisterMapping
    ///
    /// Ensures that there is no overlap between the mapping targets
    ///
    /// Returns:
    /// - RegisterMapping
    /// - A vec of unused GPR registers based on the mapping
    fn init(mapping: [MapTarget; 32]) -> Result<(Self, Vec<X86Gpr>), MapError> {
        let unused_gprs = validate_mapping(&mapping)?;
        Ok((Self { mapping }, unused_gprs))
    }

    /// Retuns the `MapTarget` for a given `RiscvRegister`
    fn get(&self, reg: &RiscvRegister) -> &MapTarget {
        &self.mapping[*reg as usize]
    }
}

/// Possible mapping targets on the x86 hardware
enum MapTarget {
    /// Concept for a register that is always 0
    ///
    /// Note:
    /// Only RiscvRegister::X0 can be mapped to this
    ///
    /// One can avoid materializing this register
    /// on physical hardware, instead handle it at
    /// translation level, this will lead to more
    /// efficient assembly output
    ConstZero,
    Gpr(X86Gpr),
    XmmShared {
        reg: X86Xmm,
        lane: XmmLane,
    },
    XmmExclusive(X86Xmm),
}

/// High or Low 64 bit lanes for a 128 bit Xmm Register
#[derive(Copy, Clone)]
#[repr(u8)]
enum XmmLane {
    Low = 0,
    High = 1,
}

fn validate_mapping(mapping: &[MapTarget; 32]) -> Result<Vec<X86Gpr>, MapError> {
    let mut gpr_slots: Vec<Option<usize>> = vec![None; 16];
    let mut xmm_slots: Vec<Option<usize>> = vec![None; 32];

    for (i, target) in mapping.iter().enumerate() {
        match target {
            MapTarget::ConstZero => {
                if i != 0 {
                    return Err(MapError::OnlyX0MapsToConstZero);
                }
            }

            MapTarget::Gpr(reg) => {
                let gpr_idx = *reg as usize;
                if gpr_slots[gpr_idx].is_some() {
                    return Err(MapError::GPRCollision);
                }
                gpr_slots[gpr_idx] = Some(i);
            }

            MapTarget::XmmShared { reg, lane } => {
                let lane_idx = (*reg as usize) * 2 + (*lane as usize);
                if xmm_slots[lane_idx].is_some() {
                    return Err(MapError::XMMCollision);
                }

                xmm_slots[lane_idx] = Some(i);
            }

            MapTarget::XmmExclusive(reg) => {
                let base_idx = (*reg as usize) * 2;
                if xmm_slots[base_idx].is_some() || xmm_slots[base_idx + 1].is_some() {
                    return Err(MapError::XMMCollision);
                }

                xmm_slots[base_idx] = Some(i);
                xmm_slots[base_idx + 1] = Some(i);
            }
        }
    }

    if !matches!(mapping[0], MapTarget::ConstZero) {
        return Err(MapError::X0MustMapToConstZero);
    }

    let mut unused_gprs = Vec::new();
    for (idx, slot) in gpr_slots.iter().enumerate() {
        if slot.is_none() {
            unused_gprs.push(X86Gpr::from_index(idx).unwrap());
        }
    }

    Ok(unused_gprs)
}
