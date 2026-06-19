use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};

use crate::aot::{
    classification::{
        classify_shadow_case, classify_unary_shadow_case, classify_unary_zero_case,
        classify_zero_case, ShadowCase, UnaryShadowCase, UnaryZeroCase, ZeroCase,
    },
    instruction_context::InstructionContextBuilder,
    register_mapping::MapTarget,
    registers::{RiscvRegister, X86Gpr},
    temp_alloc::TempAllocator,
    translator::Translator,
};
use crate::decode::{Instruction, Sh, B, I, J, R, S, U};

pub(super) fn emit_instruction(
    translator: &mut Translator,
    temps: &TempAllocator,
    insn: &Instruction,
) {
    match insn {
        Instruction::Nop => {}
        Instruction::Add(R { rd, rs1, rs2 }) => {
            emit_add(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sub(R { rd, rs1, rs2 }) => {
            emit_sub(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Xor(R { rd, rs1, rs2 }) => {
            emit_xor(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Or(R { rd, rs1, rs2 }) => emit_or(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::And(R { rd, rs1, rs2 }) => {
            emit_and(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Addw(R { rd, rs1, rs2 }) => {
            emit_addw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Subw(R { rd, rs1, rs2 }) => {
            emit_subw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sllw(R { rd, rs1, rs2 }) => {
            emit_sllw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Srlw(R { rd, rs1, rs2 }) => {
            emit_srlw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sraw(R { rd, rs1, rs2 }) => {
            emit_sraw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Slt(R { rd, rs1, rs2 }) => {
            emit_slt(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sltu(R { rd, rs1, rs2 }) => {
            emit_sltu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::LrW(R { rd, rs1, rs2 }) => {
            emit_lr_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::ScW(R { rd, rs1, rs2 }) => {
            emit_sc_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoSwapW(R { rd, rs1, rs2 }) => {
            emit_amo_swap_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoAddW(R { rd, rs1, rs2 }) => {
            emit_amo_add_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoXorW(R { rd, rs1, rs2 }) => {
            emit_amo_xor_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoAndW(R { rd, rs1, rs2 }) => {
            emit_amo_and_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoOrW(R { rd, rs1, rs2 }) => {
            emit_amo_or_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMinW(R { rd, rs1, rs2 }) => {
            emit_amo_min_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMaxW(R { rd, rs1, rs2 }) => {
            emit_amo_max_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMinuW(R { rd, rs1, rs2 }) => {
            emit_amo_minu_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMaxuW(R { rd, rs1, rs2 }) => {
            emit_amo_maxu_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::LrD(R { rd, rs1, rs2 }) => {
            emit_lr_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::ScD(R { rd, rs1, rs2 }) => {
            emit_sc_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoSwapD(R { rd, rs1, rs2 }) => {
            emit_amo_swap_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoAddD(R { rd, rs1, rs2 }) => {
            emit_amo_add_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoXorD(R { rd, rs1, rs2 }) => {
            emit_amo_xor_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoAndD(R { rd, rs1, rs2 }) => {
            emit_amo_and_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoOrD(R { rd, rs1, rs2 }) => {
            emit_amo_or_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMinD(R { rd, rs1, rs2 }) => {
            emit_amo_min_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMaxD(R { rd, rs1, rs2 }) => {
            emit_amo_max_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMinuD(R { rd, rs1, rs2 }) => {
            emit_amo_minu_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMaxuD(R { rd, rs1, rs2 }) => {
            emit_amo_maxu_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mul(R { rd, rs1, rs2 }) => {
            emit_mul(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulh(R { rd, rs1, rs2 }) => {
            emit_mulh(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulhsu(R { rd, rs1, rs2 }) => {
            emit_mulhsu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulhu(R { rd, rs1, rs2 }) => {
            emit_mulhu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Div(R { rd, rs1, rs2 }) => {
            emit_div(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Divu(R { rd, rs1, rs2 }) => {
            emit_divu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Rem(R { rd, rs1, rs2 }) => {
            emit_rem(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Remu(R { rd, rs1, rs2 }) => {
            emit_remu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulw(R { rd, rs1, rs2 }) => {
            emit_mulw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Divw(R { rd, rs1, rs2 }) => {
            emit_divw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Divuw(R { rd, rs1, rs2 }) => {
            emit_divuw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Remw(R { rd, rs1, rs2 }) => {
            emit_remw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Remuw(R { rd, rs1, rs2 }) => {
            emit_remuw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Addi(I { rd, rs1, imm }) => {
            emit_addi(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Andi(I { rd, rs1, imm }) => {
            emit_andi(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Xori(I { rd, rs1, imm }) => {
            emit_xori(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Ori(I { rd, rs1, imm }) => emit_ori(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Slti(I { rd, rs1, imm }) => {
            emit_slti(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Sltiu(I { rd, rs1, imm }) => {
            emit_sltiu(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Addiw(I { rd, rs1, imm }) => {
            emit_addiw(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Slli(Sh { rd, rs1, shamt }) => {
            emit_slli(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Srli(Sh { rd, rs1, shamt }) => {
            emit_srli(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Srai(Sh { rd, rs1, shamt }) => {
            emit_srai(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Slliw(Sh { rd, rs1, shamt }) => {
            emit_slliw(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Srliw(Sh { rd, rs1, shamt }) => {
            emit_srliw(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Sraiw(Sh { rd, rs1, shamt }) => {
            emit_sraiw(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Sll(R { rd, rs1, rs2 }) => {
            emit_sll(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Srl(R { rd, rs1, rs2 }) => {
            emit_srl(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sra(R { rd, rs1, rs2 }) => {
            emit_sra(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Lb(I { rd, rs1, imm }) => emit_lb(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lh(I { rd, rs1, imm }) => emit_lh(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lw(I { rd, rs1, imm }) => emit_lw(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lwu(I { rd, rs1, imm }) => emit_lwu(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Ld(I { rd, rs1, imm }) => emit_ld(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lbu(I { rd, rs1, imm }) => emit_lbu(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lhu(I { rd, rs1, imm }) => emit_lhu(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Sb(S { rs1, rs2, imm }) => emit_sb(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Sh(S { rs1, rs2, imm }) => emit_sh(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Sw(S { rs1, rs2, imm }) => emit_sw(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Sd(S { rs1, rs2, imm }) => emit_sd(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Lui(U { rd, imm }) => emit_lui(translator, temps, rv(rd), *imm),
        Instruction::Auipc(U { rd, imm }) => emit_auipc(translator, temps, rv(rd), *imm),
        Instruction::Beq(B { rs1, rs2, imm }) => {
            emit_beq(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bne(B { rs1, rs2, imm }) => {
            emit_bne(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Blt(B { rs1, rs2, imm }) => {
            emit_blt(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bge(B { rs1, rs2, imm }) => {
            emit_bge(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bltu(B { rs1, rs2, imm }) => {
            emit_bltu(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bgeu(B { rs1, rs2, imm }) => {
            emit_bgeu(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Jal(J { rd, imm }) => emit_jal(translator, temps, rv(rd), *imm),
        Instruction::Jalr(I { rd, rs1, imm }) => {
            emit_jalr(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Ecall => emit_ecall(translator, temps),
        Instruction::Csrrw(_) => {}
        _ => panic!("unknown opcode: {:?}", insn),
    }
}

fn rv(reg: &u8) -> RiscvRegister {
    RiscvRegister::from_index(*reg as usize).expect("invalid decoded RISC-V register")
}

/// RV64 `add`: 64-bit wrapping addition.
/// rd <- (rs1 + rs2) mod 2^64
fn emit_add(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator, temps);
    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            // x0 is hardwired to zero, writes can be ignored
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // add rd, 0, 0 -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // add rd, 0, rs2 -> rd = rs2

            // if rd and rs2 point to the same register
            // no need to waste a mov instruction
            if rd.id() == rs2.id() {
                ctx.write_back(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // add rd, rs1, 0 -> rd = rs1

            // if rd and rs1 point to the same register
            // no need to waste a mov instruction
            if rd.id() == rs1.id() {
                ctx.write_back(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(&rd, &rs1, &rs2) {
        ShadowCase::AllEqual => {
            // add rd, rd, rd
            // implies rd += rd
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // add rd, rd, rs2
            // imples rd += rs2
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // add rd, rs1, rd
            // implies rd += rs1
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::Rs1EqRs2 => {
            // add rd, rs1, rs1
            // implies rd = rs1 + rs1
            dynasm!(translator.emitter ; lea Rq(rd.id()), [Rq(rs1.id()) + Rq(rs1.id())]);
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sub`: 64-bit wrapping subtraction.
/// rd <- (rs1 - rs2) mod 2^64
fn emit_sub(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator, temps);
    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            // x0 is hardwired to zero. writes can be ignored.
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // sub rd, 0, 0 -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // sub rd, 0, rs2 -> rd = -rs2
            if rd.id() != rs2.id() {
                dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs2.id()));
            }

            dynasm!(translator.emitter ; neg Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // sub rd, rs1, 0 -> rd = rs1

            // if rd and rs1 point to the same register
            // no need to waste a mov instruction
            if rd.id() == rs1.id() {
                ctx.write_back(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(&rd, &rs1, &rs2) {
        ShadowCase::AllEqual | ShadowCase::Rs1EqRs2 => {
            // sub rd, rd, rd
            // -> rd = rd - rd
            // -> rd = 0
            //
            // sub rd, rs1, rs1
            // -> rd = rs1 - rs1
            // -> rd = 0

            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // sub rd, rd, rs2
            // -> rd -= rs2

            dynasm!(translator.emitter ; sub Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // sub rd, rs1, rd
            // -> rd = rs1 - rd
            // negate the rd
            // then add rs1

            dynasm!(translator.emitter ; neg Rq(rd.id()));
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // sub rd, rs1, rs2

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; sub Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `xor`: bitwise XOR across all 64 bits.
/// rd <- rs1 ^ rs2
#[allow(unused_variables)]
fn emit_xor(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 xor emission.
}

/// RV64 `or`: bitwise OR across all 64 bits.
/// rd <- rs1 | rs2
fn emit_or(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator, temps);
    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            // x0 is hardwired to zero. writes can be ignored.
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // or rd, 0, 0
            // -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // or rd, 0, rs2
            // -> rd = rs2

            // if they point to the same register
            // no need to waste a mov
            if rd.id() == rs2.id() {
                ctx.write_back(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // or rd, rs1, 0
            // -> rd = rs1

            // if they point to the same register
            // no need to waste a mov
            if rd.id() == rs1.id() {
                ctx.write_back(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(&rd, &rs1, &rs2) {
        ShadowCase::AllEqual => {
            // or rd, rd, rd
            // -> rd = rd | rd
            // -> rd = rd

            // no emission needed
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // or rd, rd, rs2
            // -> rd = rd | rs2

            dynasm!(translator.emitter ; or Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // or rd, rs1, rd
            // -> rd = rd | rs1

            dynasm!(translator.emitter ; or Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::Rs1EqRs2 => {
            // or rd, rs1, rs1
            // -> rd = rs1 | rs1
            // -> rd = rs1

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // or rd, rs1, rs2

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; or Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `and`: bitwise and.
/// rd <- rs1 & rs2
#[allow(unused_variables)]
fn emit_and(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 and emission.
}

/// RV64 `slt`: set if less than, signed.
/// rd <- signed(rs1) < signed(rs2)
#[allow(unused_variables)]
fn emit_slt(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 slt emission.
}

/// RV64 `addw`: add low 32 bits, then sign-extend to 64 bits.
/// rd <- sext32((rs1[31:0] + rs2[31:0]) mod 2^32)
#[allow(unused_variables)]
fn emit_addw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 addw emission.
}

/// RV64 `subw`: subtract low 32 bits, then sign-extend to 64 bits.
/// rd <- sext32((rs1[31:0] - rs2[31:0]) mod 2^32)
fn emit_subw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator, temps);
    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            // x0 is hardwired to zero, writes can be ignored
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // subw rd, 0, 0
            // -> rd = 0

            // zero out the rd register
            dynasm!(translator.emitter ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // subw rd, 0, rs2
            // -> rd = 0 - rs2
            // -> rd = low32(-rs2)

            if rd.id() != rs2.id() {
                dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs2.id()));
            }

            dynasm!(translator.emitter ; neg Rd(rd.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // subw rd, rs1, 0
            // -> rd = low32(rs1)

            if rd.id() != rs1.id() {
                dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs1.id()));
            }

            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(&rd, &rs1, &rs2) {
        ShadowCase::AllEqual | ShadowCase::Rs1EqRs2 => {
            // subw rd, rd, rd
            // -> rd = 0

            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // subw rd, rd, rs2
            // -> rd -= rs2

            dynasm!(translator.emitter ; sub Rd(rd.id()), Rd(rs2.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // subw rd, rs1, rd
            // -> rd = rs1 - rd
            // neg rd
            // add rs1

            dynasm!(translator.emitter ; neg Rd(rd.id()));
            dynasm!(translator.emitter ; add Rd(rd.id()), Rd(rs1.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // subw rd, rs1, rs2

            dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs1.id()));
            dynasm!(translator.emitter ; sub Rd(rd.id()), Rd(rs2.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sllw`: logical left shift low word, then sign-extend to 64 bits.
/// rd <- sext32(rs1[31:0] << (rs2 & 0x1f))
#[allow(unused_variables)]
fn emit_sllw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 sllw emission.
}

/// RV64 `srlw`: logical right shift low word, then sign-extend to 64 bits.
/// rd <- sext32(rs1[31:0] >> (rs2 & 0x1f))
#[allow(unused_variables)]
fn emit_srlw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 srlw emission.
}

/// RV64 `sraw`: arithmetic right shift low word, then sign-extend to 64 bits.
/// rd <- sext32(signed(rs1[31:0]) >> (rs2 & 0x1f))
#[allow(unused_variables)]
fn emit_sraw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 sraw emission.
}

/// RV64 `sltu`: set if less than, unsigned.
/// rd <- unsigned(rs1) < unsigned(rs2) ? 1 : 0
#[allow(unused_variables)]
fn emit_sltu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 sltu emission.
}

/// RV32A `lr.w`: load-reserved word.
/// rd <- sign_extend(M[rs1][31:0])
#[allow(unused_variables)]
fn emit_lr_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A lr.w emission.
}

/// RV32A `sc.w`: store-conditional word.
/// rd <- success ? 0 : nonzero; M[rs1][31:0] <- rs2[31:0] on success
#[allow(unused_variables)]
fn emit_sc_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A sc.w emission.
}

/// RV32A `amoswap.w`: atomically swap word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- rs2[31:0]
#[allow(unused_variables)]
fn emit_amo_swap_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amoswap.w emission.
}

/// RV32A `amoadd.w`: atomically add word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- M[rs1][31:0] + rs2[31:0]
#[allow(unused_variables)]
fn emit_amo_add_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amoadd.w emission.
}

/// RV32A `amoxor.w`: atomically xor word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- M[rs1][31:0] ^ rs2[31:0]
#[allow(unused_variables)]
fn emit_amo_xor_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amoxor.w emission.
}

/// RV32A `amoand.w`: atomically and word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- M[rs1][31:0] & rs2[31:0]
#[allow(unused_variables)]
fn emit_amo_and_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amoand.w emission.
}

/// RV32A `amoor.w`: atomically or word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- M[rs1][31:0] | rs2[31:0]
#[allow(unused_variables)]
fn emit_amo_or_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amoor.w emission.
}

/// RV32A `amomin.w`: atomically signed-min word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- min_signed(M[rs1][31:0], rs2[31:0])
#[allow(unused_variables)]
fn emit_amo_min_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amomin.w emission.
}

/// RV32A `amomax.w`: atomically signed-max word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- max_signed(M[rs1][31:0], rs2[31:0])
#[allow(unused_variables)]
fn emit_amo_max_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amomax.w emission.
}

/// RV32A `amominu.w`: atomically unsigned-min word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- min_unsigned(M[rs1][31:0], rs2[31:0])
#[allow(unused_variables)]
fn emit_amo_minu_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amominu.w emission.
}

/// RV32A `amomaxu.w`: atomically unsigned-max word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- max_unsigned(M[rs1][31:0], rs2[31:0])
#[allow(unused_variables)]
fn emit_amo_maxu_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amomaxu.w emission.
}

/// RV64A `lr.d`: load-reserved doubleword.
/// rd <- M[rs1][63:0]
#[allow(unused_variables)]
fn emit_lr_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A lr.d emission.
}

/// RV64A `sc.d`: store-conditional doubleword.
/// rd <- success ? 0 : nonzero; M[rs1][63:0] <- rs2 on success
#[allow(unused_variables)]
fn emit_sc_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A sc.d emission.
}

/// RV64A `amoswap.d`: atomically swap doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- rs2
#[allow(unused_variables)]
fn emit_amo_swap_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amoswap.d emission.
}

/// RV64A `amoadd.d`: atomically add doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- M[rs1][63:0] + rs2
#[allow(unused_variables)]
fn emit_amo_add_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amoadd.d emission.
}

/// RV64A `amoxor.d`: atomically xor doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- M[rs1][63:0] ^ rs2
#[allow(unused_variables)]
fn emit_amo_xor_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amoxor.d emission.
}

/// RV64A `amoand.d`: atomically and doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- M[rs1][63:0] & rs2
#[allow(unused_variables)]
fn emit_amo_and_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amoand.d emission.
}

/// RV64A `amoor.d`: atomically or doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- M[rs1][63:0] | rs2
#[allow(unused_variables)]
fn emit_amo_or_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amoor.d emission.
}

/// RV64A `amomin.d`: atomically signed-min doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- min_signed(M[rs1][63:0], rs2)
#[allow(unused_variables)]
fn emit_amo_min_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amomin.d emission.
}

/// RV64A `amomax.d`: atomically signed-max doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- max_signed(M[rs1][63:0], rs2)
#[allow(unused_variables)]
fn emit_amo_max_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amomax.d emission.
}

/// RV64A `amominu.d`: atomically unsigned-min doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- min_unsigned(M[rs1][63:0], rs2)
#[allow(unused_variables)]
fn emit_amo_minu_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amominu.d emission.
}

/// RV64A `amomaxu.d`: atomically unsigned-max doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- max_unsigned(M[rs1][63:0], rs2)
#[allow(unused_variables)]
fn emit_amo_maxu_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amomaxu.d emission.
}

/// RV64M `mul`: lower 64 bits of signed 64x64 multiply.
/// rd <- low64(rs1 * rs2)
#[allow(unused_variables)]
fn emit_mul(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M mul emission.
}

/// RV64M `mulh`: upper 64 bits of signed 64x64 multiply.
/// rd <- high64(signed(rs1) * signed(rs2))
#[allow(unused_variables)]
fn emit_mulh(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M mulh emission.
}

/// RV64M `mulhsu`: upper 64 bits of signed-by-unsigned 64x64 multiply.
/// rd <- high64(signed(rs1) * unsigned(rs2))
#[allow(unused_variables)]
fn emit_mulhsu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M mulhsu emission.
}

/// RV64 `mulhu`: upper 64 bits of unsigned 64x64 multiply.
/// rd <- high64(unsigned(rs1) * unsigned(rs2))
#[allow(unused_variables)]
fn emit_mulhu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M mulhu emission.
}

/// RV64M `div`: signed 64-bit division.
/// rd <- signed(rs1) / signed(rs2)
#[allow(unused_variables)]
fn emit_div(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M div emission.
}

/// RV64M `divu`: unsigned 64-bit division.
/// rd <- unsigned(rs1) / unsigned(rs2)
#[allow(unused_variables)]
fn emit_divu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M divu emission.
}

/// RV64M `rem`: signed 64-bit remainder.
/// rd <- signed(rs1) % signed(rs2)
#[allow(unused_variables)]
fn emit_rem(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M rem emission.
}

/// RV64M `remu`: unsigned 64-bit remainder.
/// rd <- unsigned(rs1) % unsigned(rs2)
#[allow(unused_variables)]
fn emit_remu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M remu emission.
}

/// RV64M `mulw`: lower 32 bits of multiply, then sign-extend to 64 bits.
/// rd <- sext32(rs1[31:0] * rs2[31:0])
#[allow(unused_variables)]
fn emit_mulw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M mulw emission.
}

/// RV64M `divw`: signed 32-bit division, then sign-extend to 64 bits.
/// rd <- sext32(signed(rs1[31:0]) / signed(rs2[31:0]))
#[allow(unused_variables)]
fn emit_divw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M divw emission.
}

/// RV64M `divuw`: unsigned 32-bit division, then sign-extend to 64 bits.
/// rd <- sext32(unsigned(rs1[31:0]) / unsigned(rs2[31:0]))
#[allow(unused_variables)]
fn emit_divuw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M divuw emission.
}

/// RV64M `remw`: signed 32-bit remainder, then sign-extend to 64 bits.
/// rd <- sext32(signed(rs1[31:0]) % signed(rs2[31:0]))
#[allow(unused_variables)]
fn emit_remw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M remw emission.
}

/// RV64M `remuw`: unsigned 32-bit remainder, then sign-extend to 64 bits.
/// rd <- sext32(unsigned(rs1[31:0]) % unsigned(rs2[31:0]))
#[allow(unused_variables)]
fn emit_remuw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M remuw emission.
}

/// RV64 `addi`: 64-bit wrapping add with sign-extended immediate.
/// rd <- (rs1 + sext(imm)) mod 2^64
fn emit_addi(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);
    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, imm) {
        UnaryZeroCase::RdZero => {
            // x0 is hardwired to zero, writes can be ignored
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero => {
            // addi rd, 0, 0 -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::Rs1Zero => {
            // addi rd, 0, imm -> rd = imm
            dynasm!(translator.emitter ; mov Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // addi rd, rs1, 0 -> rd = rs1
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // addi rd, rd, imm
            dynasm!(translator.emitter ; add Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // addi rd, rs1, imm
            dynasm!(translator.emitter ; lea Rq(rd.id()), [Rq(rs1.id()) + imm]);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `addiw`: add low 32-bit immediate result, then sign-extend to 64 bits.
/// rd <- sext32((rs1[31:0] + imm) mod 2^32)
#[allow(unused_variables)]
fn emit_addiw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 addiw emission.
}

/// RV64 `andi`: bitwise AND with sign-extended immediate.
/// rd <- rs1 & sext(imm)
fn emit_andi(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, imm) {
        UnaryZeroCase::RdZero => {
            // x0 is hardwired to zero, writes can be ignored
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero | UnaryZeroCase::Rs1Zero | UnaryZeroCase::ImmZero => {
            // in all cases, rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    // andi rd, rs1, -1 preserves all bits, so it is just a move/no-op.
    // Handle it before shadow lowering to avoid emitting `and rd, -1`.
    if imm == -1 {
        if rd.id() == rs1.id() {
            ctx.commit_unchanged(translator);
            return;
        }

        dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
        ctx.write_back(translator);
        return;
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // andi rd, rd, imm
            dynasm!(translator.emitter ; and Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // andi rd, rs1, imm
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; and Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `xori`: bitwise XOR with sign-extended immediate.
/// rd <- rs1 ^ sext(imm)
#[allow(unused_variables)]
fn emit_xori(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 xori emission.
}

/// RV64 `ori`: bitwise OR with sign-extended immediate.
/// rd <- rs1 | sext(imm)
#[allow(unused_variables)]
fn emit_ori(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 ori emission.
}

/// RV64 `slti`: set if less than signed immediate.
/// rd <- signed(rs1) < signed(sext(imm))
#[allow(unused_variables)]
fn emit_slti(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 slti emission.
}

/// RV64 `sltiu`: set if less than unsigned sign-extended immediate.
/// rd <- unsigned(rs1) < unsigned(sext(imm))
#[allow(unused_variables)]
fn emit_sltiu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 sltiu emission.
}

/// RV64 `slli`: logical left shift by immediate.
/// rd <- rs1 << shamt
fn emit_slli(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, shamt as i32) {
        UnaryZeroCase::RdZero => {
            // x0 is hardwired to zero, writes can be ignored
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero | UnaryZeroCase::Rs1Zero => {
            // Rs1ImmZero
            // slli rd, 0, 0 -> rd = 0
            //
            // Rs1Zero
            // slli rd, 0, imm -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // slli rd, rs1, 0 -> rd = rs1
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // slli rd, rd, imm
            dynasm!(translator.emitter ; shl Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // slli rd, rs1, imm
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; shl Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `slliw`: logical left shift low word, then sign-extend to 64 bits.
/// rd <- sext32(rs1[31:0] << shamt)
#[allow(unused_variables)]
fn emit_slliw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
    // TODO: implement RV64 slliw emission.
}

/// RV64 `srli`: logical right shift by immediate.
/// rd <- rs1 >> shamt
#[allow(unused_variables)]
fn emit_srli(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
    // TODO: implement RV64 srli emission.
}

/// RV64 `srliw`: logical right shift low word, then sign-extend to 64 bits.
/// rd <- sext32(rs1[31:0] >> shamt)
#[allow(unused_variables)]
fn emit_srliw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
    // TODO: implement RV64 srliw emission.
}

/// RV64 `srai`: arithmetic right shift by immediate.
/// rd <- signed(rs1) >> shamt
#[allow(unused_variables)]
fn emit_srai(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
    // TODO: implement RV64 srai emission.
}

/// RV64 `sraiw`: arithmetic right shift low word, then sign-extend to 64 bits.
/// rd <- sext32(signed(rs1[31:0]) >> shamt)
#[allow(unused_variables)]
fn emit_sraiw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
    // TODO: implement RV64 sraiw emission.
}

/// RV64 `sll`: logical left shift by register low bits.
/// rd <- rs1 << (rs2 & 0x3f)
fn emit_sll(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // given that the shift value for this is in a register
    // we are using this form:
    // shl r/m64, cl
    // the shift value must be in rcx before this is called
    // hence we ensure no clobber for that location
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rcx])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            // x0 is hardwired to zero, writes can be ignored
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero => {
            // Rs1Rs2Zero
            // sll rd, 0, 0 -> rd = 0
            //
            // Rs1Zero
            // sll rd, 0, rs2 -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // sll rd, rs1, 0 -> rd = rs1
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    // move the shamt value to rcx
    dynasm!(translator.emitter ; mov Rq(X86Gpr::Rcx.id()), Rq(rs2.id()));

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            dynasm!(translator.emitter ; shl Rq(rd.id()), cl);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; shl Rq(rd.id()), cl);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `srl`: logical right shift by register low bits.
/// rd <- rs1 >> (rs2 & 0x3f)
#[allow(unused_variables)]
fn emit_srl(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 srl emission.
}

/// RV64 `sra`: arithmetic right shift by register low bits.
/// rd <- signed(rs1) >> (rs2 & 0x3f)
#[allow(unused_variables)]
fn emit_sra(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 sra emission.
}

/// RV64 `lb`: load signed byte.
/// rd <- sign_extend(M[rs1 + imm][7:0])
#[allow(unused_variables)]
fn emit_lb(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 lb emission.
}

/// RV64 `lh`: load signed halfword.
/// rd <- sign_extend(M[rs1 + imm][15:0])
#[allow(unused_variables)]
fn emit_lh(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 lh emission.
}

/// RV64 `lw`: load signed word.
/// rd <- sign_extend(M[rs1 + imm][31:0])
#[allow(unused_variables)]
fn emit_lw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 lw emission.
}

/// RV64 `lwu`: load unsigned word.
/// rd <- zero_extend(M[rs1 + imm][31:0])
#[allow(unused_variables)]
fn emit_lwu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 lwu emission.
}

/// RV64 `ld`: load 64-bit value.
/// rd <- M[rs1 + imm][63:0]
#[allow(unused_variables)]
fn emit_ld(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 ld emission.
}

/// RV64 `lbu`: load unsigned byte.
/// rd <- zero_extend(M[rs1 + imm][7:0])
#[allow(unused_variables)]
fn emit_lbu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 lbu emission.
}

/// RV64 `lhu`: load unsigned halfword.
/// rd <- zero_extend(M[rs1 + imm][15:0])
#[allow(unused_variables)]
fn emit_lhu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 lhu emission.
}

/// RV64 `sb`: store low 8 bits of rs2 to memory at rs1 + sext(imm).
/// mem8[rs1 + sext(imm)] <- rs2[7:0]
fn emit_sb(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();

    // `x0` has no backing x86 register, but this x86 memory operand needs a
    // base register. Materialize `x0 + imm` into a temp and use no displacement.
    let (addr_id, addr_disp, _addr_temp) = if rs1.is_zero() {
        let temp = temps
            .allocate()
            .expect("emit_sb requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(temp.id()), QWORD imm as i64);
        (temp.id(), 0, Some(temp))
    } else {
        (rs1.id(), imm, None)
    };

    if rs2.is_zero() {
        dynasm!(translator.emitter ; mov BYTE [Rq(addr_id) + addr_disp], 0_i8);
    } else {
        dynasm!(translator.emitter ; mov BYTE [Rq(addr_id) + addr_disp], Rb(rs2.id()));
    }
    ctx.complete_no_output(translator);
}

/// RV64 `sh`: store low 16 bits of rs2 to memory at rs1 + sext(imm).
/// mem16[rs1 + sext(imm)] <- rs2[15:0]
#[allow(unused_variables)]
fn emit_sh(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 sh emission.
}

/// RV64 `sw`: store low 32 bits of rs2 to memory at rs1 + sext(imm).
/// mem32[rs1 + sext(imm)] <- rs2[31:0]
#[allow(unused_variables)]
fn emit_sw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 sw emission.
}

/// RV64 `sd`: store 64 bits of rs2 to memory at rs1 + sext(imm).
/// mem64[rs1 + sext(imm)] <- rs2
fn emit_sd(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();

    // `x0` has no backing x86 register, but this x86 memory operand needs a
    // base register. Materialize `x0 + imm` into a temp and use no displacement.
    let (addr_id, addr_disp, _addr_temp) = if rs1.is_zero() {
        let temp = temps
            .allocate()
            .expect("emit_sd requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(temp.id()), QWORD imm as i64);
        (temp.id(), 0, Some(temp))
    } else {
        (rs1.id(), imm, None)
    };

    if rs2.is_zero() {
        dynasm!(translator.emitter ; mov QWORD [Rq(addr_id) + addr_disp], 0_i32);
    } else {
        dynasm!(translator.emitter ; mov QWORD [Rq(addr_id) + addr_disp], Rq(rs2.id()));
    }

    ctx.complete_no_output(translator);
}

/// RV64 `lui`: write U-immediate to upper bits.
/// rd <- sext(imm << 12)
fn emit_lui(translator: &mut Translator, temps: &TempAllocator, rd: RiscvRegister, imm: i32) {
    if rd.is_zero() {
        // x0 is hardwired to 0, writes can be ignored
        return;
    }

    let ctx = InstructionContextBuilder::<0, 0>::new()
        .set_output(rd)
        .build(translator, temps);

    let rd = ctx.output();

    // NOTE: the immediate is already shifted by 12 from the decode layer
    dynasm!(translator.emitter ; mov Rq(rd.id()), imm);

    ctx.write_back(translator);
}

/// RV64 `auipc`: add U-immediate (<<12) to current PC.
/// rd <- pc + sext(imm << 12)
fn emit_auipc(translator: &mut Translator, temps: &TempAllocator, rd: RiscvRegister, imm: i32) {
    if rd.is_zero() {
        // x0 is hardwired to 0, writes can be ignored
        return;
    }

    let ctx = InstructionContextBuilder::<0, 0>::new()
        .set_output(rd)
        .build(translator, temps);

    let rd = ctx.output();

    let auipc_val = translator.current_pc().wrapping_add(imm as i64 as u64);

    // NOTE: the immediate is already shifted by 12 from the decode layer
    dynasm!(translator.emitter ; mov Rq(rd.id()), QWORD auipc_val as i64);

    ctx.write_back(translator);
}

/// RV64 `beq`: branch if equal.
/// if rs1 == rs2 then pc <- pc + sext(imm)
fn emit_beq(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();

    // compute the target riscv pc
    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    // retrieve or create a new dynamic label for the riscv pc
    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            // both are zero, hence both equal
            dynasm!(translator.emitter ; jmp => target_label);
        }
        (true, false) => {
            // rs1 equals zero
            // we need to check if rs2 equals zero
            dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; je => target_label);
        }
        (false, true) => {
            // rs2 equals zero
            // we need to check if rs1 equals zero
            dynasm!(translator.emitter ; test Rq(rs1.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; je => target_label);
        }
        (false, false) => {
            // both non zero
            dynasm!(translator.emitter ; cmp Rq(rs1.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; je => target_label);
        }
    }

    ctx.complete_no_output(translator);
}

/// RV64 `bne`: branch if not equal.
/// if rs1 != rs2 then pc <- pc + sext(imm)
fn emit_bne(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();

    // compute the target riscv pc
    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    // retrieve or create a new dynamic label for the riscv pc
    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            // both are equal, we shouldn't jump
        }
        (true, false) => {
            // rs1 is zero
            // check if rs2 is zero, don't jump if it is
            dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; jne => target_label);
        }
        (false, true) => {
            // rs2 is zero
            // check if rs1 is zero, don't jump if it is
            dynasm!(translator.emitter ; test Rq(rs1.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; jne => target_label);
        }
        (false, false) => {
            // both are not zero
            dynasm!(translator.emitter ; cmp Rq(rs1.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; jne => target_label);
        }
    }

    ctx.complete_no_output(translator);
}

/// RV64 `blt`: branch if signed rs1 < rs2.
/// if signed(rs1) < signed(rs2) then pc <- pc + sext(imm)
#[allow(unused_variables)]
fn emit_blt(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 blt emission.
}

/// RV64 `bge`: branch if signed rs1 >= rs2.
/// if signed(rs1) >= signed(rs2) then pc <- pc + sext(imm)
#[allow(unused_variables)]
fn emit_bge(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 bge emission.
}

/// RV64 `bltu`: branch if unsigned rs1 < rs2.
/// if unsigned(rs1) < unsigned(rs2) then pc <- pc + sext(imm)
fn emit_bltu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();

    // compute the target riscv pc
    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    // retrieve or create a new dynamic label for the riscv pc
    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            // both are equal, we shouldn't jump
        }
        (true, false) => {
            // rs1 is zero
            // if rs2 is anything but zero, it is fine to jump
            dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; jne => target_label);
        }
        (false, true) => {
            // rs2 is zero
            // rs1 cannot have a value that will be less than zero
            // hence we don't jump
        }
        (false, false) => {
            // both are not zero
            dynasm!(translator.emitter ; cmp Rq(rs1.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; jb => target_label);
        }
    }
    ctx.complete_no_output(translator);
}

/// RV64 `bgeu`: branch if unsigned rs1 >= rs2.
/// if unsigned(rs1) >= unsigned(rs2) then pc <- pc + sext(imm)
fn emit_bgeu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();

    // compute the target riscv pc
    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    // retrieve or create a new dynamic label for the riscv pc
    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            // rs1 equals rs2
            dynasm!(translator.emitter ; jmp => target_label);
        }
        (true, false) => {
            // rs1 is zero
            // only condition for jump will be if rs2 is also zero
            dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; je => target_label);
        }
        (false, true) => {
            // rs2 is zero
            // for all values of rs1, rs1 >= rs2
            // hence we always jump
            dynasm!(translator.emitter ; jmp => target_label);
        }
        (false, false) => {
            // both are not zero
            dynasm!(translator.emitter ; cmp Rq(rs1.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; jae => target_label);
        }
    }

    ctx.complete_no_output(translator);
}

/// RV64 `jal`: jump and link.
/// rd <- pc + 4; pc <- pc + sext(imm)
fn emit_jal(translator: &mut Translator, temps: &TempAllocator, rd: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<0, 0>::new()
        .set_output(rd)
        .build(translator, temps);

    let rd = ctx.output();

    if !rd.is_zero() {
        // set the return pc
        let return_pc = translator.current_pc().wrapping_add(4);
        dynasm!(translator.emitter ; mov Rq(rd.id()), QWORD return_pc as i64);
        ctx.write_back(translator);
    } else {
        ctx.discard_zero_output(translator);
    }

    // update pc
    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);
    let target_label = translator.target_label(branch_target);
    dynasm!(translator.emitter ; jmp => target_label);
}

/// RV64 `jalr`: indirect jump and link.
/// t <- pc + 4; pc <- (rs1 + sext(imm)) & !1; rd <- t
fn emit_jalr(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    // prepare temps
    let branch_target = temps.allocate().unwrap();
    let base_riscv_pc = temps.allocate().unwrap();

    // branch_target = (rs1 + imm) & !1
    // jump_table_id = (branch_target - base_riscv_pc) >> 2 (assumes uncompressed)
    if rs1.is_zero() {
        dynasm!(translator.emitter ; mov Rq(branch_target.id()), QWORD imm as i64);
    } else {
        dynasm!(translator.emitter ; lea Rq(branch_target.id()), [Rq(rs1.id()) + imm]);
    }
    dynasm!(translator.emitter ; and Rq(branch_target.id()), -2 as i32);
    dynasm!(translator.emitter ; mov Rq(base_riscv_pc.id()), QWORD translator.cf.base_riscv_pc as i64);
    dynasm!(translator.emitter ; sub Rq(branch_target.id()), Rq(base_riscv_pc.id()));
    dynasm!(translator.emitter ; shr Rq(branch_target.id()), 2);

    // write return pc
    if !rd.is_zero() {
        let return_pc = translator.current_pc().wrapping_add(4);
        dynasm!(translator.emitter ; mov Rq(rd.id()), QWORD return_pc as i64);
        ctx.write_back(translator);
    } else {
        ctx.discard_zero_output(translator);
    }

    // free the base riscv pc temp
    drop(base_riscv_pc);

    let jump_table_base_reg = temps.allocate().unwrap();

    // extract the value at the jump table index
    dynasm!(translator.emitter ; lea Rq(jump_table_base_reg.id()), [=>translator.cf.jt_label]);
    dynasm!(translator.emitter ; jmp QWORD [Rq(jump_table_base_reg.id()) + Rq(branch_target.id()) * 8]);
}

/// RV64 `ecall`: environment call trap.
/// raise environment-call-from-U-mode
fn emit_ecall(translator: &mut Translator, temps: &TempAllocator) {
    // we only handle the read, write and halt syscalls

    // this emission assumes that there is an
    // identity mapping between the riscv syscalls
    // arg registers and the x86 syscall arg registers
    assert_eq!(
        translator.reg_map.get(&RiscvRegister::A7),
        &MapTarget::Gpr(X86Gpr::Rax)
    );
    assert_eq!(
        translator.reg_map.get(&RiscvRegister::A0),
        &MapTarget::Gpr(X86Gpr::Rdi)
    );
    assert_eq!(
        translator.reg_map.get(&RiscvRegister::A1),
        &MapTarget::Gpr(X86Gpr::Rsi)
    );
    assert_eq!(
        translator.reg_map.get(&RiscvRegister::A2),
        &MapTarget::Gpr(X86Gpr::Rdx)
    );

    // syscall | riscv_code | x86_code
    // read    |     63     |    0
    // write   |     64     |    1
    // halt    |     93     |   60
    //
    // consider the polynomial that represents the mapping
    // f(x) = $(x^2 - 98x + 2205) / 29$
    // after simplification
    // f(x) = $((x - 49)^2 - 196) / 29$
    //
    // syscall clobbers rax, rcx and r11
    // we need to ensure no clobber for rcx and r11
    // trying to decide if we need the same for rax
    // technically rax will contain the riscv syscall code
    // to be semantically correct, we'd want to ensure no clobber also
    // but probably manually, because the non manual version

    // TODO: ideally Rax(a7) should also be clobber free
    // but that might be a waste as a7 is usually not live
    // preferred is to get this information from some liveness analysis
    let ctx = InstructionContextBuilder::<0, 2>::new()
        .ensure_no_clobber([X86Gpr::Rcx, X86Gpr::R11])
        .set_output(RiscvRegister::A0)
        .build(translator, temps);

    // rdx will be clobbered by imul and cqo
    // when preforming the polynomial evaluation
    let rdx_temp = temps.allocate().unwrap();
    dynasm!(translator.emitter ; mov Rq(rdx_temp.id()), Rq(X86Gpr::Rdx.id()));

    // rax = x - 49
    dynasm!(translator.emitter ; sub Rq(X86Gpr::Rax.id()), 49);
    // rax = (x - 49)^2
    dynasm!(translator.emitter ; imul Rq(X86Gpr::Rax.id()), Rq(X86Gpr::Rax.id()));
    // rax = (x - 49)^2 - 196
    dynasm!(translator.emitter ; sub Rq(X86Gpr::Rax.id()), 196);
    // sign extend rax into rdx
    // this is needed because idiv divides RDX:RAX
    // hence RDX:RAX must factually represent the number we
    // are trying to divide
    dynasm!(translator.emitter ; cqo);

    // computes ((x - 49)^2 - 196) / 29
    // stores quotient in RAX (correct syscall code)
    // stores remainder in RDX
    // after this RAX will contain the correct x86 syscall code
    let divisor_reg = temps.allocate().unwrap();
    dynasm!(translator.emitter ; mov Rq(divisor_reg.id()), 29);
    dynasm!(translator.emitter ; idiv Rq(divisor_reg.id()));

    // before calling syscall, we need to move
    // the old value of rdx back
    dynasm!(translator.emitter ; mov Rq(X86Gpr::Rdx.id()), Rq(rdx_temp.id()));

    // syscall
    dynasm!(translator.emitter ; syscall);

    // in x86 the result of the syscall is written to RAX
    // for riscv it is written to a0
    // given that our mapping doesn't make those locations equal
    // we need to move from rax to a0
    dynasm!(translator.emitter ; mov Rq(ctx.output().id()), Rq(X86Gpr::Rax.id()));

    // write back context
    ctx.write_back(translator);
}
