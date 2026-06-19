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

/// RV64M `mul`: lower 64 bits of signed 64x64 multiply.
/// rd <- low64(rs1 * rs2)
#[allow(unused_variables)]
pub(super) fn emit_mul(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M mul emission.
}

/// RV64M `mulh`: upper 64 bits of signed 64x64 multiply.
/// rd <- high64(signed(rs1) * signed(rs2))
#[allow(unused_variables)]
pub(super) fn emit_mulh(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M mulh emission.
}

/// RV64M `mulhsu`: upper 64 bits of signed-by-unsigned 64x64 multiply.
/// rd <- high64(signed(rs1) * unsigned(rs2))
#[allow(unused_variables)]
pub(super) fn emit_mulhsu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M mulhsu emission.
}

/// RV64 `mulhu`: upper 64 bits of unsigned 64x64 multiply.
/// rd <- high64(unsigned(rs1) * unsigned(rs2))
pub(super) fn emit_mulhu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // x86 mul r/m64 uses the rdx and rax as implicit registers
    // RDX:RAX = RAX * r/m64
    // high XLEN bits of the multiplication are in RDX
    // low  XLEN bits of the multiplication are in RAX

    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rax, X86Gpr::Rdx])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            // x0 is hardwired to zero, writes can be ignored
            ctx.discard_zero_output(translator);
            return;
        }
        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero | ZeroCase::Rs2Zero => {
            // in all of these cases, rd should be set to zero
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }
        ZeroCase::None => {}
    }

    // I am not handling the shadow case here because,
    // for most of them, it is pretty difficult to beat the
    // 3 instruction baseline:
    // mov rax, rs1;
    // mul rs2;
    // mov rd, rdx;

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

/// RV64M `div`: signed 64-bit division.
/// rd <- signed(rs1) / signed(rs2)
#[allow(unused_variables)]
pub(super) fn emit_div(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M div emission.
}

/// RV64M `divu`: unsigned 64-bit division.
/// rd <- unsigned(rs1) / unsigned(rs2)
#[allow(unused_variables)]
pub(super) fn emit_divu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M divu emission.
}

/// RV64M `rem`: signed 64-bit remainder.
/// rd <- signed(rs1) % signed(rs2)
#[allow(unused_variables)]
pub(super) fn emit_rem(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M rem emission.
}

/// RV64M `remu`: unsigned 64-bit remainder.
/// rd <- unsigned(rs1) % unsigned(rs2)
#[allow(unused_variables)]
pub(super) fn emit_remu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M remu emission.
}

/// RV64M `mulw`: lower 32 bits of multiply, then sign-extend to 64 bits.
/// rd <- sext32(rs1[31:0] * rs2[31:0])
#[allow(unused_variables)]
pub(super) fn emit_mulw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M mulw emission.
}

/// RV64M `divw`: signed 32-bit division, then sign-extend to 64 bits.
/// rd <- sext32(signed(rs1[31:0]) / signed(rs2[31:0]))
#[allow(unused_variables)]
pub(super) fn emit_divw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M divw emission.
}

/// RV64M `divuw`: unsigned 32-bit division, then sign-extend to 64 bits.
/// rd <- sext32(unsigned(rs1[31:0]) / unsigned(rs2[31:0]))
#[allow(unused_variables)]
pub(super) fn emit_divuw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M divuw emission.
}

/// RV64M `remw`: signed 32-bit remainder, then sign-extend to 64 bits.
/// rd <- sext32(signed(rs1[31:0]) % signed(rs2[31:0]))
#[allow(unused_variables)]
pub(super) fn emit_remw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M remw emission.
}

/// RV64M `remuw`: unsigned 32-bit remainder, then sign-extend to 64 bits.
/// rd <- sext32(unsigned(rs1[31:0]) % unsigned(rs2[31:0]))
#[allow(unused_variables)]
pub(super) fn emit_remuw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M remuw emission.
}
