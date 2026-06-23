use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};

use crate::aot::{
    classification::{
        classify_shadow_case, classify_unary_shadow_case, classify_unary_zero_case,
        classify_zero_case, ShadowCase, UnaryShadowCase, UnaryZeroCase, ZeroCase,
    },
    instruction_context::InstructionContextBuilder,
    registers::{RiscvRegister, X86Gpr},
    temp_alloc::TempAllocator,
    translator::Translator,
};

/// RV64 `subw`: subtract low 32 bits, then sign-extend to 64 bits.
/// rd <- sext32((rs1[31:0] - rs2[31:0]) mod 2^32)
pub(super) fn emit_subw(
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
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            dynasm!(translator.emitter ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            if rd.id() != rs2.id() {
                dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs2.id()));
            }

            dynasm!(translator.emitter ; neg Rd(rd.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs1.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(&rd, &rs1, &rs2) {
        ShadowCase::AllEqual | ShadowCase::Rs1EqRs2 => {
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            dynasm!(translator.emitter ; sub Rd(rd.id()), Rd(rs2.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            dynasm!(translator.emitter ; neg Rd(rd.id()));
            dynasm!(translator.emitter ; add Rd(rd.id()), Rd(rs1.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs1.id()));
            dynasm!(translator.emitter ; sub Rd(rd.id()), Rd(rs2.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sraiw`: arithmetic right shift word by immediate.
/// rd <- sext32(rs1[31:0] >>> shamt)
pub(super) fn emit_sraiw(
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
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero | UnaryZeroCase::Rs1Zero => {
            // case 1
            // sraiw rd, 0, 0 -> rd = 0
            //
            // case 2
            // sraiw rd, 0, shamt -> rd = 0
            dynasm!(translator.emitter ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // sraiw rd, rs1, 0
            // -> rd = sext32(rs1[31:0])
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // sraiw rd, rd, shamt
            dynasm!(translator.emitter ; sar Rd(rd.id()), shamt as i8);
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // sraiw rd, rs1, shamt
            dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs1.id()));
            dynasm!(translator.emitter ; sar Rd(rd.id()), shamt as i8);
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sllw`: logical left shift word by register low bits.
/// rd <- sext32(rs1[31:0] << (rs2 & 0x1f))
pub(super) fn emit_sllw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rcx])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero => {
            // case 1
            // sllw rd, 0, 0 -> rd = 0
            //
            // case 2
            // sllw rd, 0, rs2 -> rd = 0
            dynasm!(translator.emitter ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // sllw rd, rs1, 0
            // -> rd = sext32(rs1[31:0])
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    dynasm!(translator.emitter ; mov Rq(X86Gpr::Rcx.id()), Rq(rs2.id()));

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // sllw rd, rd, rs2
            dynasm!(translator.emitter ; shl Rd(rd.id()), cl);
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // sllw rd, rs1, rs2
            dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs1.id()));
            dynasm!(translator.emitter ; shl Rd(rd.id()), cl);
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `srlw`: logical right shift word by register low bits.
/// rd <- sext32(rs1[31:0] >> (rs2 & 0x1f))
pub(super) fn emit_srlw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rcx])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero => {
            // case 1
            // srlw rd, 0, 0 -> rd = 0
            //
            // case 2
            // srlw rd, 0, rs2 -> rd = 0
            dynasm!(translator.emitter ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // srlw rd, rs1, 0
            // -> rd = sext32(rs1[31:0])
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    dynasm!(translator.emitter ; mov Rq(X86Gpr::Rcx.id()), Rq(rs2.id()));

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // srlw rd, rd, rs2
            dynasm!(translator.emitter ; shr Rd(rd.id()), cl);
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // srlw rd, rs1, rs2
            dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs1.id()));
            dynasm!(translator.emitter ; shr Rd(rd.id()), cl);
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `slliw`: logical left shift word by immediate.
/// rd <- sext32(rs1[31:0] << shamt)
pub(super) fn emit_slliw(
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
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero | UnaryZeroCase::Rs1Zero => {
            // case 1
            // slliw rd, 0, 0 -> rd = 0
            //
            // case 2
            // slliw rd, 0, shamt -> rd = 0
            dynasm!(translator.emitter ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // slliw rd, rs1, 0
            // -> rd = sext32(rs1[31:0])
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // slliw rd, rd, shamt
            dynasm!(translator.emitter ; shl Rd(rd.id()), shamt as i8);
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // slliw rd, rs1, shamt
            dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs1.id()));
            dynasm!(translator.emitter ; shl Rd(rd.id()), shamt as i8);
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `srliw`: logical right shift word by immediate.
/// rd <- sext32(rs1[31:0] >> shamt)
#[allow(unused_variables)]
pub(super) fn emit_srliw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
}

/// RV64 `addw`: add low 32 bits, then sign-extend to 64 bits.
/// rd <- sext32((rs1[31:0] + rs2[31:0]) mod 2^32)
#[allow(unused_variables)]
pub(super) fn emit_addw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
}

/// RV64 `addiw`: add sign-extended immediate to low 32 bits, then sign-extend.
/// rd <- sext32((rs1[31:0] + sext(imm)) mod 2^32)
#[allow(unused_variables)]
pub(super) fn emit_addiw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
}
