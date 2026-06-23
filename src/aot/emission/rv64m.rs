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
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs2Zero => {
            // case 1
            // divu rd, 0, 0
            //
            // case 2
            // divu rd, rs1, 0
            //
            // since both cases divide by 0
            // -> rd = -1
            dynasm!(translator.emitter ; mov Rq(rd.id()), -1);
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // divu rd, 0, rs2
            // it is possible that rs2 might be 0
            // so rd can be -1 or 0
            // we can't distinguish this at compile time
            // so we fall back to the generic handler
        }

        ZeroCase::None => {}
    }

    let safe_divide_label = translator.emitter.new_dynamic_label();
    let done_label = translator.emitter.new_dynamic_label();

    if rs1.id() != X86Gpr::Rax.id() {
        dynasm!(translator.emitter ; mov Rq(X86Gpr::Rax.id()), Rq(rs1.id()));
    }

    dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
    // if rs2 != 0 then it is safe to divide
    dynasm!(translator.emitter ; jnz => safe_divide_label);
    // rs2 == 0, so we set rd = -1
    dynasm!(translator.emitter ; mov Rq(rd.id()), -1);
    dynasm!(translator.emitter ; jmp => done_label);

    // SAFE DIVIDE
    // -----------
    dynasm!(translator.emitter ; => safe_divide_label);
    // zero extend Rax into Rdx
    dynasm!(translator.emitter ; xor Rq(X86Gpr::Rdx.id()), Rq(X86Gpr::Rdx.id()));
    // perform Rdx:Rax / rs2
    dynasm!(translator.emitter ; div Rq(rs2.id()));
    // move rax (quotient) into rd
    if rd.id() != X86Gpr::Rax.id() {
        dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(X86Gpr::Rax.id()));
    }

    // DONE
    // ----
    dynasm!(translator.emitter ; => done_label);

    ctx.write_back(translator);
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
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
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
            // remu rd, 0, 0
            // when dividing by 0 return rs1
            // -> rd = 0
            //
            // case 2
            // remu rd, 0, rs2
            // if rs2 is zero, rd = 0
            // if rs2 is not zero, rd is still 0
            // (as 0 divided by anything is 0)
            // -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // remu rd, rs1, 0
            // dividing by 0, return rs1
            // -> rd = rs1
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

    let safe_divide_label = translator.emitter.new_dynamic_label();
    let done_label = translator.emitter.new_dynamic_label();

    if rs1.id() != X86Gpr::Rax.id() {
        dynasm!(translator.emitter ; mov Rq(X86Gpr::Rax.id()), Rq(rs1.id()));
    }

    dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
    // if rs2 != 0 then it is safe to divide
    dynasm!(translator.emitter ; jnz => safe_divide_label);
    // rs2 == 0, so rd = rs1
    if rd.id() != rs1.id() {
        dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
    }
    dynasm!(translator.emitter ; jmp => done_label);

    // SAFE DIVIDE
    // -----------
    dynasm!(translator.emitter ; => safe_divide_label);
    // zero extend Rax into Rdx
    dynasm!(translator.emitter ; xor Rq(X86Gpr::Rdx.id()), Rq(X86Gpr::Rdx.id()));
    // perform Rdx:Rax / rs2
    dynasm!(translator.emitter ; div Rq(rs2.id()));
    // move rdx (remainder) into rd
    if rd.id() != X86Gpr::Rdx.id() {
        dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(X86Gpr::Rdx.id()));
    }

    // DONE
    // ----
    dynasm!(translator.emitter ; => done_label);

    ctx.write_back(translator);
}
