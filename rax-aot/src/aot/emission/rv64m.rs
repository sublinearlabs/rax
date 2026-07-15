use crate::aot::emit_asm;

use crate::aot::{
    classification::{classify_shadow_case, classify_zero_case, ShadowCase, ZeroCase},
    instruction_context::InstructionContextBuilder,
    registers::{RiscvRegister, X86Gpr},
    translator::Translator,
};

/// RV64 `mulhu`: upper 64 bits of unsigned 64x64 multiply.
/// rd <- high64(unsigned(rs1) * unsigned(rs2))
pub(super) fn emit_mulhu(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }
        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero | ZeroCase::Rs2Zero => {
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }
        ZeroCase::None => {}
    }

    if rs1.id() == X86Gpr::Rax.id() {
        emit_asm!(translator ; mul Rq(rs2.id()));
    } else if rs2.id() == X86Gpr::Rax.id() {
        emit_asm!(translator ; mul Rq(rs1.id()));
    } else {
        emit_asm!(translator ; mov Rq(X86Gpr::Rax.id()), Rq(rs1.id()));
        emit_asm!(translator ; mul Rq(rs2.id()));
    }

    if rd.id() != X86Gpr::Rdx.id() {
        emit_asm!(translator ; mov Rq(rd.id()), Rq(X86Gpr::Rdx.id()));
    }

    ctx.write_back(translator);
}

/// RV64 `mul`: low 64 bits of signed 64x64 multiply.
/// rd <- low64(signed(rs1) * signed(rs2))
pub(super) fn emit_mul(
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

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero | ZeroCase::Rs2Zero => {
            // since at least one of the multiplication arguments
            // are zero, the result will be zero
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(rd, rs1, rs2) {
        ShadowCase::AllEqual => {
            // mul rd, rd, rd
            emit_asm!(translator ; imul Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // mul rd, rd, rs2
            emit_asm!(translator ; imul Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // mul rd, rs1, rd
            emit_asm!(translator ; imul Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::Rs1EqRs2 => {
            // mul rd, rs1, rs1
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; imul Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // mul rd, rs1, rs2
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; imul Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `divu`: unsigned 64-bit division.
/// rd <- unsigned(rs1) / unsigned(rs2)
pub(super) fn emit_divu(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
        .build(translator);

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
            emit_asm!(translator ; mov Rq(rd.id()), -1);
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

    let safe_divide_label = translator.new_dynamic_label();
    let done_label = translator.new_dynamic_label();

    if rs1.id() != X86Gpr::Rax.id() {
        emit_asm!(translator ; mov Rq(X86Gpr::Rax.id()), Rq(rs1.id()));
    }

    emit_asm!(translator ; test Rq(rs2.id()), Rq(rs2.id()));
    // if rs2 != 0 then it is safe to divide
    emit_asm!(translator ; jnz => safe_divide_label);
    // rs2 == 0, so we set rd = -1
    emit_asm!(translator ; mov Rq(rd.id()), -1);
    emit_asm!(translator ; jmp => done_label);

    // SAFE DIVIDE
    // -----------
    emit_asm!(translator ; => safe_divide_label);
    // zero extend Rax into Rdx
    emit_asm!(translator ; xor Rq(X86Gpr::Rdx.id()), Rq(X86Gpr::Rdx.id()));
    // perform Rdx:Rax / rs2
    emit_asm!(translator ; div Rq(rs2.id()));
    // move rax (quotient) into rd
    if rd.id() != X86Gpr::Rax.id() {
        emit_asm!(translator ; mov Rq(rd.id()), Rq(X86Gpr::Rax.id()));
    }

    // DONE
    // ----
    emit_asm!(translator ; => done_label);

    ctx.write_back(translator);
}

/// RV64 `remu`: unsigned 64-bit remainder.
/// rd <- unsigned(rs1) % unsigned(rs2)
pub(super) fn emit_remu(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
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
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
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

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    let safe_divide_label = translator.new_dynamic_label();
    let done_label = translator.new_dynamic_label();

    if rs1.id() != X86Gpr::Rax.id() {
        emit_asm!(translator ; mov Rq(X86Gpr::Rax.id()), Rq(rs1.id()));
    }

    emit_asm!(translator ; test Rq(rs2.id()), Rq(rs2.id()));
    // if rs2 != 0 then it is safe to divide
    emit_asm!(translator ; jnz => safe_divide_label);
    // rs2 == 0, so rd = rs1
    if rd.id() != rs1.id() {
        emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
    }
    emit_asm!(translator ; jmp => done_label);

    // SAFE DIVIDE
    // -----------
    emit_asm!(translator ; => safe_divide_label);
    // zero extend Rax into Rdx
    emit_asm!(translator ; xor Rq(X86Gpr::Rdx.id()), Rq(X86Gpr::Rdx.id()));
    // perform Rdx:Rax / rs2
    emit_asm!(translator ; div Rq(rs2.id()));
    // move rdx (remainder) into rd
    if rd.id() != X86Gpr::Rdx.id() {
        emit_asm!(translator ; mov Rq(rd.id()), Rq(X86Gpr::Rdx.id()));
    }

    // DONE
    // ----
    emit_asm!(translator ; => done_label);

    ctx.write_back(translator);
}

/// RV64 `mulh`: upper 64 bits of signed 64x64 multiply.
/// rd <- high64(signed(rs1) * signed(rs2))
pub(super) fn emit_mulh(
    translator: &Translator,
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
            .build(translator);
        let rd = ctx.output();
        emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
        return;
    }

    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    emit_asm!(translator ; mov Rq(X86Gpr::Rax.id()), Rq(rs1.id()));
    emit_asm!(translator ; imul Rq(rs2.id()));
    if rd.id() != X86Gpr::Rdx.id() {
        emit_asm!(translator ; mov Rq(rd.id()), Rq(X86Gpr::Rdx.id()));
    }

    ctx.write_back(translator);
}

/// RV64 `mulhsu`: upper 64 bits of signed x unsigned 64x64 multiply.
/// rd <- high64(signed(rs1) * unsigned(rs2))
pub(super) fn emit_mulhsu(
    translator: &Translator,
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
            .build(translator);
        let rd = ctx.output();
        emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
        return;
    }

    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    let non_negative_label = translator.new_dynamic_label();

    emit_asm!(translator ; mov Rq(X86Gpr::Rax.id()), Rq(rs1.id()));
    emit_asm!(translator ; mul Rq(rs2.id()));
    emit_asm!(translator ; test Rq(rs1.id()), Rq(rs1.id()));
    emit_asm!(translator ; jns => non_negative_label);
    emit_asm!(translator ; sub Rq(X86Gpr::Rdx.id()), Rq(rs2.id()));
    emit_asm!(translator ; => non_negative_label);

    if rd.id() != X86Gpr::Rdx.id() {
        emit_asm!(translator ; mov Rq(rd.id()), Rq(X86Gpr::Rdx.id()));
    }

    ctx.write_back(translator);
}

/// RV64 `mulw`: low 32 bits of multiply, sign-extended to 64 bits.
/// rd <- sext((rs1 * rs2)[31:0])
pub(super) fn emit_mulw(
    translator: &Translator,
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
            .build(translator);
        let rd = ctx.output();
        emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
        return;
    }

    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    let scratch = translator.temp_pool.allocate().unwrap();
    emit_asm!(translator ; mov Rd(scratch.id()), Rd(rs1.id()));
    emit_asm!(translator ; imul Rd(scratch.id()), Rd(rs2.id()));
    emit_asm!(translator ; movsxd Rq(rd.id()), Rd(scratch.id()));

    ctx.write_back(translator);
}

/// RV64 `div`: signed 64-bit division.
/// rd <- signed(rs1) / signed(rs2)
pub(super) fn emit_div(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_signed_div_rem(translator, rd, rs1, rs2, false, false);
}

/// RV64 `rem`: signed 64-bit remainder.
/// rd <- signed(rs1) % signed(rs2)
pub(super) fn emit_rem(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_signed_div_rem(translator, rd, rs1, rs2, true, false);
}

/// RV64 `divw`: signed 32-bit division, sign-extended to 64 bits.
/// rd <- sext(signed(rs1[31:0]) / signed(rs2[31:0]))
pub(super) fn emit_divw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_signed_div_rem(translator, rd, rs1, rs2, false, true);
}

/// RV64 `remw`: signed 32-bit remainder, sign-extended to 64 bits.
/// rd <- sext(signed(rs1[31:0]) % signed(rs2[31:0]))
pub(super) fn emit_remw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_signed_div_rem(translator, rd, rs1, rs2, true, true);
}

/// RV64 `divuw`: unsigned 32-bit division, sign-extended to 64 bits.
/// rd <- sext(unsigned(rs1[31:0]) / unsigned(rs2[31:0]))
pub(super) fn emit_divuw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_unsigned_word_div_rem(translator, rd, rs1, rs2, false);
}

/// RV64 `remuw`: unsigned 32-bit remainder, sign-extended to 64 bits.
/// rd <- sext(unsigned(rs1[31:0]) % unsigned(rs2[31:0]))
pub(super) fn emit_remuw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_unsigned_word_div_rem(translator, rd, rs1, rs2, true);
}

fn emit_signed_div_rem(
    translator: &Translator,
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
        .build(translator);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    if rd.is_zero() {
        ctx.discard_zero_output(translator);
        return;
    }

    let divide_by_zero_label = translator.new_dynamic_label();
    let overflow_label = translator.new_dynamic_label();
    let safe_divide_label = translator.new_dynamic_label();
    let done_label = translator.new_dynamic_label();
    let min_temp = translator.temp_pool.allocate().unwrap();

    if word {
        emit_asm!(translator ; movsxd Rq(X86Gpr::Rax.id()), Rd(rs1.id()));
        emit_asm!(translator ; movsxd Rq(min_temp.id()), Rd(rs2.id()));
    } else {
        emit_asm!(translator ; mov Rq(X86Gpr::Rax.id()), Rq(rs1.id()));
        emit_asm!(translator ; mov Rq(min_temp.id()), Rq(rs2.id()));
    }

    emit_asm!(translator ; test Rq(min_temp.id()), Rq(min_temp.id()));
    emit_asm!(translator ; jz => divide_by_zero_label);

    if word {
        emit_asm!(translator ; cmp Rq(X86Gpr::Rax.id()), -2147483648i32);
        emit_asm!(translator ; jne => safe_divide_label);
    } else {
        // i64::MIN is the only negative value whose low 63 bits are all zero.
        emit_asm!(translator ; mov Rq(min_temp.id()), Rq(X86Gpr::Rax.id()));
        emit_asm!(translator ; shl Rq(min_temp.id()), 1);
        emit_asm!(translator ; jne => safe_divide_label);
        emit_asm!(translator ; test Rq(X86Gpr::Rax.id()), Rq(X86Gpr::Rax.id()));
        emit_asm!(translator ; jns => safe_divide_label);
    }
    if word {
        emit_asm!(translator ; cmp Rd(rs2.id()), -1);
    } else {
        emit_asm!(translator ; cmp Rq(rs2.id()), -1);
    }
    emit_asm!(translator ; je => overflow_label);

    emit_asm!(translator ; => safe_divide_label);
    emit_asm!(translator ; cqo);
    if word {
        emit_asm!(translator ; idiv Rq(min_temp.id()));
    } else {
        emit_asm!(translator ; idiv Rq(rs2.id()));
    }
    if want_remainder {
        emit_asm!(translator ; mov Rq(rd.id()), Rq(X86Gpr::Rdx.id()));
    } else {
        emit_asm!(translator ; mov Rq(rd.id()), Rq(X86Gpr::Rax.id()));
    }
    if word {
        emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rd.id()));
    }
    emit_asm!(translator ; jmp => done_label);

    emit_asm!(translator ; => divide_by_zero_label);
    if want_remainder {
        if word {
            emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rs1.id()));
        } else {
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
        }
    } else {
        emit_asm!(translator ; mov Rq(rd.id()), -1);
    }
    emit_asm!(translator ; jmp => done_label);

    emit_asm!(translator ; => overflow_label);
    if want_remainder {
        emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
    } else {
        emit_asm!(translator ; mov Rq(rd.id()), Rq(X86Gpr::Rax.id()));
    }

    emit_asm!(translator ; => done_label);
    ctx.write_back(translator);
}

fn emit_unsigned_word_div_rem(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    want_remainder: bool,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    if rd.is_zero() {
        ctx.discard_zero_output(translator);
        return;
    }

    let divide_by_zero_label = translator.new_dynamic_label();
    let done_label = translator.new_dynamic_label();
    let divisor = translator.temp_pool.allocate().unwrap();

    emit_asm!(translator ; mov Rd(X86Gpr::Rax.id()), Rd(rs1.id()));
    emit_asm!(translator ; mov Rd(divisor.id()), Rd(rs2.id()));
    emit_asm!(translator ; test Rd(divisor.id()), Rd(divisor.id()));
    emit_asm!(translator ; jz => divide_by_zero_label);
    emit_asm!(translator ; xor Rq(X86Gpr::Rdx.id()), Rq(X86Gpr::Rdx.id()));
    emit_asm!(translator ; div Rd(divisor.id()));
    if want_remainder {
        emit_asm!(translator ; movsxd Rq(rd.id()), Rd(X86Gpr::Rdx.id()));
    } else {
        emit_asm!(translator ; movsxd Rq(rd.id()), Rd(X86Gpr::Rax.id()));
    }
    emit_asm!(translator ; jmp => done_label);

    emit_asm!(translator ; => divide_by_zero_label);
    if want_remainder {
        emit_asm!(translator ; movsxd Rq(rd.id()), Rd(rs1.id()));
    } else {
        emit_asm!(translator ; mov Rq(rd.id()), -1);
    }

    emit_asm!(translator ; => done_label);
    ctx.write_back(translator);
}
