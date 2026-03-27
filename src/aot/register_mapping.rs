use std::ops::Index;

/// Represents the different locations a RISCV register might be stored
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RegisterLocation {
    GPR(u8),
    XMM(u8, u8),
    MEM(u64),
}

pub(crate) struct RegisterMapping {
    map: [RegisterLocation; 32],
}

/// Represents a valid RISCV register
struct RiscvRegister(u8);

impl RiscvRegister {
    fn new(reg_index: u8) -> Self {
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
            1 => RegisterLocation::GPR(3),
            2 => RegisterLocation::XMM(4, 5),
            3 => RegisterLocation::MEM(0x1234),
            _ => RegisterLocation::GPR(0),
        });
        let mapping = RegisterMapping { map };

        assert_eq!(mapping[RiscvRegister::new(1)], RegisterLocation::GPR(3));
        assert_eq!(mapping[RiscvRegister::new(2)], RegisterLocation::XMM(4, 5));
        assert_eq!(
            mapping[RiscvRegister::new(3)],
            RegisterLocation::MEM(0x1234)
        );
    }
}
