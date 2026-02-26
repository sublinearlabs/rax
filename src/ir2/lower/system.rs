use crate::decode::Instruction;
use crate::ir2::{IrBuilder, IrType, Reg};

pub(crate) fn lower_system_into(insn: &Instruction, builder: &mut IrBuilder) {
    match insn {
        Instruction::Csrrw(i) => {
            let csr = (i.imm as u32) & 0x0fff;
            let old = builder.get_csr(csr);
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            builder.set_csr(csr, rs1);
            builder.set_reg(reg_from_u8(i.rd), old);
        }
        Instruction::Csrrs(i) => {
            let csr = (i.imm as u32) & 0x0fff;
            let old = builder.get_csr(csr);
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let new_val = builder.or(old, rs1, IrType::I64);
            builder.set_csr(csr, new_val);
            builder.set_reg(reg_from_u8(i.rd), old);
        }
        Instruction::Csrrc(i) => {
            let csr = (i.imm as u32) & 0x0fff;
            let old = builder.get_csr(csr);
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let ones = builder.const_i64(-1);
            let not_rs1 = builder.xor(rs1, ones, IrType::I64);
            let new_val = builder.and(old, not_rs1, IrType::I64);
            builder.set_csr(csr, new_val);
            builder.set_reg(reg_from_u8(i.rd), old);
        }
        Instruction::Csrrwi(i) => {
            let csr = (i.imm as u32) & 0x0fff;
            let old = builder.get_csr(csr);
            let imm = builder.const_i64((i.rs1 & 0x1f) as i64);
            builder.set_csr(csr, imm);
            builder.set_reg(reg_from_u8(i.rd), old);
        }
        Instruction::Csrrsi(i) => {
            let csr = (i.imm as u32) & 0x0fff;
            let old = builder.get_csr(csr);
            let imm = builder.const_i64((i.rs1 & 0x1f) as i64);
            let new_val = builder.or(old, imm, IrType::I64);
            builder.set_csr(csr, new_val);
            builder.set_reg(reg_from_u8(i.rd), old);
        }
        Instruction::Csrrci(i) => {
            let csr = (i.imm as u32) & 0x0fff;
            let old = builder.get_csr(csr);
            let imm = builder.const_i64((i.rs1 & 0x1f) as i64);
            let ones = builder.const_i64(-1);
            let not_imm = builder.xor(imm, ones, IrType::I64);
            let new_val = builder.and(old, not_imm, IrType::I64);
            builder.set_csr(csr, new_val);
            builder.set_reg(reg_from_u8(i.rd), old);
        }
        Instruction::Ecall => {
            builder.ecall();
        }
        Instruction::Ebreak => {
            builder.ebreak();
            builder.ret();
        }
        _ => panic!("IR2 system lowering missing for {:?}", insn),
    }
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
