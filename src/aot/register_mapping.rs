use std::ops::Index;

/// Represents the different locations a RISCV register might be stored
// TODO: add a zero register
// TODO: consider adding Mem spill (handle base + offset semantics)
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub(crate) enum RegisterLocation {
    Gpr(u8),
    Xmm(u8),
    XmmShared(u8, XmmLane),
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub(crate) enum XmmLane {
    UPPER,
    LOWER,
}

pub(crate) struct RegisterMapping {
    map: [RegisterLocation; 32],
    /// Represents the x86 gprs that have been reserved
    /// for temporary operations
    pub(crate) temps: [u8; 3],
}

impl RegisterMapping {
    /// Create a new register mapping from an array of register locations
    pub fn new(map: [RegisterLocation; 32], temps: [u8; 3]) -> Self {
        RegisterMapping { map, temps }
    }
}

#[derive(PartialEq)]
/// Represents a valid RISCV register
pub(crate) struct RiscvRegister(u8);

impl RiscvRegister {
    pub(crate) fn new(reg_index: u8) -> Self {
        if reg_index >= 32 {
            panic!("riscv registers are x0 - x31");
        }

        Self(reg_index)
    }
}

impl Index<RiscvRegister> for RegisterMapping {
    type Output = RegisterLocation;

    fn index(&self, index: RiscvRegister) -> &Self::Output {
        &self.map[index.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use crate::aot::register_mapping::XmmLane;

    use super::{RegisterLocation, RegisterMapping, RiscvRegister};

    #[test]
    fn riscv_register_new_accepts_lower_bound() {
        let _ = RiscvRegister::new(0);
    }

    #[test]
    fn riscv_register_new_accepts_upper_bound() {
        let _ = RiscvRegister::new(31);
    }

    #[test]
    #[should_panic(expected = "riscv registers are x0 - x31")]
    fn riscv_register_new_panics_on_out_of_range() {
        let _ = RiscvRegister::new(32);
    }

    #[test]
    fn register_mapping_index_returns_expected_locations() {
        let map = std::array::from_fn(|idx| match idx {
            1 => RegisterLocation::Gpr(3),
            2 => RegisterLocation::XmmShared(4, XmmLane::UPPER),
            _ => RegisterLocation::Gpr(0),
        });
        let mapping = RegisterMapping { map, temp_base: 0 };

        assert_eq!(mapping[RiscvRegister::new(1)], RegisterLocation::Gpr(3));
        assert_eq!(
            mapping[RiscvRegister::new(2)],
            RegisterLocation::XmmShared(4, XmmLane::UPPER)
        );
    }
}
