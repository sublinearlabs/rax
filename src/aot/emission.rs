use dynasmrt::{dynasm, DynasmApi};

use crate::aot::{
    registers::RiscvRegister,
    temp_alloc::TempAllocator,
    translator::{PreparedInput, PreparedOutput, Translator},
};
use crate::decode::{Instruction, Sh, B, I, J, R, S, U};

pub(super) fn emit_instruction(
    translator: &mut Translator,
    temps: &TempAllocator,
    insn: &Instruction,
) {
    match insn {
        Instruction::Add(R { rd, rs1, rs2 }) => {
            emit_add(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sub(R { rd, rs1, rs2 }) => {
            emit_sub(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Or(R { rd, rs1, rs2 }) => emit_or(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::Subw(R { rd, rs1, rs2 }) => {
            emit_subw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulhu(R { rd, rs1, rs2 }) => {
            emit_mulhu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Addi(I { rd, rs1, imm }) => {
            emit_addi(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Andi(I { rd, rs1, imm }) => {
            emit_andi(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Slli(Sh { rd, rs1, shamt }) => {
            emit_slli(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Sll(R { rd, rs1, rs2 }) => {
            emit_sll(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sb(S { rs1, rs2, imm }) => emit_sb(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Sd(S { rs1, rs2, imm }) => emit_sd(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Lui(U { rd, imm }) => emit_lui(translator, temps, rv(rd), *imm),
        Instruction::Auipc(U { rd, imm }) => emit_auipc(translator, temps, rv(rd), *imm),
        Instruction::Beq(B { rs1, rs2, imm }) => {
            emit_beq(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bne(B { rs1, rs2, imm }) => {
            emit_bne(translator, temps, rv(rs1), rv(rs2), *imm)
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

/// Normalized zero-related classification for `(rd, rs1, rs2)`.
///
/// Variants are mutually exclusive and exhaustive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ZeroCase {
    /// Destination is architectural zero (`rd == x0`).
    ///
    /// Output write is semantically elided and lowering should return early.
    RdZero,
    /// Both sources are architectural zero (`rs1 == x0 && rs2 == x0`),
    /// with `rd != x0`.
    Rs1Rs2Zero,
    /// First source is architectural zero and second is non-zero
    /// (`rs1 == x0 && rs2 != x0`), with `rd != x0`.
    Rs1Zero,
    /// Second source is architectural zero and first is non-zero
    /// (`rs2 == x0 && rs1 != x0`), with `rd != x0`.
    Rs2Zero,
    /// No zero-specific simplification applies
    /// (`rd != x0 && rs1 != x0 && rs2 != x0`).
    None,
}

/// Normalized alias/equality classification for `(rd, rs1, rs2)`.
///
/// This classification is intended for non-zero source paths, i.e. when
/// zero classification yields `ZeroCase::None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShadowCase {
    /// All operands name the same architectural register
    /// (`rd == rs1 && rs1 == rs2`).
    AllEqual,
    /// Destination aliases the first source (`rd == rs1`), with `rd != rs2`.
    RdEqRs1,
    /// Destination aliases the second source (`rd == rs2`), with `rd != rs1`.
    RdEqRs2,
    /// Sources are equal but destination is distinct (`rs1 == rs2`),
    /// with `rd != rs1`.
    Rs1EqRs2,
    /// All operands are pairwise distinct.
    AllDistinct,
}

/// Classifies zero-related simplification cases for prepared operands.
fn classify_zero_case(
    rd: &PreparedOutput<'_>,
    rs1: &PreparedInput<'_>,
    rs2: &PreparedInput<'_>,
) -> ZeroCase {
    if rd.is_zero() {
        return ZeroCase::RdZero;
    }
    if rs1.is_zero() && rs2.is_zero() {
        return ZeroCase::Rs1Rs2Zero;
    }
    if rs1.is_zero() {
        return ZeroCase::Rs1Zero;
    }
    if rs2.is_zero() {
        return ZeroCase::Rs2Zero;
    }
    ZeroCase::None
}

/// Classifies alias/equality relationships for prepared non-zero operands.
fn classify_shadow_case(
    rd: &PreparedOutput<'_>,
    rs1: &PreparedInput<'_>,
    rs2: &PreparedInput<'_>,
) -> ShadowCase {
    let rd_id = rd.id();
    let rs1_id = rs1.id();
    let rs2_id = rs2.id();

    if rd_id == rs1_id && rs1_id == rs2_id {
        return ShadowCase::AllEqual;
    }
    if rd_id == rs1_id {
        return ShadowCase::RdEqRs1;
    }
    if rd_id == rs2_id {
        return ShadowCase::RdEqRs2;
    }
    if rs1_id == rs2_id {
        return ShadowCase::Rs1EqRs2;
    }
    ShadowCase::AllDistinct
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
    let [rs1, rs2] = translator.prepare_inputs([rs1, rs2], temps);
    let rd = translator.prepare_output(rd, temps);

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            // x0 is hardwired to zero, writes can be ignored
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // add rd, 0, 0 -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            rd.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // add rd, 0, rs2 -> rd = rs2

            // if rd and rs2 point to the same register
            // no need to waste a mov instruction
            if rd.id() == rs2.id() {
                rd.commit_unchanged();
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs2.id()));
            rd.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // add rd, rs1, 0 -> rd = rs1

            // if rd and rs1 point to the same register
            // no need to waste a mov instruction
            if rd.id() == rs1.id() {
                rd.commit_unchanged();
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            rd.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(&rd, &rs1, &rs2) {
        ShadowCase::AllEqual => {
            // add rd, rd, rd
            // implies rd += rd
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rd.id()));
            rd.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // add rd, rd, rs2
            // imples rd += rs2
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs2.id()));
            rd.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // add rd, rs1, rd
            // implies rd += rs1
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs1.id()));
            rd.write_back(translator);
            return;
        }

        ShadowCase::Rs1EqRs2 => {
            // add rd, rs1, rs1
            // implies rd = rs1 + rs1
            dynasm!(translator.emitter ; lea Rq(rd.id()), [Rq(rs1.id()) + Rq(rs1.id())]);
            rd.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs2.id()));
            rd.write_back(translator);
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
    let [rs1, rs2] = translator.prepare_inputs([rs1, rs2], temps);
    let rd = translator.prepare_output(rd, temps);

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            // x0 is hardwired to zero. writes can be ignored.
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // sub rd, 0, 0 -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            rd.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // sub rd, 0, rs2 -> rd = -rs2
            if rd.id() != rs2.id() {
                dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs2.id()));
            }

            dynasm!(translator.emitter ; neg Rq(rd.id()));
            rd.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // sub rd, rs1, 0 -> rd = rs1

            // if rd and rs1 point to the same register
            // no need to waste a mov instruction
            if rd.id() == rs1.id() {
                rd.commit_unchanged();
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            rd.write_back(translator);
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
            rd.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // sub rd, rd, rs2
            // -> rd -= rs2

            dynasm!(translator.emitter ; sub Rq(rd.id()), Rq(rs2.id()));
            rd.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // sub rd, rs1, rd
            // -> rd = rs1 - rd
            // negate the rd
            // then add rs1

            dynasm!(translator.emitter ; neg Rq(rd.id()));
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs1.id()));
            rd.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // sub rd, rs1, rs2

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; sub Rq(rd.id()), Rq(rs2.id()));
            rd.write_back(translator);
            return;
        }
    }
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
    let _ = (translator, temps, rd, rs1, rs2);
    todo!("emit_or")
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
    let _ = (translator, temps, rd, rs1, rs2);
    todo!("emit_subw")
}

/// RV64 `mulhu`: upper 64 bits of unsigned 64x64 multiply.
/// rd <- high64(unsigned(rs1) * unsigned(rs2))
fn emit_mulhu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let _ = (translator, temps, rd, rs1, rs2);
    todo!("emit_mulhu")
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
    let _ = (translator, temps, rd, rs1, imm);
    todo!("emit_addi")
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
    let _ = (translator, temps, rd, rs1, imm);
    todo!("emit_andi")
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
    let _ = (translator, temps, rd, rs1, shamt);
    todo!("emit_slli")
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
    let _ = (translator, temps, rd, rs1, rs2);
    todo!("emit_sll")
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
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_sb")
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
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_sd")
}

/// RV64 `lui`: write U-immediate to upper bits.
/// rd <- sext(imm << 12)
fn emit_lui(translator: &mut Translator, temps: &TempAllocator, rd: RiscvRegister, imm: i32) {
    let _ = (translator, temps, rd, imm);
    todo!("emit_lui")
}

/// RV64 `auipc`: add U-immediate (<<12) to current PC.
/// rd <- pc + sext(imm << 12)
fn emit_auipc(translator: &mut Translator, temps: &TempAllocator, rd: RiscvRegister, imm: i32) {
    let _ = (translator, temps, rd, imm);
    todo!("emit_auipc")
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
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_beq")
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
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_bne")
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
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_bltu")
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
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_bgeu")
}

/// RV64 `jal`: jump and link.
/// rd <- pc + 4; pc <- pc + sext(imm)
fn emit_jal(translator: &mut Translator, temps: &TempAllocator, rd: RiscvRegister, imm: i32) {
    let _ = (translator, temps, rd, imm);
    todo!("emit_jal")
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
    let _ = (translator, temps, rd, rs1, imm);
    todo!("emit_jalr")
}

/// RV64 `ecall`: environment call trap.
/// raise environment-call-from-U-mode
fn emit_ecall(translator: &mut Translator, temps: &TempAllocator) {
    let _ = (translator, temps);
    todo!("emit_ecall")
}
