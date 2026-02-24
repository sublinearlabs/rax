use crate::decode::Instruction;
use crate::ir::{IrBuilder, Reg, ValueId};

pub fn lower_csr(insn: &Instruction, builder: &mut IrBuilder) -> bool {
    match insn {
        Instruction::Csrrw(i) => {
            let csr = csr_from_imm(i.imm);
            let rs1 = reg(builder, i.rs1);
            let prev = builder.get_csr(csr);
            builder.set_csr(csr, rs1);
            set_reg_if_needed(builder, i.rd, prev);
            builder.ret();
            true
        }
        Instruction::Csrrs(i) => {
            let csr = csr_from_imm(i.imm);
            let rs1 = reg(builder, i.rs1);
            let prev = builder.get_csr(csr);
            if i.rs1 != 0 {
                let next = builder.or(prev, rs1);
                builder.set_csr(csr, next);
            }
            set_reg_if_needed(builder, i.rd, prev);
            builder.ret();
            true
        }
        Instruction::Csrrc(i) => {
            let csr = csr_from_imm(i.imm);
            let rs1 = reg(builder, i.rs1);
            let prev = builder.get_csr(csr);
            if i.rs1 != 0 {
                let all_ones = builder.const_i64(-1);
                let inv = builder.xor(rs1, all_ones);
                let next = builder.and(prev, inv);
                builder.set_csr(csr, next);
            }
            set_reg_if_needed(builder, i.rd, prev);
            builder.ret();
            true
        }
        Instruction::Csrrwi(i) => {
            let csr = csr_from_imm(i.imm);
            let zimm = zimm(builder, i.rs1);
            let prev = builder.get_csr(csr);
            builder.set_csr(csr, zimm);
            set_reg_if_needed(builder, i.rd, prev);
            builder.ret();
            true
        }
        Instruction::Csrrsi(i) => {
            let csr = csr_from_imm(i.imm);
            let zimm = zimm(builder, i.rs1);
            let prev = builder.get_csr(csr);
            if i.rs1 != 0 {
                let next = builder.or(prev, zimm);
                builder.set_csr(csr, next);
            }
            set_reg_if_needed(builder, i.rd, prev);
            builder.ret();
            true
        }
        Instruction::Csrrci(i) => {
            let csr = csr_from_imm(i.imm);
            let zimm = zimm(builder, i.rs1);
            let prev = builder.get_csr(csr);
            if i.rs1 != 0 {
                let all_ones = builder.const_i64(-1);
                let inv = builder.xor(zimm, all_ones);
                let next = builder.and(prev, inv);
                builder.set_csr(csr, next);
            }
            set_reg_if_needed(builder, i.rd, prev);
            builder.ret();
            true
        }
        _ => false,
    }
}

fn reg(builder: &mut IrBuilder, idx: u8) -> ValueId {
    builder.get_reg(reg_from_u8(idx))
}

fn set_reg_if_needed(builder: &mut IrBuilder, idx: u8, val: ValueId) {
    if idx == 0 {
        return;
    }
    builder.set_reg(reg_from_u8(idx), val);
}

fn zimm(builder: &mut IrBuilder, value: u8) -> ValueId {
    builder.const_i64((value & 0x1f) as i64)
}

fn csr_from_imm(imm: i32) -> u32 {
    (imm as u32) & 0xfff
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

#[cfg(test)]
mod tests {
    use super::lower_csr;
    use crate::decode::{Instruction, I};
    use crate::ir::execute_ir;
    use crate::trace::NoopTracer;
    use crate::{HostIO, VM};

    #[test]
    fn lower_csrrw_updates_csr_and_rd() {
        let insn = Instruction::Csrrw(I {
            rd: 5,
            rs1: 1,
            imm: 0x1,
        });

        let mut builder = crate::ir::IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);
        let seven = builder.const_i64(7);
        builder.set_reg(crate::ir::Reg::X1, seven);
        assert!(lower_csr(&insn, &mut builder));
        let func = builder.finish();

        let mut vm = VM::<NoopTracer>::init();
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.read_csr(0x1), 7);
        assert_eq!(vm.reg(5), 0);
    }

    #[test]
    fn lower_csrrs_no_write_when_rs1_zero() {
        let insn = Instruction::Csrrs(I {
            rd: 0,
            rs1: 0,
            imm: 0x2,
        });

        let mut builder = crate::ir::IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);
        let three = builder.const_i64(3);
        builder.set_csr(0x2, three);
        assert!(lower_csr(&insn, &mut builder));
        let func = builder.finish();

        let mut vm = VM::<NoopTracer>::init();
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.read_csr(0x2), 3);
    }

    #[test]
    fn lower_csrrci_no_write_when_zimm_zero() {
        let insn = Instruction::Csrrci(I {
            rd: 0,
            rs1: 0,
            imm: 0x3,
        });

        let mut builder = crate::ir::IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);
        let mask = builder.const_i64(0x1f);
        builder.set_csr(0x3, mask);
        assert!(lower_csr(&insn, &mut builder));
        let func = builder.finish();

        let mut vm = VM::<NoopTracer>::init();
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.read_csr(0x3), 0x1f);
    }
}
