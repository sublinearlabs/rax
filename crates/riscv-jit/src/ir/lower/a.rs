use riscv::decode::Instruction;
use crate::ir::{AtomicRmwOp, AtomicWidth, IrBuilder, IrType, Reg};

pub(crate) fn lower_a_into(insn: &Instruction, builder: &mut IrBuilder) {
    match insn {
        Instruction::LrW(r) => {
            let addr = builder.get_reg(reg_from_u8(r.rs1));
            let value = builder.load_reserved(addr, AtomicWidth::W, IrType::I32);
            let value = builder.sext(value, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), value);
        }
        Instruction::LrD(r) => {
            let addr = builder.get_reg(reg_from_u8(r.rs1));
            let value = builder.load_reserved(addr, AtomicWidth::D, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), value);
        }
        Instruction::ScW(r) => {
            let addr = builder.get_reg(reg_from_u8(r.rs1));
            let val = builder.get_reg(reg_from_u8(r.rs2));
            let val = builder.trunc(val, IrType::I64, IrType::I32);
            let result = builder.store_conditional(addr, val, AtomicWidth::W, IrType::I32);
            let result = builder.sext(result, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), result);
        }
        Instruction::ScD(r) => {
            let addr = builder.get_reg(reg_from_u8(r.rs1));
            let val = builder.get_reg(reg_from_u8(r.rs2));
            let result = builder.store_conditional(addr, val, AtomicWidth::D, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), result);
        }
        Instruction::AmoSwapW(r) => lower_amo_w(builder, AtomicRmwOp::Xchg, r.rd, r.rs1, r.rs2),
        Instruction::AmoAddW(r) => lower_amo_w(builder, AtomicRmwOp::Add, r.rd, r.rs1, r.rs2),
        Instruction::AmoXorW(r) => lower_amo_w(builder, AtomicRmwOp::Xor, r.rd, r.rs1, r.rs2),
        Instruction::AmoAndW(r) => lower_amo_w(builder, AtomicRmwOp::And, r.rd, r.rs1, r.rs2),
        Instruction::AmoOrW(r) => lower_amo_w(builder, AtomicRmwOp::Or, r.rd, r.rs1, r.rs2),
        Instruction::AmoMinW(r) => lower_amo_w(builder, AtomicRmwOp::Min, r.rd, r.rs1, r.rs2),
        Instruction::AmoMaxW(r) => lower_amo_w(builder, AtomicRmwOp::Max, r.rd, r.rs1, r.rs2),
        Instruction::AmoMinuW(r) => lower_amo_w(builder, AtomicRmwOp::Umin, r.rd, r.rs1, r.rs2),
        Instruction::AmoMaxuW(r) => lower_amo_w(builder, AtomicRmwOp::Umax, r.rd, r.rs1, r.rs2),
        Instruction::AmoSwapD(r) => lower_amo_d(builder, AtomicRmwOp::Xchg, r.rd, r.rs1, r.rs2),
        Instruction::AmoAddD(r) => lower_amo_d(builder, AtomicRmwOp::Add, r.rd, r.rs1, r.rs2),
        Instruction::AmoXorD(r) => lower_amo_d(builder, AtomicRmwOp::Xor, r.rd, r.rs1, r.rs2),
        Instruction::AmoAndD(r) => lower_amo_d(builder, AtomicRmwOp::And, r.rd, r.rs1, r.rs2),
        Instruction::AmoOrD(r) => lower_amo_d(builder, AtomicRmwOp::Or, r.rd, r.rs1, r.rs2),
        Instruction::AmoMinD(r) => lower_amo_d(builder, AtomicRmwOp::Min, r.rd, r.rs1, r.rs2),
        Instruction::AmoMaxD(r) => lower_amo_d(builder, AtomicRmwOp::Max, r.rd, r.rs1, r.rs2),
        Instruction::AmoMinuD(r) => lower_amo_d(builder, AtomicRmwOp::Umin, r.rd, r.rs1, r.rs2),
        Instruction::AmoMaxuD(r) => lower_amo_d(builder, AtomicRmwOp::Umax, r.rd, r.rs1, r.rs2),
        _ => panic!("IR2 A lowering missing for {:?}", insn),
    }
}

fn lower_amo_w(builder: &mut IrBuilder, op: AtomicRmwOp, rd: u8, rs1: u8, rs2: u8) {
    let addr = builder.get_reg(reg_from_u8(rs1));
    let val = builder.get_reg(reg_from_u8(rs2));
    let val = builder.trunc(val, IrType::I64, IrType::I32);
    let old = builder.atomic_rmw(op, AtomicWidth::W, addr, val, IrType::I32);
    let old = builder.sext(old, IrType::I32, IrType::I64);
    builder.set_reg(reg_from_u8(rd), old);
}

fn lower_amo_d(builder: &mut IrBuilder, op: AtomicRmwOp, rd: u8, rs1: u8, rs2: u8) {
    let addr = builder.get_reg(reg_from_u8(rs1));
    let val = builder.get_reg(reg_from_u8(rs2));
    let old = builder.atomic_rmw(op, AtomicWidth::D, addr, val, IrType::I64);
    builder.set_reg(reg_from_u8(rd), old);
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
