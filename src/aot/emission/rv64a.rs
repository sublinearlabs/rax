use crate::aot::{
    emission::rv64i::{emit_ld, emit_lw},
    registers::RiscvRegister,
    temp_alloc::TempAllocator,
    translator::Translator,
};

/// RV64 `lr.w`: load-reserved word.
/// rd <- M[rs1][31:0]; reserve for later store-conditional
pub(super) fn emit_lrw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    _rs2: RiscvRegister,
) {
    // NOTE: this delegation is only safe for single core
    emit_lw(translator, temps, rd, rs1, 0);
}

/// RV64 `lr.d`: load-reserved doubleword.
/// rd <- M[rs1][63:0]; reserve for later store-conditional
pub(super) fn emit_lrd(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // NOTE: this delegation is only safe for single core
    emit_ld(translator, temps, rd, rs1, 0);
}

/// RV64 `sc.w`: store-conditional word.
/// if reservation held then M[rs1] <- rs2[31:0], rd <- 0 else rd <- 1
#[allow(unused_variables)]
pub(super) fn emit_scw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
}

/// RV64 `sc.d`: store-conditional doubleword.
/// if reservation held then M[rs1] <- rs2[63:0], rd <- 0 else rd <- 1
#[allow(unused_variables)]
pub(super) fn emit_scd(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
}

/// RV64 `amoswap.w`: atomic swap word.
/// rd <- M[rs1]; M[rs1] <- rs2[31:0]
#[allow(unused_variables)]
pub(super) fn emit_amoaddw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
}

/// RV64 `amoswap.d`: atomic swap doubleword.
/// rd <- M[rs1]; M[rs1] <- rs2[63:0]
#[allow(unused_variables)]
pub(super) fn emit_amoaddd(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
}

/// RV64 `amoor.w`: atomic OR word.
/// rd <- M[rs1]; M[rs1] <- M[rs1] | rs2[31:0]
#[allow(unused_variables)]
pub(super) fn emit_amoorw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
}

/// RV64 `amoor.d`: atomic OR doubleword.
/// rd <- M[rs1]; M[rs1] <- M[rs1] | rs2[63:0]
#[allow(unused_variables)]
pub(super) fn emit_amoodd(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
}
