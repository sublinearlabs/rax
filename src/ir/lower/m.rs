use crate::decode::Instruction;
use crate::ir::{IrBuilder, IrType, Reg, ValueId};

pub(crate) fn lower_m_into(insn: &Instruction, builder: &mut IrBuilder) {
    match insn {
        Instruction::Mul(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let v = builder.mul(rs1, rs2, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Mulh(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let hi = mulh_hi(builder, rs1, rs2);
            builder.set_reg(reg_from_u8(r.rd), hi);
        }
        Instruction::Mulhsu(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let hi = mulhsu_hi(builder, rs1, rs2);
            builder.set_reg(reg_from_u8(r.rd), hi);
        }
        Instruction::Mulhu(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let hi = mulhu_hi(builder, rs1, rs2);
            builder.set_reg(reg_from_u8(r.rd), hi);
        }
        Instruction::Div(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let v = builder.div(rs1, rs2, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Divu(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let v = builder.divu(rs1, rs2, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Rem(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let v = builder.rem(rs1, rs2, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Remu(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let v = builder.remu(rs1, rs2, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Mulw(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let a = builder.trunc(rs1, IrType::I64, IrType::I32);
            let b = builder.trunc(rs2, IrType::I64, IrType::I32);
            let product = builder.mul(a, b, IrType::I32);
            let v = builder.sext(product, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Divw(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let a = builder.trunc(rs1, IrType::I64, IrType::I32);
            let b = builder.trunc(rs2, IrType::I64, IrType::I32);
            let q = builder.div(a, b, IrType::I32);
            let v = builder.sext(q, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Divuw(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let a = builder.trunc(rs1, IrType::I64, IrType::I32);
            let b = builder.trunc(rs2, IrType::I64, IrType::I32);
            let q = builder.divu(a, b, IrType::I32);
            let v = builder.sext(q, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Remw(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let a = builder.trunc(rs1, IrType::I64, IrType::I32);
            let b = builder.trunc(rs2, IrType::I64, IrType::I32);
            let q = builder.rem(a, b, IrType::I32);
            let v = builder.sext(q, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Remuw(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let a = builder.trunc(rs1, IrType::I64, IrType::I32);
            let b = builder.trunc(rs2, IrType::I64, IrType::I32);
            let q = builder.remu(a, b, IrType::I32);
            let v = builder.sext(q, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        _ => panic!("IR2 M lowering missing for {:?}", insn),
    }
}

fn mulhu_hi(builder: &mut IrBuilder, a: ValueId, b: ValueId) -> ValueId {
    let mask32 = builder.const_i64(0xffff_ffff);
    let shift32 = builder.const_i64(32);

    let a0 = builder.and(a, mask32, IrType::I64);
    let b0 = builder.and(b, mask32, IrType::I64);
    let a1 = builder.shr(a, shift32, IrType::I64);
    let b1 = builder.shr(b, shift32, IrType::I64);

    let p0 = builder.mul(a0, b0, IrType::I64);
    let p1 = builder.mul(a0, b1, IrType::I64);
    let p2 = builder.mul(a1, b0, IrType::I64);
    let p3 = builder.mul(a1, b1, IrType::I64);

    let t = builder.shr(p0, shift32, IrType::I64);
    let u = builder.add(p1, t, IrType::I64);
    let u_low = builder.and(u, mask32, IrType::I64);
    let u_high = builder.shr(u, shift32, IrType::I64);
    let v = builder.add(p2, u_low, IrType::I64);
    let v_high = builder.shr(v, shift32, IrType::I64);

    let hi = builder.add(p3, u_high, IrType::I64);
    builder.add(hi, v_high, IrType::I64)
}

fn mulh_hi(builder: &mut IrBuilder, a: ValueId, b: ValueId) -> ValueId {
    let hi = mulhu_hi(builder, a, b);
    let zero = builder.const_i64(0);
    let sign_a = builder.lt(a, zero);
    let sign_b = builder.lt(b, zero);
    let adj_a = builder.select(sign_a, b, zero, IrType::I64);
    let adj_b = builder.select(sign_b, a, zero, IrType::I64);
    let hi = builder.sub(hi, adj_a, IrType::I64);
    builder.sub(hi, adj_b, IrType::I64)
}

fn mulhsu_hi(builder: &mut IrBuilder, a: ValueId, b: ValueId) -> ValueId {
    let hi = mulhu_hi(builder, a, b);
    let zero = builder.const_i64(0);
    let sign_a = builder.lt(a, zero);
    let adj_a = builder.select(sign_a, b, zero, IrType::I64);
    builder.sub(hi, adj_a, IrType::I64)
}

fn reg_from_u8(idx: u8) -> Reg {
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
