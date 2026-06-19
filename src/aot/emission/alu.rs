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

/// RV64 `add`: 64-bit wrapping addition.
/// rd <- (rs1 + rs2) mod 2^64
pub(super) fn emit_add(
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
pub(super) fn emit_sub(
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
pub(super) fn emit_xor(
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
pub(super) fn emit_or(
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
pub(super) fn emit_and(
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
pub(super) fn emit_slt(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 slt emission.
}
