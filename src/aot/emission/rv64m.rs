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

/// RV64 `mulh`: upper 64 bits of signed 64x64 multiply.
/// rd <- high64(signed(rs1) * signed(rs2))
pub(super) fn emit_mulh(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    if rd.is_zero() {
        return;
    }

    if rs1.is_zero() || rs2.is_zero() {
        let ctx = InstructionContextBuilder::<0, 0>::new()
            .set_output(rd)
            .build(translator, temps);
        let rd = ctx.output();
        dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
        return;
    }

    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    dynasm!(translator.emitter ; mov Rq(X86Gpr::Rax.id()), Rq(rs1.id()));
    dynasm!(translator.emitter ; imul Rq(rs2.id()));
    if rd.id() != X86Gpr::Rdx.id() {
        dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(X86Gpr::Rdx.id()));
    }

    ctx.write_back(translator);
}

/// RV64 `mulhsu`: upper 64 bits of signed x unsigned 64x64 multiply.
/// rd <- high64(signed(rs1) * unsigned(rs2))
pub(super) fn emit_mulhsu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    if rd.is_zero() {
        return;
    }

    if rs1.is_zero() || rs2.is_zero() {
        let ctx = InstructionContextBuilder::<0, 0>::new()
            .set_output(rd)
            .build(translator, temps);
        let rd = ctx.output();
        dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
        return;
    }

    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    let non_negative_label = translator.emitter.new_dynamic_label();

    dynasm!(translator.emitter ; mov Rq(X86Gpr::Rax.id()), Rq(rs1.id()));
    dynasm!(translator.emitter ; mul Rq(rs2.id()));
    dynasm!(translator.emitter ; test Rq(rs1.id()), Rq(rs1.id()));
    dynasm!(translator.emitter ; jns => non_negative_label);
    dynasm!(translator.emitter ; sub Rq(X86Gpr::Rdx.id()), Rq(rs2.id()));
    dynasm!(translator.emitter ; => non_negative_label);

    if rd.id() != X86Gpr::Rdx.id() {
        dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(X86Gpr::Rdx.id()));
    }

    ctx.write_back(translator);
}

/// RV64 `mulw`: low 32 bits of multiply, sign-extended to 64 bits.
/// rd <- sext((rs1 * rs2)[31:0])
pub(super) fn emit_mulw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    if rd.is_zero() {
        return;
    }

    if rs1.is_zero() || rs2.is_zero() {
        let ctx = InstructionContextBuilder::<0, 0>::new()
            .set_output(rd)
            .build(translator, temps);
        let rd = ctx.output();
        dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
        return;
    }

    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    let scratch = temps.allocate().unwrap();
    dynasm!(translator.emitter ; mov Rd(scratch.id()), Rd(rs1.id()));
    dynasm!(translator.emitter ; imul Rd(scratch.id()), Rd(rs2.id()));
    dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(scratch.id()));

    ctx.write_back(translator);
}

/// RV64 `div`: signed 64-bit division.
/// rd <- signed(rs1) / signed(rs2)
pub(super) fn emit_div(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_signed_div_rem(translator, temps, rd, rs1, rs2, false, false);
}

/// RV64 `rem`: signed 64-bit remainder.
/// rd <- signed(rs1) % signed(rs2)
pub(super) fn emit_rem(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_signed_div_rem(translator, temps, rd, rs1, rs2, true, false);
}

/// RV64 `divw`: signed 32-bit division, sign-extended to 64 bits.
/// rd <- sext(signed(rs1[31:0]) / signed(rs2[31:0]))
pub(super) fn emit_divw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_signed_div_rem(translator, temps, rd, rs1, rs2, false, true);
}

/// RV64 `remw`: signed 32-bit remainder, sign-extended to 64 bits.
/// rd <- sext(signed(rs1[31:0]) % signed(rs2[31:0]))
pub(super) fn emit_remw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_signed_div_rem(translator, temps, rd, rs1, rs2, true, true);
}

/// RV64 `divuw`: unsigned 32-bit division, sign-extended to 64 bits.
/// rd <- sext(unsigned(rs1[31:0]) / unsigned(rs2[31:0]))
pub(super) fn emit_divuw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_unsigned_word_div_rem(translator, temps, rd, rs1, rs2, false);
}

/// RV64 `remuw`: unsigned 32-bit remainder, sign-extended to 64 bits.
/// rd <- sext(unsigned(rs1[31:0]) % unsigned(rs2[31:0]))
pub(super) fn emit_remuw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_unsigned_word_div_rem(translator, temps, rd, rs1, rs2, true);
}

fn emit_signed_div_rem(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    want_remainder: bool,
    word: bool,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    if rd.is_zero() {
        ctx.discard_zero_output(translator);
        return;
    }

    let divide_by_zero_label = translator.emitter.new_dynamic_label();
    let overflow_label = translator.emitter.new_dynamic_label();
    let safe_divide_label = translator.emitter.new_dynamic_label();
    let done_label = translator.emitter.new_dynamic_label();
    let min_temp = temps.allocate().unwrap();

    if word {
        dynasm!(translator.emitter ; movsxd Rq(X86Gpr::Rax.id()), Rd(rs1.id()));
        dynasm!(translator.emitter ; movsxd Rq(min_temp.id()), Rd(rs2.id()));
    } else {
        dynasm!(translator.emitter ; mov Rq(X86Gpr::Rax.id()), Rq(rs1.id()));
        dynasm!(translator.emitter ; mov Rq(min_temp.id()), Rq(rs2.id()));
    }

    dynasm!(translator.emitter ; test Rq(min_temp.id()), Rq(min_temp.id()));
    dynasm!(translator.emitter ; jz => divide_by_zero_label);

    if word {
        dynasm!(translator.emitter ; cmp Rq(X86Gpr::Rax.id()), -2147483648i32);
        dynasm!(translator.emitter ; jne => safe_divide_label);
    } else {
        // i64::MIN is the only negative value whose low 63 bits are all zero.
        dynasm!(translator.emitter ; mov Rq(min_temp.id()), Rq(X86Gpr::Rax.id()));
        dynasm!(translator.emitter ; shl Rq(min_temp.id()), 1);
        dynasm!(translator.emitter ; jne => safe_divide_label);
        dynasm!(translator.emitter ; test Rq(X86Gpr::Rax.id()), Rq(X86Gpr::Rax.id()));
        dynasm!(translator.emitter ; jns => safe_divide_label);
    }
    if word {
        dynasm!(translator.emitter ; cmp Rd(rs2.id()), -1);
    } else {
        dynasm!(translator.emitter ; cmp Rq(rs2.id()), -1);
    }
    dynasm!(translator.emitter ; je => overflow_label);

    dynasm!(translator.emitter ; => safe_divide_label);
    dynasm!(translator.emitter ; cqo);
    if word {
        dynasm!(translator.emitter ; idiv Rq(min_temp.id()));
    } else {
        dynasm!(translator.emitter ; idiv Rq(rs2.id()));
    }
    if want_remainder {
        dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(X86Gpr::Rdx.id()));
    } else {
        dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(X86Gpr::Rax.id()));
    }
    if word {
        dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
    }
    dynasm!(translator.emitter ; jmp => done_label);

    dynasm!(translator.emitter ; => divide_by_zero_label);
    if want_remainder {
        if word {
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rs1.id()));
        } else {
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
        }
    } else {
        dynasm!(translator.emitter ; mov Rq(rd.id()), -1);
    }
    dynasm!(translator.emitter ; jmp => done_label);

    dynasm!(translator.emitter ; => overflow_label);
    if want_remainder {
        dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
    } else {
        dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(X86Gpr::Rax.id()));
    }

    dynasm!(translator.emitter ; => done_label);
    ctx.write_back(translator);
}

fn emit_unsigned_word_div_rem(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    want_remainder: bool,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    if rd.is_zero() {
        ctx.discard_zero_output(translator);
        return;
    }

    let divide_by_zero_label = translator.emitter.new_dynamic_label();
    let done_label = translator.emitter.new_dynamic_label();
    let divisor = temps.allocate().unwrap();

    dynasm!(translator.emitter ; mov Rd(X86Gpr::Rax.id()), Rd(rs1.id()));
    dynasm!(translator.emitter ; mov Rd(divisor.id()), Rd(rs2.id()));
    dynasm!(translator.emitter ; test Rd(divisor.id()), Rd(divisor.id()));
    dynasm!(translator.emitter ; jz => divide_by_zero_label);
    dynasm!(translator.emitter ; xor Rq(X86Gpr::Rdx.id()), Rq(X86Gpr::Rdx.id()));
    dynasm!(translator.emitter ; div Rd(divisor.id()));
    if want_remainder {
        dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(X86Gpr::Rdx.id()));
    } else {
        dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(X86Gpr::Rax.id()));
    }
    dynasm!(translator.emitter ; jmp => done_label);

    dynasm!(translator.emitter ; => divide_by_zero_label);
    if want_remainder {
        dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rs1.id()));
    } else {
        dynasm!(translator.emitter ; mov Rq(rd.id()), -1);
    }

    dynasm!(translator.emitter ; => done_label);
    ctx.write_back(translator);
}
