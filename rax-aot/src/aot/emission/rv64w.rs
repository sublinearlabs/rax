use dynasmrt::DynasmApi;

use crate::aot::emit_asm;

use crate::aot::{
    classification::{
        classify_shadow_case, classify_unary_shadow_case, classify_unary_zero_case,
        classify_zero_case, ShadowCase, UnaryShadowCase, UnaryZeroCase, ZeroCase,
    },
    instruction_context::InstructionContextBuilder,
    registers::{RiscvRegister, X86Gpr},
    translator::Translator,
};

/// RV64 `subw`: subtract low 32 bits, then sign-extend to 64 bits.
/// rd <- sext32((rs1[31:0] - rs2[31:0]) mod 2^32)
pub(super) fn emit_subw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator);
    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            emit_asm!(translator ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            if rd.id() != rs2.id() {
                emit_asm!(translator ; mov Rd(rd.id()), Rd(rs2.id()));
            }

            emit_asm!(translator ; neg Rd(rd.id()));
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(&rd, &rs1, &rs2) {
        ShadowCase::AllEqual | ShadowCase::Rs1EqRs2 => {
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            emit_asm!(translator ; sub Rd(rd.id()), Rd(rs2.id()));
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            emit_asm!(translator ; neg Rd(rd.id()));
            emit_asm!(translator ; add Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            emit_asm!(translator ; mov Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; sub Rd(rd.id()), Rd(rs2.id()));
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sraiw`: arithmetic right shift word by immediate.
/// rd <- sext32(rs1[31:0] >>> shamt)
pub(super) fn emit_sraiw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

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
            emit_asm!(translator ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // sraiw rd, rs1, 0
            // -> rd = sext32(rs1[31:0])
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // sraiw rd, rd, shamt
            emit_asm!(translator ; sar Rd(rd.id()), shamt as i8);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // sraiw rd, rs1, shamt
            emit_asm!(translator ; mov Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; sar Rd(rd.id()), shamt as i8);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sllw`: logical left shift word by register low bits.
/// rd <- sext32(rs1[31:0] << (rs2 & 0x1f))
pub(super) fn emit_sllw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rcx])
        .build(translator);

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
            emit_asm!(translator ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // sllw rd, rs1, 0
            // -> rd = sext32(rs1[31:0])
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    emit_asm!(translator ; mov Rq(X86Gpr::Rcx.id()), Rq(rs2.id()));

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // sllw rd, rd, rs2
            emit_asm!(translator ; shl Rd(rd.id()), cl);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // sllw rd, rs1, rs2
            emit_asm!(translator ; mov Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; shl Rd(rd.id()), cl);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `srlw`: logical right shift word by register low bits.
/// rd <- sext32(rs1[31:0] >> (rs2 & 0x1f))
pub(super) fn emit_srlw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rcx])
        .build(translator);

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
            emit_asm!(translator ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // srlw rd, rs1, 0
            // -> rd = sext32(rs1[31:0])
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    emit_asm!(translator ; mov Rq(X86Gpr::Rcx.id()), Rq(rs2.id()));

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // srlw rd, rd, rs2
            emit_asm!(translator ; shr Rd(rd.id()), cl);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // srlw rd, rs1, rs2
            emit_asm!(translator ; mov Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; shr Rd(rd.id()), cl);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sraw`: arithmetic right shift word by register low bits.
/// rd <- sext32(rs1[31:0] >>> (rs2 & 0x1f))
pub(super) fn emit_sraw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rcx])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero => {
            emit_asm!(translator ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    emit_asm!(translator ; mov Rq(X86Gpr::Rcx.id()), Rq(rs2.id()));

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            emit_asm!(translator ; sar Rd(rd.id()), cl);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            emit_asm!(translator ; mov Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; sar Rd(rd.id()), cl);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `slliw`: logical left shift word by immediate.
/// rd <- sext32(rs1[31:0] << shamt)
pub(super) fn emit_slliw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

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
            emit_asm!(translator ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // slliw rd, rs1, 0
            // -> rd = sext32(rs1[31:0])
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // slliw rd, rd, shamt
            emit_asm!(translator ; shl Rd(rd.id()), shamt as i8);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // slliw rd, rs1, shamt
            emit_asm!(translator ; mov Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; shl Rd(rd.id()), shamt as i8);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `srliw`: logical right shift word by immediate.
/// rd <- sext32(rs1[31:0] >> shamt)
pub(super) fn emit_srliw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, shamt as i32) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero | UnaryZeroCase::Rs1Zero => {
            // case 1
            // srliw rd, 0, 0 -> rd = 0
            //
            // case 2
            // srliw rd, 0, shamt -> rd = 0
            emit_asm!(translator ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // srliw rd, rs1, 0
            // -> rd = sext32(rs1[31:0])
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // srliw rd, rd, shamt
            emit_asm!(translator ; shr Rd(rd.id()), shamt as i8);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // srliw rd, rs1, shamt
            emit_asm!(translator ; mov Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; shr Rd(rd.id()), shamt as i8);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `addw`: add low 32 bits, then sign-extend to 64 bits.
/// rd <- sext32((rs1[31:0] + rs2[31:0]) mod 2^32)
pub(super) fn emit_addw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator);
    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // addw rd, 0, 0 -> rd = 0
            emit_asm!(translator ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // addw rd, 0, rs2 -> sext32(rs2[31:0])
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // addw rd, rs1, 0 -> sext32(rs1[31:0])
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(&rd, &rs1, &rs2) {
        ShadowCase::AllEqual | ShadowCase::Rs1EqRs2 => {
            // case 1 — AllEqual
            // addw rd, rd, rd -> sext32(rd[31:0] << 1)
            //
            // case 2 — Rs1EqRs2
            // addw rd, rs1, rs1 -> sext32(rs1[31:0] << 1)
            // Avoid LEA here: x86-64 cannot encode RSP as an index register, so a
            // source operand carried in RSP would be misencoded or omitted.
            emit_asm!(translator ; mov Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; add Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // addw rd, rd, rs2
            emit_asm!(translator ; add Rd(rd.id()), Rd(rs2.id()));
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // addw rd, rs1, rd
            emit_asm!(translator ; add Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // addw rd, rs1, rs2
            // Avoid LEA here: x86-64 cannot encode RSP as an index register, so a
            // source operand carried in RSP would be misencoded or omitted.
            emit_asm!(translator ; mov Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; add Rd(rd.id()), Rd(rs2.id()));
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `addiw`: add sign-extended immediate to low 32 bits, then sign-extend.
/// rd <- sext32((rs1[31:0] + sext(imm)) mod 2^32)
pub(super) fn emit_addiw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, imm) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero => {
            // addiw rd, 0, 0 -> rd = 0
            emit_asm!(translator ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::Rs1Zero => {
            // addiw rd, 0, imm
            // -> rd = sext64(imm)
            emit_asm!(translator ; mov Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // addiw rd, rs1, 0 -> sext32(rs1[31:0])
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // addiw rd, rd, imm
            emit_asm!(translator ; add Rd(rd.id()), imm);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // addiw rd, rs1, imm
            emit_asm!(translator ; mov Rd(rd.id()), Rd(rs1.id()));
            emit_asm!(translator ; add Rd(rd.id()), imm);
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}
