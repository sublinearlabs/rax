use crate::decode::Instruction;
use crate::ir::IrBuilder;

fn finish_insn(builder: &mut IrBuilder, terminate: bool) {
    if terminate {
        builder.ret();
    }
}

pub(crate) fn lower_csr(insn: &Instruction, builder: &mut IrBuilder, terminate: bool) -> bool {
    match insn {
        Instruction::Csrrw(i) => {
            let csr = csr_from_imm(i.imm);
            let rs1 = builder.reg(i.rs1);
            let prev = builder.get_csr(csr);
            builder.set_csr(csr, rs1);
            builder.set_reg_if_needed(i.rd, prev);
            finish_insn(builder, terminate);
            true
        }
        Instruction::Csrrs(i) => {
            let csr = csr_from_imm(i.imm);
            let rs1 = builder.reg(i.rs1);
            let prev = builder.get_csr(csr);
            if i.rs1 != 0 {
                let next = builder.or(prev, rs1);
                builder.set_csr(csr, next);
            }
            builder.set_reg_if_needed(i.rd, prev);
            finish_insn(builder, terminate);
            true
        }
        Instruction::Csrrc(i) => {
            let csr = csr_from_imm(i.imm);
            let rs1 = builder.reg(i.rs1);
            let prev = builder.get_csr(csr);
            if i.rs1 != 0 {
                let inv = builder.not(rs1);
                let next = builder.and(prev, inv);
                builder.set_csr(csr, next);
            }
            builder.set_reg_if_needed(i.rd, prev);
            finish_insn(builder, terminate);
            true
        }
        Instruction::Csrrwi(i) => {
            let csr = csr_from_imm(i.imm);
            let zimm = builder.zimm5(i.rs1);
            let prev = builder.get_csr(csr);
            builder.set_csr(csr, zimm);
            builder.set_reg_if_needed(i.rd, prev);
            finish_insn(builder, terminate);
            true
        }
        Instruction::Csrrsi(i) => {
            let csr = csr_from_imm(i.imm);
            let zimm = builder.zimm5(i.rs1);
            let prev = builder.get_csr(csr);
            if i.rs1 != 0 {
                let next = builder.or(prev, zimm);
                builder.set_csr(csr, next);
            }
            builder.set_reg_if_needed(i.rd, prev);
            finish_insn(builder, terminate);
            true
        }
        Instruction::Csrrci(i) => {
            let csr = csr_from_imm(i.imm);
            let zimm = builder.zimm5(i.rs1);
            let prev = builder.get_csr(csr);
            if i.rs1 != 0 {
                let inv = builder.not(zimm);
                let next = builder.and(prev, inv);
                builder.set_csr(csr, next);
            }
            builder.set_reg_if_needed(i.rd, prev);
            finish_insn(builder, terminate);
            true
        }
        _ => false,
    }
}

fn csr_from_imm(imm: i32) -> u32 {
    (imm as u32) & 0xfff
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
        assert!(lower_csr(&insn, &mut builder, true));
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
        assert!(lower_csr(&insn, &mut builder, true));
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
        assert!(lower_csr(&insn, &mut builder, true));
        let func = builder.finish();

        let mut vm = VM::<NoopTracer>::init();
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.read_csr(0x3), 0x1f);
    }
}
