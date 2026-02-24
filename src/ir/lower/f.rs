use crate::decode::{Instruction, RF};
use crate::ir::{IrBuilder, IrFunction, IrType};

pub(crate) fn lower_f(insn: &Instruction, _current_pc: u64, _next_pc: u64) -> IrFunction {
    let mut builder = IrBuilder::new();
    let entry = builder.block();
    builder.switch_to(entry);

    match insn {
        // Loads
        Instruction::Flw(i) => {
            let base = builder.reg(i.rs1);
            let offset = builder.iconst(crate::util::sext(i.imm as u64, 12) as i64);
            let addr = builder.add(base, offset);
            let value = builder.load_f32(addr);
            builder.set_freg_idx(i.rd, value);
            builder.ret();
        }

        // Stores
        Instruction::Fsw(s) => {
            let base = builder.reg(s.rs1);
            let offset = builder.iconst(crate::util::sext(s.imm as u64, 12) as i64);
            let addr = builder.add(base, offset);
            let value = builder.freg(s.rs2);
            builder.store_f32(addr, value);
            builder.ret();
        }

        // Arithmetic
        Instruction::FaddS(r) => {
            let rs1 = builder.freg(r.rs1);
            let rs2 = builder.freg(r.rs2);
            let v = builder.fadd(rs1, rs2);
            builder.set_freg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::FsubS(r) => {
            let rs1 = builder.freg(r.rs1);
            let rs2 = builder.freg(r.rs2);
            let v = builder.fsub(rs1, rs2);
            builder.set_freg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::FmulS(r) => {
            let rs1 = builder.freg(r.rs1);
            let rs2 = builder.freg(r.rs2);
            let v = builder.fmul(rs1, rs2);
            builder.set_freg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::FdivS(r) => {
            let rs1 = builder.freg(r.rs1);
            let rs2 = builder.freg(r.rs2);
            let v = builder.fdiv(rs1, rs2);
            builder.set_freg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::FsqrtS(r) => {
            let rs1 = builder.freg(r.rs1);
            let v = builder.fsqrt(rs1);
            builder.set_freg_idx(r.rd, v);
            builder.ret();
        }

        // Sign manipulation
        Instruction::FsgnjS(r) => {
            let rs1 = builder.freg(r.rs1);
            let rs2 = builder.freg(r.rs2);
            let v = builder.fsgnj(rs1, rs2);
            builder.set_freg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::FsgnjnS(r) => {
            let rs1 = builder.freg(r.rs1);
            let rs2 = builder.freg(r.rs2);
            let v = builder.fsgnjn(rs1, rs2);
            builder.set_freg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::FsgnjxS(r) => {
            let rs1 = builder.freg(r.rs1);
            let rs2 = builder.freg(r.rs2);
            let v = builder.fsgnjx(rs1, rs2);
            builder.set_freg_idx(r.rd, v);
            builder.ret();
        }

        // Min/Max
        Instruction::FminS(r) => {
            let rs1 = builder.freg(r.rs1);
            let rs2 = builder.freg(r.rs2);
            let v = builder.fmin(rs1, rs2);
            builder.set_freg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::FmaxS(r) => {
            let rs1 = builder.freg(r.rs1);
            let rs2 = builder.freg(r.rs2);
            let v = builder.fmax(rs1, rs2);
            builder.set_freg_idx(r.rd, v);
            builder.ret();
        }

        // Comparisons (set integer register)
        Instruction::FeqS(r) => {
            let rs1 = builder.freg(r.rs1);
            let rs2 = builder.freg(r.rs2);
            let v = builder.feq(rs1, rs2);
            let v = builder.zext(v, IrType::I1, IrType::I64);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::FltS(r) => {
            let rs1 = builder.freg(r.rs1);
            let rs2 = builder.freg(r.rs2);
            let v = builder.flt(rs1, rs2);
            let v = builder.zext(v, IrType::I1, IrType::I64);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::FleS(r) => {
            let rs1 = builder.freg(r.rs1);
            let rs2 = builder.freg(r.rs2);
            let v = builder.fle(rs1, rs2);
            let v = builder.zext(v, IrType::I1, IrType::I64);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }

        // Conversions
        Instruction::FcvtWS(r) => {
            let rs1 = builder.freg(r.rs1);
            let v = builder.fcvt_w_s(rs1);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::FcvtWuS(r) => {
            let rs1 = builder.freg(r.rs1);
            let v = builder.fcvt_wu_s(rs1);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::FcvtSW(r) => {
            let rs1 = builder.reg(r.rs1);
            let v = builder.fcvt_s_w(rs1);
            builder.set_freg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::FcvtSWu(r) => {
            let rs1 = builder.reg(r.rs1);
            let v = builder.fcvt_s_wu(rs1);
            builder.set_freg_idx(r.rd, v);
            builder.ret();
        }

        // Fused multiply-add
        Instruction::FmaddS(r4) => {
            let rs1 = builder.freg(r4.rs1);
            let rs2 = builder.freg(r4.rs2);
            let rs3 = builder.freg(r4.rs3);
            let v = builder.fmadd(rs1, rs2, rs3);
            builder.set_freg_idx(r4.rd, v);
            builder.ret();
        }
        Instruction::FmsubS(r4) => {
            let rs1 = builder.freg(r4.rs1);
            let rs2 = builder.freg(r4.rs2);
            let rs3 = builder.freg(r4.rs3);
            let v = builder.fmsub(rs1, rs2, rs3);
            builder.set_freg_idx(r4.rd, v);
            builder.ret();
        }
        Instruction::FnmsubS(r4) => {
            let rs1 = builder.freg(r4.rs1);
            let rs2 = builder.freg(r4.rs2);
            let rs3 = builder.freg(r4.rs3);
            let v = builder.fnmsub(rs1, rs2, rs3);
            builder.set_freg_idx(r4.rd, v);
            builder.ret();
        }
        Instruction::FnmaddS(r4) => {
            let rs1 = builder.freg(r4.rs1);
            let rs2 = builder.freg(r4.rs2);
            let rs3 = builder.freg(r4.rs3);
            let v = builder.fnmadd(rs1, rs2, rs3);
            builder.set_freg_idx(r4.rd, v);
            builder.ret();
        }

        _ => panic!("IR lowering missing for F instruction {:?}", insn),
    }

    builder.finish()
}

#[cfg(test)]
mod tests {
    use super::lower_f;
    use crate::decode::{Instruction, RF};
    use crate::ir::execute_ir;
    use crate::trace::NoopTracer;
    use crate::{HostIO, VM};

    #[test]
    fn lower_fadd_s_sets_register() {
        let insn = Instruction::FaddS(RF {
            rd: 3,
            rs1: 1,
            rs2: 2,
            rm: 0,
        });
        let func = lower_f(&insn, 0, 4);

        let mut vm = VM::<NoopTracer>::init();
        vm.write_f32(1, 1.5);
        vm.write_f32(2, 2.5);
        vm.set_pc(4);
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.read_f32(3), 4.0); // 1.5 + 2.5
    }

    #[test]
    fn lower_feq_s_sets_register() {
        let insn = Instruction::FeqS(RF {
            rd: 3,
            rs1: 1,
            rs2: 2,
            rm: 0,
        });
        let func = lower_f(&insn, 0, 4);

        let mut vm = VM::<NoopTracer>::init();
        vm.write_f32(1, 1.0);
        vm.write_f32(2, 1.0);
        vm.set_pc(4);
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.reg(3), 1); // Equal
    }
}
