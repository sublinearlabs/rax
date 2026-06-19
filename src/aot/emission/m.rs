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
#[allow(unused_variables)]
pub(super) fn emit_mulhu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64M mulhu emission.
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
