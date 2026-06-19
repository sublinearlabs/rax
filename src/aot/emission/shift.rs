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

/// RV64 `sll`: logical left shift by register low bits.
/// rd <- rs1 << (rs2 & 0x3f)
pub(super) fn emit_sll(
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
pub(super) fn emit_srl(
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
pub(super) fn emit_sra(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 sra emission.
}
