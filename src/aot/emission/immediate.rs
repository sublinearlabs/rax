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

/// RV64 `addi`: 64-bit wrapping add with sign-extended immediate.
/// rd <- (rs1 + sext(imm)) mod 2^64
pub(super) fn emit_addi(
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
pub(super) fn emit_addiw(
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
pub(super) fn emit_andi(
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
pub(super) fn emit_xori(
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
pub(super) fn emit_ori(
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
pub(super) fn emit_slti(
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
pub(super) fn emit_sltiu(
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
pub(super) fn emit_slli(
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
pub(super) fn emit_slliw(
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
pub(super) fn emit_srli(
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
pub(super) fn emit_srliw(
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
pub(super) fn emit_srai(
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
pub(super) fn emit_sraiw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
    // TODO: implement RV64 sraiw emission.
}
