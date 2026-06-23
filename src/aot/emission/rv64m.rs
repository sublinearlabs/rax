use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};

use crate::aot::{
    classification::{classify_shadow_case, classify_zero_case, ShadowCase, ZeroCase},
    instruction_context::InstructionContextBuilder,
    registers::{RiscvRegister, X86Gpr},
    temp_alloc::TempAllocator,
    translator::Translator,
};

/// RV64 `mulhu`: upper 64 bits of unsigned 64x64 multiply.
/// rd <- high64(unsigned(rs1) * unsigned(rs2))
pub(super) fn emit_mulhu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }
        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero | ZeroCase::Rs2Zero => {
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }
        ZeroCase::None => {}
    }

    if rs1.id() == X86Gpr::Rax.id() {
        dynasm!(translator.emitter ; mul Rq(rs2.id()));
    } else if rs2.id() == X86Gpr::Rax.id() {
        dynasm!(translator.emitter ; mul Rq(rs1.id()));
    } else {
        dynasm!(translator.emitter ; mov Rq(X86Gpr::Rax.id()), Rq(rs1.id()));
        dynasm!(translator.emitter ; mul Rq(rs2.id()));
    }

    if rd.id() != X86Gpr::Rdx.id() {
        dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(X86Gpr::Rdx.id()));
    }

    ctx.write_back(translator);
}

/// RV64 `mul`: low 64 bits of signed 64x64 multiply.
/// rd <- low64(signed(rs1) * signed(rs2))
pub(super) fn emit_mul(
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

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero | ZeroCase::Rs2Zero => {
            // since at least one of the multiplication arguments
            // are zero, the result will be zero
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(rd, rs1, rs2) {
        ShadowCase::AllEqual => {
            // mul rd, rd, rd
            dynasm!(translator.emitter ; imul Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // mul rd, rd, rs2
            dynasm!(translator.emitter ; imul Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // mul rd, rs1, rd
            dynasm!(translator.emitter ; imul Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::Rs1EqRs2 => {
            // mul rd, rs1, rs1
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; imul Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // mul rd, rs1, rs2
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; imul Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `divu`: unsigned 64-bit division.
/// rd <- unsigned(rs1) / unsigned(rs2)
pub(super) fn emit_divu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
}

/// RV64 `remu`: unsigned 64-bit remainder.
/// rd <- unsigned(rs1) % unsigned(rs2)
pub(super) fn emit_remu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
}
