/// Represents the different locations a RISCV register might be stored
pub(crate) enum RegisterLocation {
    GPR(u8),
    XMM(u8, u8),
    MEM(u64),
}

pub(crate) struct RegisterMapping {
    map: [RegisterLocation; 32],
}

impl RegisterMapping {
    pub(crate) fn get_register_location(&self, riscv_register: u8) -> &RegisterLocation {
        &self.map[riscv_register as usize]
    }
}
