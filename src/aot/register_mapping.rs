use std::ops::Index;

/// Represents the different locations a RISCV register might be stored
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
