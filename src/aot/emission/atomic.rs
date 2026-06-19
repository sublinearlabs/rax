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

/// RV32A `lr.w`: load-reserved word.
/// rd <- sign_extend(M[rs1][31:0])
#[allow(unused_variables)]
pub(super) fn emit_lr_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A lr.w emission.
}

/// RV32A `sc.w`: store-conditional word.
/// rd <- success ? 0 : nonzero; M[rs1][31:0] <- rs2[31:0] on success
#[allow(unused_variables)]
pub(super) fn emit_sc_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A sc.w emission.
}

/// RV32A `amoswap.w`: atomically swap word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- rs2[31:0]
#[allow(unused_variables)]
pub(super) fn emit_amo_swap_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amoswap.w emission.
}

/// RV32A `amoadd.w`: atomically add word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- M[rs1][31:0] + rs2[31:0]
#[allow(unused_variables)]
pub(super) fn emit_amo_add_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amoadd.w emission.
}

/// RV32A `amoxor.w`: atomically xor word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- M[rs1][31:0] ^ rs2[31:0]
#[allow(unused_variables)]
pub(super) fn emit_amo_xor_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amoxor.w emission.
}

/// RV32A `amoand.w`: atomically and word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- M[rs1][31:0] & rs2[31:0]
#[allow(unused_variables)]
pub(super) fn emit_amo_and_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amoand.w emission.
}

/// RV32A `amoor.w`: atomically or word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- M[rs1][31:0] | rs2[31:0]
#[allow(unused_variables)]
pub(super) fn emit_amo_or_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amoor.w emission.
}

/// RV32A `amomin.w`: atomically signed-min word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- min_signed(M[rs1][31:0], rs2[31:0])
#[allow(unused_variables)]
pub(super) fn emit_amo_min_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amomin.w emission.
}

/// RV32A `amomax.w`: atomically signed-max word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- max_signed(M[rs1][31:0], rs2[31:0])
#[allow(unused_variables)]
pub(super) fn emit_amo_max_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amomax.w emission.
}

/// RV32A `amominu.w`: atomically unsigned-min word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- min_unsigned(M[rs1][31:0], rs2[31:0])
#[allow(unused_variables)]
pub(super) fn emit_amo_minu_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amominu.w emission.
}

/// RV32A `amomaxu.w`: atomically unsigned-max word.
/// rd <- sign_extend(M[rs1][31:0]); M[rs1][31:0] <- max_unsigned(M[rs1][31:0], rs2[31:0])
#[allow(unused_variables)]
pub(super) fn emit_amo_maxu_w(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV32A amomaxu.w emission.
}

/// RV64A `lr.d`: load-reserved doubleword.
/// rd <- M[rs1][63:0]
#[allow(unused_variables)]
pub(super) fn emit_lr_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A lr.d emission.
}

/// RV64A `sc.d`: store-conditional doubleword.
/// rd <- success ? 0 : nonzero; M[rs1][63:0] <- rs2 on success
#[allow(unused_variables)]
pub(super) fn emit_sc_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A sc.d emission.
}

/// RV64A `amoswap.d`: atomically swap doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- rs2
#[allow(unused_variables)]
pub(super) fn emit_amo_swap_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amoswap.d emission.
}

/// RV64A `amoadd.d`: atomically add doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- M[rs1][63:0] + rs2
#[allow(unused_variables)]
pub(super) fn emit_amo_add_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amoadd.d emission.
}

/// RV64A `amoxor.d`: atomically xor doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- M[rs1][63:0] ^ rs2
#[allow(unused_variables)]
pub(super) fn emit_amo_xor_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amoxor.d emission.
}

/// RV64A `amoand.d`: atomically and doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- M[rs1][63:0] & rs2
#[allow(unused_variables)]
pub(super) fn emit_amo_and_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amoand.d emission.
}

/// RV64A `amoor.d`: atomically or doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- M[rs1][63:0] | rs2
#[allow(unused_variables)]
pub(super) fn emit_amo_or_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amoor.d emission.
}

/// RV64A `amomin.d`: atomically signed-min doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- min_signed(M[rs1][63:0], rs2)
#[allow(unused_variables)]
pub(super) fn emit_amo_min_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amomin.d emission.
}

/// RV64A `amomax.d`: atomically signed-max doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- max_signed(M[rs1][63:0], rs2)
#[allow(unused_variables)]
pub(super) fn emit_amo_max_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amomax.d emission.
}

/// RV64A `amominu.d`: atomically unsigned-min doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- min_unsigned(M[rs1][63:0], rs2)
#[allow(unused_variables)]
pub(super) fn emit_amo_minu_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amominu.d emission.
}

/// RV64A `amomaxu.d`: atomically unsigned-max doubleword.
/// rd <- M[rs1][63:0]; M[rs1][63:0] <- max_unsigned(M[rs1][63:0], rs2)
#[allow(unused_variables)]
pub(super) fn emit_amo_maxu_d(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64A amomaxu.d emission.
}
