use crate::ir::{IrBuilder, Reg, ValueId};

pub fn reg(builder: &mut IrBuilder, idx: u8) -> ValueId {
    builder.get_reg(reg_from_u8(idx))
}

pub fn set_reg(builder: &mut IrBuilder, idx: u8, val: ValueId) {
    builder.set_reg(reg_from_u8(idx), val);
}

pub fn set_reg_if_needed(builder: &mut IrBuilder, idx: u8, val: ValueId) {
    if idx == 0 {
        return;
    }
    builder.set_reg(reg_from_u8(idx), val);
}

pub fn imm_i32(builder: &mut IrBuilder, value: i32) -> ValueId {
    builder.const_i64(value as i64)
}

pub fn imm_u8(builder: &mut IrBuilder, value: u8) -> ValueId {
    builder.const_i64(value as i64)
}

pub fn imm_u64(builder: &mut IrBuilder, value: u64) -> ValueId {
    builder.const_i64(value as i64)
}

pub fn zimm5(builder: &mut IrBuilder, value: u8) -> ValueId {
    builder.const_i64((value & 0x1f) as i64)
}

pub fn reg_from_u8(idx: u8) -> Reg {
    match idx {
        0 => Reg::X0,
        1 => Reg::X1,
        2 => Reg::X2,
        3 => Reg::X3,
        4 => Reg::X4,
        5 => Reg::X5,
        6 => Reg::X6,
        7 => Reg::X7,
        8 => Reg::X8,
        9 => Reg::X9,
        10 => Reg::X10,
        11 => Reg::X11,
        12 => Reg::X12,
        13 => Reg::X13,
        14 => Reg::X14,
        15 => Reg::X15,
        16 => Reg::X16,
        17 => Reg::X17,
        18 => Reg::X18,
        19 => Reg::X19,
        20 => Reg::X20,
        21 => Reg::X21,
        22 => Reg::X22,
        23 => Reg::X23,
        24 => Reg::X24,
        25 => Reg::X25,
        26 => Reg::X26,
        27 => Reg::X27,
        28 => Reg::X28,
        29 => Reg::X29,
        30 => Reg::X30,
        31 => Reg::X31,
        _ => panic!("invalid register index: {}", idx),
    }
}
