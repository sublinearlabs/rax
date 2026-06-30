use riscv_core::decode::Instruction;
use crate::ir::{IrBuilder, IrType, MemWidth, Reg, ValueId};

pub(crate) fn lower_i_into(
    insn: &Instruction,
    current_pc: u64,
    next_pc: u64,
    builder: &mut IrBuilder,
) {
    match insn {
        // Integer Register-Register
        Instruction::Add(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let v = builder.add(rs1, rs2, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Sub(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let v = builder.sub(rs1, rs2, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::And(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let v = builder.and(rs1, rs2, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Or(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let v = builder.or(rs1, rs2, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Xor(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let v = builder.xor(rs1, rs2, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Sll(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let mask = builder.const_i64(0x3f);
            let sh = builder.and(rs2, mask, IrType::I64);
            let v = builder.shl(rs1, sh, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Srl(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let mask = builder.const_i64(0x3f);
            let sh = builder.and(rs2, mask, IrType::I64);
            let v = builder.shr(rs1, sh, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Sra(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let mask = builder.const_i64(0x3f);
            let sh = builder.and(rs2, mask, IrType::I64);
            let v = builder.sar(rs1, sh, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Slt(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let v = builder.lt(rs1, rs2);
            let v = builder.zext(v, IrType::I1, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Sltu(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let v = builder.ltu(rs1, rs2);
            let v = builder.zext(v, IrType::I1, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }

        // Integer Register-Immediate
        Instruction::Addi(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let v = builder.add(rs1, imm, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }
        Instruction::Andi(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let v = builder.and(rs1, imm, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }
        Instruction::Ori(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let v = builder.or(rs1, imm, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }
        Instruction::Xori(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let v = builder.xor(rs1, imm, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }
        Instruction::Slli(sh) => {
            let rs1 = builder.get_reg(reg_from_u8(sh.rs1));
            let imm = builder.const_i64(sh.shamt as i64);
            let v = builder.shl(rs1, imm, IrType::I64);
            builder.set_reg(reg_from_u8(sh.rd), v);
        }
        Instruction::Srli(sh) => {
            let rs1 = builder.get_reg(reg_from_u8(sh.rs1));
            let imm = builder.const_i64(sh.shamt as i64);
            let v = builder.shr(rs1, imm, IrType::I64);
            builder.set_reg(reg_from_u8(sh.rd), v);
        }
        Instruction::Srai(sh) => {
            let rs1 = builder.get_reg(reg_from_u8(sh.rs1));
            let imm = builder.const_i64(sh.shamt as i64);
            let v = builder.sar(rs1, imm, IrType::I64);
            builder.set_reg(reg_from_u8(sh.rd), v);
        }
        Instruction::Slti(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let v = builder.lt(rs1, imm);
            let v = builder.zext(v, IrType::I1, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }
        Instruction::Sltiu(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let v = builder.ltu(rs1, imm);
            let v = builder.zext(v, IrType::I1, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }

        // Loads
        Instruction::Lb(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let addr = builder.add(rs1, imm, IrType::I64);
            let raw = builder.load(addr, MemWidth::W8, IrType::I8);
            let v = builder.sext(raw, IrType::I8, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }
        Instruction::Lbu(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let addr = builder.add(rs1, imm, IrType::I64);
            let raw = builder.load(addr, MemWidth::W8, IrType::I8);
            let v = builder.zext(raw, IrType::I8, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }
        Instruction::Lh(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let addr = builder.add(rs1, imm, IrType::I64);
            let raw = builder.load(addr, MemWidth::W16, IrType::I16);
            let v = builder.sext(raw, IrType::I16, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }
        Instruction::Lhu(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let addr = builder.add(rs1, imm, IrType::I64);
            let raw = builder.load(addr, MemWidth::W16, IrType::I16);
            let v = builder.zext(raw, IrType::I16, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }
        Instruction::Lw(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let addr = builder.add(rs1, imm, IrType::I64);
            let raw = builder.load(addr, MemWidth::W32, IrType::I32);
            let v = builder.sext(raw, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }
        Instruction::Lwu(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let addr = builder.add(rs1, imm, IrType::I64);
            let raw = builder.load(addr, MemWidth::W32, IrType::I32);
            let v = builder.zext(raw, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }
        Instruction::Ld(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let addr = builder.add(rs1, imm, IrType::I64);
            let v = builder.load(addr, MemWidth::W64, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }

        // Stores
        Instruction::Sb(s) => {
            let rs1 = builder.get_reg(reg_from_u8(s.rs1));
            let rs2 = builder.get_reg(reg_from_u8(s.rs2));
            let imm = builder.const_i64(s.imm as i64);
            let addr = builder.add(rs1, imm, IrType::I64);
            let v = builder.trunc(rs2, IrType::I64, IrType::I8);
            builder.store(addr, v, MemWidth::W8);
        }
        Instruction::Sh(s) => {
            let rs1 = builder.get_reg(reg_from_u8(s.rs1));
            let rs2 = builder.get_reg(reg_from_u8(s.rs2));
            let imm = builder.const_i64(s.imm as i64);
            let addr = builder.add(rs1, imm, IrType::I64);
            let v = builder.trunc(rs2, IrType::I64, IrType::I16);
            builder.store(addr, v, MemWidth::W16);
        }
        Instruction::Sw(s) => {
            let rs1 = builder.get_reg(reg_from_u8(s.rs1));
            let rs2 = builder.get_reg(reg_from_u8(s.rs2));
            let imm = builder.const_i64(s.imm as i64);
            let addr = builder.add(rs1, imm, IrType::I64);
            let v = builder.trunc(rs2, IrType::I64, IrType::I32);
            builder.store(addr, v, MemWidth::W32);
        }
        Instruction::Sd(s) => {
            let rs1 = builder.get_reg(reg_from_u8(s.rs1));
            let rs2 = builder.get_reg(reg_from_u8(s.rs2));
            let imm = builder.const_i64(s.imm as i64);
            let addr = builder.add(rs1, imm, IrType::I64);
            builder.store(addr, rs2, MemWidth::W64);
        }

        // Branches
        Instruction::Beq(b) => {
            let rs1 = builder.get_reg(reg_from_u8(b.rs1));
            let rs2 = builder.get_reg(reg_from_u8(b.rs2));
            let cond = builder.eq(rs1, rs2);
            lower_branch(builder, cond, current_pc, next_pc, b.imm);
        }
        Instruction::Bne(b) => {
            let rs1 = builder.get_reg(reg_from_u8(b.rs1));
            let rs2 = builder.get_reg(reg_from_u8(b.rs2));
            let cond = builder.ne(rs1, rs2);
            lower_branch(builder, cond, current_pc, next_pc, b.imm);
        }
        Instruction::Blt(b) => {
            let rs1 = builder.get_reg(reg_from_u8(b.rs1));
            let rs2 = builder.get_reg(reg_from_u8(b.rs2));
            let cond = builder.lt(rs1, rs2);
            lower_branch(builder, cond, current_pc, next_pc, b.imm);
        }
        Instruction::Bge(b) => {
            let rs1 = builder.get_reg(reg_from_u8(b.rs1));
            let rs2 = builder.get_reg(reg_from_u8(b.rs2));
            let cond = builder.ge(rs1, rs2);
            lower_branch(builder, cond, current_pc, next_pc, b.imm);
        }
        Instruction::Bltu(b) => {
            let rs1 = builder.get_reg(reg_from_u8(b.rs1));
            let rs2 = builder.get_reg(reg_from_u8(b.rs2));
            let cond = builder.ltu(rs1, rs2);
            lower_branch(builder, cond, current_pc, next_pc, b.imm);
        }
        Instruction::Bgeu(b) => {
            let rs1 = builder.get_reg(reg_from_u8(b.rs1));
            let rs2 = builder.get_reg(reg_from_u8(b.rs2));
            let cond = builder.geu(rs1, rs2);
            lower_branch(builder, cond, current_pc, next_pc, b.imm);
        }

        // Jumps
        Instruction::Jal(j) => {
            let next_pc_val = builder.const_i64(next_pc as i64);
            builder.set_reg(reg_from_u8(j.rd), next_pc_val);

            let base = builder.const_i64(current_pc as i64);
            let imm = builder.const_i64(j.imm as i64);
            let target = builder.add(base, imm, IrType::I64);
            builder.set_pc(target);
            builder.ret();
        }
        Instruction::Jalr(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let imm = builder.const_i64(i.imm as i64);
            let sum = builder.add(rs1, imm, IrType::I64);
            let mask = builder.const_i64(-2);
            let target = builder.and(sum, mask, IrType::I64);

            let next_pc_val = builder.const_i64(next_pc as i64);
            builder.set_reg(reg_from_u8(i.rd), next_pc_val);
            builder.set_pc(target);
            builder.ret();
        }

        // Upper immediates
        Instruction::Lui(u) => {
            let imm = builder.const_i64(u.imm as i64);
            builder.set_reg(reg_from_u8(u.rd), imm);
        }
        Instruction::Auipc(u) => {
            let base = builder.const_i64(current_pc as i64);
            let imm = builder.const_i64(u.imm as i64);
            let v = builder.add(base, imm, IrType::I64);
            builder.set_reg(reg_from_u8(u.rd), v);
        }

        // RV64I Register-Immediate
        Instruction::Addiw(i) => {
            let rs1 = builder.get_reg(reg_from_u8(i.rs1));
            let rs1_32 = builder.trunc(rs1, IrType::I64, IrType::I32);
            let imm = builder.const_i32(i.imm);
            let sum = builder.add(rs1_32, imm, IrType::I32);
            let v = builder.sext(sum, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(i.rd), v);
        }
        Instruction::Slliw(sh) => {
            let rs1 = builder.get_reg(reg_from_u8(sh.rs1));
            let rs1_32 = builder.trunc(rs1, IrType::I64, IrType::I32);
            let shamt = builder.const_i32(sh.shamt as i32);
            let shifted = builder.shl(rs1_32, shamt, IrType::I32);
            let v = builder.sext(shifted, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(sh.rd), v);
        }
        Instruction::Srliw(sh) => {
            let rs1 = builder.get_reg(reg_from_u8(sh.rs1));
            let rs1_32 = builder.trunc(rs1, IrType::I64, IrType::I32);
            let shamt = builder.const_i32(sh.shamt as i32);
            let shifted = builder.shr(rs1_32, shamt, IrType::I32);
            let v = builder.sext(shifted, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(sh.rd), v);
        }
        Instruction::Sraiw(sh) => {
            let rs1 = builder.get_reg(reg_from_u8(sh.rs1));
            let rs1_32 = builder.trunc(rs1, IrType::I64, IrType::I32);
            let shamt = builder.const_i32(sh.shamt as i32);
            let shifted = builder.sar(rs1_32, shamt, IrType::I32);
            let v = builder.sext(shifted, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(sh.rd), v);
        }

        // RV64I Register-Register
        Instruction::Addw(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let rs1_32 = builder.trunc(rs1, IrType::I64, IrType::I32);
            let rs2_32 = builder.trunc(rs2, IrType::I64, IrType::I32);
            let sum = builder.add(rs1_32, rs2_32, IrType::I32);
            let v = builder.sext(sum, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Subw(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let rs1_32 = builder.trunc(rs1, IrType::I64, IrType::I32);
            let rs2_32 = builder.trunc(rs2, IrType::I64, IrType::I32);
            let diff = builder.sub(rs1_32, rs2_32, IrType::I32);
            let v = builder.sext(diff, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Sllw(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let rs1_32 = builder.trunc(rs1, IrType::I64, IrType::I32);
            let mask = builder.const_i64(0x1f);
            let sh64 = builder.and(rs2, mask, IrType::I64);
            let sh = builder.trunc(sh64, IrType::I64, IrType::I32);
            let shifted = builder.shl(rs1_32, sh, IrType::I32);
            let v = builder.sext(shifted, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Srlw(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let rs1_32 = builder.trunc(rs1, IrType::I64, IrType::I32);
            let mask = builder.const_i64(0x1f);
            let sh64 = builder.and(rs2, mask, IrType::I64);
            let sh = builder.trunc(sh64, IrType::I64, IrType::I32);
            let shifted = builder.shr(rs1_32, sh, IrType::I32);
            let v = builder.sext(shifted, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }
        Instruction::Sraw(r) => {
            let rs1 = builder.get_reg(reg_from_u8(r.rs1));
            let rs2 = builder.get_reg(reg_from_u8(r.rs2));
            let rs1_32 = builder.trunc(rs1, IrType::I64, IrType::I32);
            let mask = builder.const_i64(0x1f);
            let sh64 = builder.and(rs2, mask, IrType::I64);
            let sh = builder.trunc(sh64, IrType::I64, IrType::I32);
            let shifted = builder.sar(rs1_32, sh, IrType::I32);
            let v = builder.sext(shifted, IrType::I32, IrType::I64);
            builder.set_reg(reg_from_u8(r.rd), v);
        }

        Instruction::Nop => {}

        _ => panic!("IR2 lowering missing for {:?}", insn),
    }
}

fn lower_branch(builder: &mut IrBuilder, cond: ValueId, current_pc: u64, next_pc: u64, imm: i32) {
    let taken = builder.block();
    let fallthrough = builder.block();

    builder.cbr(cond, taken, fallthrough, vec![], vec![]);

    builder.switch_to(taken);
    let base = builder.const_i64(current_pc as i64);
    let offset = builder.const_i64(imm as i64);
    let target = builder.add(base, offset, IrType::I64);
    builder.set_pc(target);
    builder.ret();

    builder.switch_to(fallthrough);
    let fallthrough_pc = builder.const_i64(next_pc as i64);
    builder.set_pc(fallthrough_pc);
    builder.ret();
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
