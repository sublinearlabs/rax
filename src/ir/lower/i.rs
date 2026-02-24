use crate::decode::Instruction;
use crate::ir::lower::csr::lower_csr;
use crate::ir::{IrBuilder, IrFunction, IrType, ValueId};

pub(crate) fn lower_i(insn: &Instruction, current_pc: u64, next_pc: u64) -> IrFunction {
    let mut builder = IrBuilder::new();
    let entry = builder.block();
    builder.switch_to(entry);

    if lower_csr(insn, &mut builder) {
        return builder.finish();
    }

    match insn {
        // Integer Register-Register
        Instruction::Add(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.add(rs1, rs2);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Sub(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.sub(rs1, rs2);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::And(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.and(rs1, rs2);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Or(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.or(rs1, rs2);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Xor(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.xor(rs1, rs2);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Sll(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let sh = builder.shamt64(rs2);
            let v = builder.shl(rs1, sh);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Srl(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let sh = builder.shamt64(rs2);
            let v = builder.shr(rs1, sh);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Sra(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let sh = builder.shamt64(rs2);
            let v = builder.sar(rs1, sh);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Slt(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.lt(rs1, rs2);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Sltu(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.ltu(rs1, rs2);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }

        // Integer Register-Immediate
        Instruction::Addi(i) => {
            let rs1 = builder.reg(i.rs1);
            let imm = builder.imm_i32(i.imm);
            let v = builder.add(rs1, imm);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }
        Instruction::Andi(i) => {
            let rs1 = builder.reg(i.rs1);
            let imm = builder.imm_i32(i.imm);
            let v = builder.and(rs1, imm);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }
        Instruction::Ori(i) => {
            let rs1 = builder.reg(i.rs1);
            let imm = builder.imm_i32(i.imm);
            let v = builder.or(rs1, imm);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }
        Instruction::Xori(i) => {
            let rs1 = builder.reg(i.rs1);
            let imm = builder.imm_i32(i.imm);
            let v = builder.xor(rs1, imm);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }
        Instruction::Slti(i) => {
            let rs1 = builder.reg(i.rs1);
            let imm = builder.imm_i32(i.imm);
            let v = builder.lt(rs1, imm);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }
        Instruction::Sltiu(i) => {
            let rs1 = builder.reg(i.rs1);
            let imm = builder.imm_i32(i.imm);
            let v = builder.ltu(rs1, imm);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }
        Instruction::Slli(sh) => {
            let shamt = builder.imm_u8(sh.shamt & 0x3f);
            let rs1 = builder.reg(sh.rs1);
            let v = builder.shl(rs1, shamt);
            builder.set_reg_idx(sh.rd, v);
            builder.ret();
        }
        Instruction::Srli(sh) => {
            let shamt = builder.imm_u8(sh.shamt & 0x3f);
            let rs1 = builder.reg(sh.rs1);
            let v = builder.shr(rs1, shamt);
            builder.set_reg_idx(sh.rd, v);
            builder.ret();
        }
        Instruction::Srai(sh) => {
            let shamt = builder.imm_u8(sh.shamt & 0x3f);
            let rs1 = builder.reg(sh.rs1);
            let v = builder.sar(rs1, shamt);
            builder.set_reg_idx(sh.rd, v);
            builder.ret();
        }

        // Loads
        Instruction::Lb(i) => {
            let addr = builder.addr(i.rs1, i.imm);
            let v = builder.load8s(addr);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }
        Instruction::Lbu(i) => {
            let addr = builder.addr(i.rs1, i.imm);
            let v = builder.load8u(addr);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }
        Instruction::Lh(i) => {
            let addr = builder.addr(i.rs1, i.imm);
            let v = builder.load16s(addr);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }
        Instruction::Lhu(i) => {
            let addr = builder.addr(i.rs1, i.imm);
            let v = builder.load16u(addr);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }
        Instruction::Lw(i) => {
            let addr = builder.addr(i.rs1, i.imm);
            let v = builder.load32s(addr);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }
        Instruction::Lwu(i) => {
            let addr = builder.addr(i.rs1, i.imm);
            let v = builder.load32u(addr);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }
        Instruction::Ld(i) => {
            let addr = builder.addr(i.rs1, i.imm);
            let v = builder.load64(addr);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }

        // Stores
        Instruction::Sb(s) => {
            let addr = builder.addr(s.rs1, s.imm);
            let val = builder.reg(s.rs2);
            builder.store8(addr, val);
            builder.ret();
        }
        Instruction::Sh(s) => {
            let addr = builder.addr(s.rs1, s.imm);
            let val = builder.reg(s.rs2);
            builder.store16(addr, val);
            builder.ret();
        }
        Instruction::Sw(s) => {
            let addr = builder.addr(s.rs1, s.imm);
            let val = builder.reg(s.rs2);
            builder.store32(addr, val);
            builder.ret();
        }
        Instruction::Sd(s) => {
            let addr = builder.addr(s.rs1, s.imm);
            let val = builder.reg(s.rs2);
            builder.store64(addr, val);
            builder.ret();
        }

        // Branches
        Instruction::Beq(b) => {
            let rs1 = builder.reg(b.rs1);
            let rs2 = builder.reg(b.rs2);
            let cond = builder.eq(rs1, rs2);
            lower_branch(&mut builder, cond, current_pc, b.imm);
        }
        Instruction::Bne(b) => {
            let rs1 = builder.reg(b.rs1);
            let rs2 = builder.reg(b.rs2);
            let cond = builder.ne(rs1, rs2);
            lower_branch(&mut builder, cond, current_pc, b.imm);
        }
        Instruction::Blt(b) => {
            let rs1 = builder.reg(b.rs1);
            let rs2 = builder.reg(b.rs2);
            let cond = builder.lt(rs1, rs2);
            lower_branch(&mut builder, cond, current_pc, b.imm);
        }
        Instruction::Bltu(b) => {
            let rs1 = builder.reg(b.rs1);
            let rs2 = builder.reg(b.rs2);
            let cond = builder.ltu(rs1, rs2);
            lower_branch(&mut builder, cond, current_pc, b.imm);
        }
        Instruction::Bge(b) => {
            let rs1 = builder.reg(b.rs1);
            let rs2 = builder.reg(b.rs2);
            let cond = builder.ge(rs1, rs2);
            lower_branch(&mut builder, cond, current_pc, b.imm);
        }
        Instruction::Bgeu(b) => {
            let rs1 = builder.reg(b.rs1);
            let rs2 = builder.reg(b.rs2);
            let cond = builder.geu(rs1, rs2);
            lower_branch(&mut builder, cond, current_pc, b.imm);
        }

        // Jumps
        Instruction::Jal(j) => {
            let link = builder.imm_u64(next_pc);
            builder.set_reg_idx(j.rd, link);
            let target = builder.pc_plus(current_pc, j.imm);
            builder.set_pc(target);
            builder.ret();
        }
        Instruction::Jalr(i) => {
            let link = builder.imm_u64(next_pc);
            builder.set_reg_idx(i.rd, link);
            let target = builder.addr(i.rs1, i.imm);
            let mask = builder.imm_i32(-2);
            let masked = builder.and(target, mask);
            builder.set_pc(masked);
            builder.ret();
        }

        // Upper immediates
        Instruction::Lui(u) => {
            let imm = builder.imm_i32(u.imm);
            builder.set_reg_idx(u.rd, imm);
            builder.ret();
        }
        Instruction::Auipc(u) => {
            let v = builder.pc_plus(current_pc, u.imm);
            builder.set_reg_idx(u.rd, v);
            builder.ret();
        }

        // RV64I word ops
        Instruction::Addiw(i) => {
            let rs1 = builder.reg(i.rs1);
            let imm = builder.imm_i32(i.imm);
            let sum = builder.add(rs1, imm);
            let trunc = trunc_i32(&mut builder, sum);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(i.rd, v);
            builder.ret();
        }
        Instruction::Slliw(sh) => {
            let shamt = builder.imm_u8(sh.shamt & 0x1f);
            let rs1 = builder.reg(sh.rs1);
            let shifted = builder.shl(rs1, shamt);
            let trunc = trunc_i32(&mut builder, shifted);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(sh.rd, v);
            builder.ret();
        }
        Instruction::Srliw(sh) => {
            let shamt = builder.imm_u8(sh.shamt & 0x1f);
            let rs1 = builder.reg(sh.rs1);
            let shifted = builder.shr(rs1, shamt);
            let trunc = trunc_i32(&mut builder, shifted);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(sh.rd, v);
            builder.ret();
        }
        Instruction::Sraiw(sh) => {
            let shamt = builder.imm_u8(sh.shamt & 0x1f);
            let rs1 = builder.reg(sh.rs1);
            let shifted = builder.sar(rs1, shamt);
            let trunc = trunc_i32(&mut builder, shifted);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(sh.rd, v);
            builder.ret();
        }
        Instruction::Addw(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let sum = builder.add(rs1, rs2);
            let trunc = trunc_i32(&mut builder, sum);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Subw(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let diff = builder.sub(rs1, rs2);
            let trunc = trunc_i32(&mut builder, diff);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Sllw(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let sh = builder.shamt32(rs2);
            let shifted = builder.shl(rs1, sh);
            let trunc = trunc_i32(&mut builder, shifted);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Srlw(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let sh = builder.shamt32(rs2);
            let shifted = builder.shr(rs1, sh);
            let trunc = trunc_i32(&mut builder, shifted);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Sraw(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let sh = builder.shamt32(rs2);
            let shifted = builder.sar(rs1, sh);
            let trunc = trunc_i32(&mut builder, shifted);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }

        // System
        Instruction::Ecall => {
            builder.ecall();
            builder.ret();
        }
        Instruction::Ebreak => {
            builder.ebreak();
            builder.ret();
        }
        Instruction::Nop
        | Instruction::Mret
        | Instruction::Sret
        | Instruction::Uret
        | Instruction::Wfi
        | Instruction::SfenceVma
        | Instruction::Fence => {
            builder.ret();
        }

        _ => panic!("IR lowering missing for {:?}", insn),
    }

    builder.finish()
}

fn lower_branch(builder: &mut IrBuilder, cond: ValueId, current_pc: u64, imm: i32) {
    let taken = builder.block();
    let fallthrough = builder.block();

    builder.cbr(cond, taken, fallthrough, vec![], vec![]);

    builder.switch_to(taken);
    let target = builder.pc_plus(current_pc, imm);
    builder.set_pc(target);
    builder.ret();

    builder.switch_to(fallthrough);
    builder.ret();
}

fn trunc_i32(builder: &mut IrBuilder, value: ValueId) -> ValueId {
    builder.trunc(value, IrType::I64, IrType::I32)
}

fn sext_i32(builder: &mut IrBuilder, value: ValueId) -> ValueId {
    builder.sext(value, IrType::I32, IrType::I64)
}

#[cfg(test)]
mod tests {
    use super::lower_i;
    use crate::decode::{Instruction, B, I};
    use crate::ir::execute_ir;
    use crate::trace::NoopTracer;
    use crate::{HostIO, VM};

    #[test]
    fn lower_addi_sets_register() {
        let insn = Instruction::Addi(I {
            rd: 1,
            rs1: 0,
            imm: 7,
        });
        let func = lower_i(&insn, 0, 4);

        let mut vm = VM::<NoopTracer>::init();
        vm.set_pc(4);
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.reg(1), 7);
    }

    #[test]
    fn lower_beq_updates_pc() {
        let insn = Instruction::Beq(B {
            rs1: 1,
            rs2: 2,
            imm: 12,
        });
        let func = lower_i(&insn, 100, 104);

        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(1, 5);
        vm.reg_mut(2, 5);
        vm.set_pc(104);
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.pc(), 112);
    }
}
