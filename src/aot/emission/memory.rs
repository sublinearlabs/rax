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

/// RV64 `lb`: load signed byte.
/// rd <- sign_extend(M[rs1 + imm][7:0])
#[allow(unused_variables)]
pub(super) fn emit_lb(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 lb emission.
}

/// RV64 `lh`: load signed halfword.
/// rd <- sign_extend(M[rs1 + imm][15:0])
#[allow(unused_variables)]
pub(super) fn emit_lh(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 lh emission.
}

/// RV64 `lw`: load signed word.
/// rd <- sign_extend(M[rs1 + imm][31:0])
#[allow(unused_variables)]
pub(super) fn emit_lw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 lw emission.
}

/// RV64 `lwu`: load unsigned word.
/// rd <- zero_extend(M[rs1 + imm][31:0])
#[allow(unused_variables)]
pub(super) fn emit_lwu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 lwu emission.
}

/// RV64 `ld`: load 64-bit value.
/// rd <- M[rs1 + imm][63:0]
#[allow(unused_variables)]
pub(super) fn emit_ld(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 ld emission.
}

/// RV64 `lbu`: load unsigned byte.
/// rd <- zero_extend(M[rs1 + imm][7:0])
#[allow(unused_variables)]
pub(super) fn emit_lbu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 lbu emission.
}

/// RV64 `lhu`: load unsigned halfword.
/// rd <- zero_extend(M[rs1 + imm][15:0])
#[allow(unused_variables)]
pub(super) fn emit_lhu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 lhu emission.
}

/// RV64 `sb`: store low 8 bits of rs2 to memory at rs1 + sext(imm).
/// mem8[rs1 + sext(imm)] <- rs2[7:0]
pub(super) fn emit_sb(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();

    // `x0` has no backing x86 register, but this x86 memory operand needs a
    // base register. Materialize `x0 + imm` into a temp and use no displacement.
    let (addr_id, addr_disp, _addr_temp) = if rs1.is_zero() {
        let temp = temps
            .allocate()
            .expect("emit_sb requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(temp.id()), QWORD imm as i64);
        (temp.id(), 0, Some(temp))
    } else {
        (rs1.id(), imm, None)
    };

    if rs2.is_zero() {
        dynasm!(translator.emitter ; mov BYTE [Rq(addr_id) + addr_disp], 0_i8);
    } else {
        dynasm!(translator.emitter ; mov BYTE [Rq(addr_id) + addr_disp], Rb(rs2.id()));
    }
    ctx.complete_no_output(translator);
}

/// RV64 `sh`: store low 16 bits of rs2 to memory at rs1 + sext(imm).
/// mem16[rs1 + sext(imm)] <- rs2[15:0]
#[allow(unused_variables)]
pub(super) fn emit_sh(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 sh emission.
}

/// RV64 `sw`: store low 32 bits of rs2 to memory at rs1 + sext(imm).
/// mem32[rs1 + sext(imm)] <- rs2[31:0]
#[allow(unused_variables)]
pub(super) fn emit_sw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 sw emission.
}

/// RV64 `sd`: store 64 bits of rs2 to memory at rs1 + sext(imm).
/// mem64[rs1 + sext(imm)] <- rs2
pub(super) fn emit_sd(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();

    // `x0` has no backing x86 register, but this x86 memory operand needs a
    // base register. Materialize `x0 + imm` into a temp and use no displacement.
    let (addr_id, addr_disp, _addr_temp) = if rs1.is_zero() {
        let temp = temps
            .allocate()
            .expect("emit_sd requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(temp.id()), QWORD imm as i64);
        (temp.id(), 0, Some(temp))
    } else {
        (rs1.id(), imm, None)
    };

    if rs2.is_zero() {
        dynasm!(translator.emitter ; mov QWORD [Rq(addr_id) + addr_disp], 0_i32);
    } else {
        dynasm!(translator.emitter ; mov QWORD [Rq(addr_id) + addr_disp], Rq(rs2.id()));
    }

    ctx.complete_no_output(translator);
}
