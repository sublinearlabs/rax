use dynasmrt::{dynasm, DynasmApi};

use crate::aot::{
    emission::rv64i::{emit_ld, emit_lw},
    instruction_context::InstructionContextBuilder,
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
    _rs2: RiscvRegister,
) {
    // NOTE: this delegation is only safe for single core
    emit_ld(translator, temps, rd, rs1, 0);
}

/// RV64 `sc.w`: store-conditional word.
/// if reservation held then M[rs1] <- rs2[31:0], rd <- 0 else rd <- 1
pub(super) fn emit_scw(
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

    let addr_temp;
    let addr_id = if rs1.is_zero() {
        addr_temp = temps.allocate().unwrap();
        dynasm!(translator.emitter ; xor Rq(addr_temp.id()), Rq(addr_temp.id()));
        addr_temp.id()
    } else {
        rs1.id()
    };

    if rs2.is_zero() {
        dynasm!(translator.emitter ; mov DWORD [Rq(addr_id)], 0);
    } else {
        dynasm!(translator.emitter ; mov DWORD [Rq(addr_id)], Rd(rs2.id()));
    }

    if rd.is_zero() {
        ctx.discard_zero_output(translator);
    } else {
        dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
    }
}

/// RV64 `sc.d`: store-conditional doubleword.
/// if reservation held then M[rs1] <- rs2[63:0], rd <- 0 else rd <- 1
pub(super) fn emit_scd(
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

    let addr_temp;
    let addr_id = if rs1.is_zero() {
        addr_temp = temps.allocate().unwrap();
        dynasm!(translator.emitter ; xor Rq(addr_temp.id()), Rq(addr_temp.id()));
        addr_temp.id()
    } else {
        rs1.id()
    };

    if rs2.is_zero() {
        dynasm!(translator.emitter ; mov QWORD [Rq(addr_id)], 0);
    } else {
        dynasm!(translator.emitter ; mov QWORD [Rq(addr_id)], Rq(rs2.id()));
    }

    if rd.is_zero() {
        ctx.discard_zero_output(translator);
    } else {
        dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
    }
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
