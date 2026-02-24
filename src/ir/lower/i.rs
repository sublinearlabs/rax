use crate::decode::Instruction;
use crate::ir::{IrBuilder, IrFunction, IrType, Reg, ValueId};

pub fn lower_i(insn: &Instruction, current_pc: u64, next_pc: u64) -> IrFunction {
    let mut builder = IrBuilder::new();
    let entry = builder.block();
    builder.switch_to(entry);

    match insn {
        // Integer Register-Register
        Instruction::Add(r) => {
            let v = builder.add(reg(&mut builder, r.rs1), reg(&mut builder, r.rs2));
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }
        Instruction::Sub(r) => {
            let v = builder.sub(reg(&mut builder, r.rs1), reg(&mut builder, r.rs2));
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }
        Instruction::And(r) => {
            let v = builder.and(reg(&mut builder, r.rs1), reg(&mut builder, r.rs2));
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }
        Instruction::Or(r) => {
            let v = builder.or(reg(&mut builder, r.rs1), reg(&mut builder, r.rs2));
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }
        Instruction::Xor(r) => {
            let v = builder.xor(reg(&mut builder, r.rs1), reg(&mut builder, r.rs2));
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }
        Instruction::Sll(r) => {
            let sh = mask_shamt64(&mut builder, reg(&mut builder, r.rs2));
            let v = builder.shl(reg(&mut builder, r.rs1), sh);
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }
        Instruction::Srl(r) => {
            let sh = mask_shamt64(&mut builder, reg(&mut builder, r.rs2));
            let v = builder.shr(reg(&mut builder, r.rs1), sh);
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }
        Instruction::Sra(r) => {
            let sh = mask_shamt64(&mut builder, reg(&mut builder, r.rs2));
            let v = builder.sar(reg(&mut builder, r.rs1), sh);
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }
        Instruction::Slt(r) => {
            let v = builder.lt(reg(&mut builder, r.rs1), reg(&mut builder, r.rs2));
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }
        Instruction::Sltu(r) => {
            let v = builder.ltu(reg(&mut builder, r.rs1), reg(&mut builder, r.rs2));
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }

        // Integer Register-Immediate
        Instruction::Addi(i) => {
            let v = builder.add(reg(&mut builder, i.rs1), imm(&mut builder, i.imm));
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }
        Instruction::Andi(i) => {
            let v = builder.and(reg(&mut builder, i.rs1), imm(&mut builder, i.imm));
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }
        Instruction::Ori(i) => {
            let v = builder.or(reg(&mut builder, i.rs1), imm(&mut builder, i.imm));
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }
        Instruction::Xori(i) => {
            let v = builder.xor(reg(&mut builder, i.rs1), imm(&mut builder, i.imm));
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }
        Instruction::Slti(i) => {
            let v = builder.lt(reg(&mut builder, i.rs1), imm(&mut builder, i.imm));
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }
        Instruction::Sltiu(i) => {
            let v = builder.ltu(reg(&mut builder, i.rs1), imm(&mut builder, i.imm));
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }
        Instruction::Slli(sh) => {
            let shamt = imm_u8(&mut builder, sh.shamt & 0x3f);
            let v = builder.shl(reg(&mut builder, sh.rs1), shamt);
            set_reg(&mut builder, sh.rd, v);
            builder.ret();
        }
        Instruction::Srli(sh) => {
            let shamt = imm_u8(&mut builder, sh.shamt & 0x3f);
            let v = builder.shr(reg(&mut builder, sh.rs1), shamt);
            set_reg(&mut builder, sh.rd, v);
            builder.ret();
        }
        Instruction::Srai(sh) => {
            let shamt = imm_u8(&mut builder, sh.shamt & 0x3f);
            let v = builder.sar(reg(&mut builder, sh.rs1), shamt);
            set_reg(&mut builder, sh.rd, v);
            builder.ret();
        }

        // Loads
        Instruction::Lb(i) => {
            let addr = addr(&mut builder, i.rs1, i.imm);
            let v = builder.load8s(addr);
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }
        Instruction::Lbu(i) => {
            let addr = addr(&mut builder, i.rs1, i.imm);
            let v = builder.load8u(addr);
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }
        Instruction::Lh(i) => {
            let addr = addr(&mut builder, i.rs1, i.imm);
            let v = builder.load16s(addr);
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }
        Instruction::Lhu(i) => {
            let addr = addr(&mut builder, i.rs1, i.imm);
            let v = builder.load16u(addr);
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }
        Instruction::Lw(i) => {
            let addr = addr(&mut builder, i.rs1, i.imm);
            let v = builder.load32s(addr);
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }
        Instruction::Lwu(i) => {
            let addr = addr(&mut builder, i.rs1, i.imm);
            let v = builder.load32u(addr);
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }
        Instruction::Ld(i) => {
            let addr = addr(&mut builder, i.rs1, i.imm);
            let v = builder.load64(addr);
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }

        // Stores
        Instruction::Sb(s) => {
            let addr = addr(&mut builder, s.rs1, s.imm);
            builder.store8(addr, reg(&mut builder, s.rs2));
            builder.ret();
        }
        Instruction::Sh(s) => {
            let addr = addr(&mut builder, s.rs1, s.imm);
            builder.store16(addr, reg(&mut builder, s.rs2));
            builder.ret();
        }
        Instruction::Sw(s) => {
            let addr = addr(&mut builder, s.rs1, s.imm);
            builder.store32(addr, reg(&mut builder, s.rs2));
            builder.ret();
        }
        Instruction::Sd(s) => {
            let addr = addr(&mut builder, s.rs1, s.imm);
            builder.store64(addr, reg(&mut builder, s.rs2));
            builder.ret();
        }

        // Branches
        Instruction::Beq(b) => {
            let cond = builder.eq(reg(&mut builder, b.rs1), reg(&mut builder, b.rs2));
            lower_branch(&mut builder, cond, current_pc, b.imm);
        }
        Instruction::Bne(b) => {
            let cond = builder.ne(reg(&mut builder, b.rs1), reg(&mut builder, b.rs2));
            lower_branch(&mut builder, cond, current_pc, b.imm);
        }
        Instruction::Blt(b) => {
            let cond = builder.lt(reg(&mut builder, b.rs1), reg(&mut builder, b.rs2));
            lower_branch(&mut builder, cond, current_pc, b.imm);
        }
        Instruction::Bltu(b) => {
            let cond = builder.ltu(reg(&mut builder, b.rs1), reg(&mut builder, b.rs2));
            lower_branch(&mut builder, cond, current_pc, b.imm);
        }
        Instruction::Bge(b) => {
            let cond = builder.ge(reg(&mut builder, b.rs1), reg(&mut builder, b.rs2));
            lower_branch(&mut builder, cond, current_pc, b.imm);
        }
        Instruction::Bgeu(b) => {
            let cond = builder.geu(reg(&mut builder, b.rs1), reg(&mut builder, b.rs2));
            lower_branch(&mut builder, cond, current_pc, b.imm);
        }

        // Jumps
        Instruction::Jal(j) => {
            set_reg(&mut builder, j.rd, const_u64(&mut builder, next_pc));
            let target = add_pc_imm(&mut builder, current_pc, j.imm);
            builder.set_pc(target);
            builder.ret();
        }
        Instruction::Jalr(i) => {
            set_reg(&mut builder, i.rd, const_u64(&mut builder, next_pc));
            let target = addr(&mut builder, i.rs1, i.imm);
            let masked = builder.and(target, imm(&mut builder, -2));
            builder.set_pc(masked);
            builder.ret();
        }

        // Upper immediates
        Instruction::Lui(u) => {
            set_reg(&mut builder, u.rd, imm(&mut builder, u.imm));
            builder.ret();
        }
        Instruction::Auipc(u) => {
            let v = add_pc_imm(&mut builder, current_pc, u.imm);
            set_reg(&mut builder, u.rd, v);
            builder.ret();
        }

        // RV64I word ops
        Instruction::Addiw(i) => {
            let sum = builder.add(reg(&mut builder, i.rs1), imm(&mut builder, i.imm));
            let v = sext_i32(&mut builder, trunc_i32(&mut builder, sum));
            set_reg(&mut builder, i.rd, v);
            builder.ret();
        }
        Instruction::Slliw(sh) => {
            let shamt = imm_u8(&mut builder, sh.shamt & 0x1f);
            let shifted = builder.shl(reg(&mut builder, sh.rs1), shamt);
            let v = sext_i32(&mut builder, trunc_i32(&mut builder, shifted));
            set_reg(&mut builder, sh.rd, v);
            builder.ret();
        }
        Instruction::Srliw(sh) => {
            let shamt = imm_u8(&mut builder, sh.shamt & 0x1f);
            let shifted = builder.shr(reg(&mut builder, sh.rs1), shamt);
            let v = sext_i32(&mut builder, trunc_i32(&mut builder, shifted));
            set_reg(&mut builder, sh.rd, v);
            builder.ret();
        }
        Instruction::Sraiw(sh) => {
            let shamt = imm_u8(&mut builder, sh.shamt & 0x1f);
            let shifted = builder.sar(reg(&mut builder, sh.rs1), shamt);
            let v = sext_i32(&mut builder, trunc_i32(&mut builder, shifted));
            set_reg(&mut builder, sh.rd, v);
            builder.ret();
        }
        Instruction::Addw(r) => {
            let sum = builder.add(reg(&mut builder, r.rs1), reg(&mut builder, r.rs2));
            let v = sext_i32(&mut builder, trunc_i32(&mut builder, sum));
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }
        Instruction::Subw(r) => {
            let diff = builder.sub(reg(&mut builder, r.rs1), reg(&mut builder, r.rs2));
            let v = sext_i32(&mut builder, trunc_i32(&mut builder, diff));
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }
        Instruction::Sllw(r) => {
            let sh = mask_shamt32(&mut builder, reg(&mut builder, r.rs2));
            let shifted = builder.shl(reg(&mut builder, r.rs1), sh);
            let v = sext_i32(&mut builder, trunc_i32(&mut builder, shifted));
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }
        Instruction::Srlw(r) => {
            let sh = mask_shamt32(&mut builder, reg(&mut builder, r.rs2));
            let shifted = builder.shr(reg(&mut builder, r.rs1), sh);
            let v = sext_i32(&mut builder, trunc_i32(&mut builder, shifted));
            set_reg(&mut builder, r.rd, v);
            builder.ret();
        }
        Instruction::Sraw(r) => {
            let sh = mask_shamt32(&mut builder, reg(&mut builder, r.rs2));
            let shifted = builder.sar(reg(&mut builder, r.rs1), sh);
            let v = sext_i32(&mut builder, trunc_i32(&mut builder, shifted));
            set_reg(&mut builder, r.rd, v);
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

        _ => panic!("IR lowering missing for {:?}", insn),
    }

    builder.finish()
}

fn reg(builder: &mut IrBuilder, idx: u8) -> ValueId {
    builder.get_reg(reg_from_u8(idx))
}

fn set_reg(builder: &mut IrBuilder, idx: u8, val: ValueId) {
    builder.set_reg(reg_from_u8(idx), val);
}

fn imm(builder: &mut IrBuilder, value: i32) -> ValueId {
    builder.const_i64(value as i64)
}

fn imm_u8(builder: &mut IrBuilder, value: u8) -> ValueId {
    builder.const_i64(value as i64)
}

fn const_u64(builder: &mut IrBuilder, value: u64) -> ValueId {
    builder.const_i64(value as i64)
}

fn addr(builder: &mut IrBuilder, rs1: u8, offset: i32) -> ValueId {
    let base = reg(builder, rs1);
    let off = imm(builder, offset);
    builder.add(base, off)
}

fn add_pc_imm(builder: &mut IrBuilder, current_pc: u64, imm: i32) -> ValueId {
    let base = const_u64(builder, current_pc);
    let off = imm(builder, imm);
    builder.add(base, off)
}

fn lower_branch(builder: &mut IrBuilder, cond: ValueId, current_pc: u64, imm: i32) {
    let taken = builder.block();
    let fallthrough = builder.block();

    builder.cbr(cond, taken, fallthrough, vec![], vec![]);

    builder.switch_to(taken);
    let target = add_pc_imm(builder, current_pc, imm);
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

fn mask_shamt64(builder: &mut IrBuilder, value: ValueId) -> ValueId {
    builder.and(value, builder.const_i64(0x3f))
}

fn mask_shamt32(builder: &mut IrBuilder, value: ValueId) -> ValueId {
    builder.and(value, builder.const_i64(0x1f))
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
    use super::lower_i;
    use crate::decode::insn_formats::{B, I};
    use crate::decode::Instruction;
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
