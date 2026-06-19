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

/// RV64 `lui`: write U-immediate to upper bits.
/// rd <- sext(imm << 12)
pub(super) fn emit_lui(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    imm: i32,
) {
    if rd.is_zero() {
        // x0 is hardwired to 0, writes can be ignored
        return;
    }

    let ctx = InstructionContextBuilder::<0, 0>::new()
        .set_output(rd)
        .build(translator, temps);

    let rd = ctx.output();

    // NOTE: the immediate is already shifted by 12 from the decode layer
    dynasm!(translator.emitter ; mov Rq(rd.id()), imm);

    ctx.write_back(translator);
}

/// RV64 `auipc`: add U-immediate (<<12) to current PC.
/// rd <- pc + sext(imm << 12)
pub(super) fn emit_auipc(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    imm: i32,
) {
    if rd.is_zero() {
        // x0 is hardwired to 0, writes can be ignored
        return;
    }

    let ctx = InstructionContextBuilder::<0, 0>::new()
        .set_output(rd)
        .build(translator, temps);

    let rd = ctx.output();

    let auipc_val = translator.current_pc().wrapping_add(imm as i64 as u64);

    // NOTE: the immediate is already shifted by 12 from the decode layer
    dynasm!(translator.emitter ; mov Rq(rd.id()), QWORD auipc_val as i64);

    ctx.write_back(translator);
}

/// RV64 `beq`: branch if equal.
/// if rs1 == rs2 then pc <- pc + sext(imm)
pub(super) fn emit_beq(
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

    // compute the target riscv pc
    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    // retrieve or create a new dynamic label for the riscv pc
    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            // both are zero, hence both equal
            dynasm!(translator.emitter ; jmp => target_label);
        }
        (true, false) => {
            // rs1 equals zero
            // we need to check if rs2 equals zero
            dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; je => target_label);
        }
        (false, true) => {
            // rs2 equals zero
            // we need to check if rs1 equals zero
            dynasm!(translator.emitter ; test Rq(rs1.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; je => target_label);
        }
        (false, false) => {
            // both non zero
            dynasm!(translator.emitter ; cmp Rq(rs1.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; je => target_label);
        }
    }

    ctx.complete_no_output(translator);
}

/// RV64 `bne`: branch if not equal.
/// if rs1 != rs2 then pc <- pc + sext(imm)
pub(super) fn emit_bne(
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

    // compute the target riscv pc
    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    // retrieve or create a new dynamic label for the riscv pc
    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            // both are equal, we shouldn't jump
        }
        (true, false) => {
            // rs1 is zero
            // check if rs2 is zero, don't jump if it is
            dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; jne => target_label);
        }
        (false, true) => {
            // rs2 is zero
            // check if rs1 is zero, don't jump if it is
            dynasm!(translator.emitter ; test Rq(rs1.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; jne => target_label);
        }
        (false, false) => {
            // both are not zero
            dynasm!(translator.emitter ; cmp Rq(rs1.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; jne => target_label);
        }
    }

    ctx.complete_no_output(translator);
}

/// RV64 `blt`: branch if signed rs1 < rs2.
/// if signed(rs1) < signed(rs2) then pc <- pc + sext(imm)
#[allow(unused_variables)]
pub(super) fn emit_blt(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 blt emission.
}

/// RV64 `bge`: branch if signed rs1 >= rs2.
/// if signed(rs1) >= signed(rs2) then pc <- pc + sext(imm)
#[allow(unused_variables)]
pub(super) fn emit_bge(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    // TODO: implement RV64 bge emission.
}

/// RV64 `bltu`: branch if unsigned rs1 < rs2.
/// if unsigned(rs1) < unsigned(rs2) then pc <- pc + sext(imm)
pub(super) fn emit_bltu(
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

    // compute the target riscv pc
    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    // retrieve or create a new dynamic label for the riscv pc
    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            // both are equal, we shouldn't jump
        }
        (true, false) => {
            // rs1 is zero
            // if rs2 is anything but zero, it is fine to jump
            dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; jne => target_label);
        }
        (false, true) => {
            // rs2 is zero
            // rs1 cannot have a value that will be less than zero
            // hence we don't jump
        }
        (false, false) => {
            // both are not zero
            dynasm!(translator.emitter ; cmp Rq(rs1.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; jb => target_label);
        }
    }
    ctx.complete_no_output(translator);
}

/// RV64 `bgeu`: branch if unsigned rs1 >= rs2.
/// if unsigned(rs1) >= unsigned(rs2) then pc <- pc + sext(imm)
pub(super) fn emit_bgeu(
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

    // compute the target riscv pc
    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    // retrieve or create a new dynamic label for the riscv pc
    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            // rs1 equals rs2
            dynasm!(translator.emitter ; jmp => target_label);
        }
        (true, false) => {
            // rs1 is zero
            // only condition for jump will be if rs2 is also zero
            dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; je => target_label);
        }
        (false, true) => {
            // rs2 is zero
            // for all values of rs1, rs1 >= rs2
            // hence we always jump
            dynasm!(translator.emitter ; jmp => target_label);
        }
        (false, false) => {
            // both are not zero
            dynasm!(translator.emitter ; cmp Rq(rs1.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; jae => target_label);
        }
    }

    ctx.complete_no_output(translator);
}

/// RV64 `jal`: jump and link.
/// rd <- pc + 4; pc <- pc + sext(imm)
pub(super) fn emit_jal(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<0, 0>::new()
        .set_output(rd)
        .build(translator, temps);

    let rd = ctx.output();

    if !rd.is_zero() {
        // set the return pc
        let return_pc = translator.current_pc().wrapping_add(4);
        dynasm!(translator.emitter ; mov Rq(rd.id()), QWORD return_pc as i64);
        ctx.write_back(translator);
    } else {
        ctx.discard_zero_output(translator);
    }

    // update pc
    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);
    let target_label = translator.target_label(branch_target);
    dynasm!(translator.emitter ; jmp => target_label);
}

/// RV64 `jalr`: indirect jump and link.
/// t <- pc + 4; pc <- (rs1 + sext(imm)) & !1; rd <- t
pub(super) fn emit_jalr(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    // prepare temps
    let branch_target = temps.allocate().unwrap();
    let base_riscv_pc = temps.allocate().unwrap();

    // branch_target = (rs1 + imm) & !1
    // jump_table_id = (branch_target - base_riscv_pc) >> 2 (assumes uncompressed)
    if rs1.is_zero() {
        dynasm!(translator.emitter ; mov Rq(branch_target.id()), QWORD imm as i64);
    } else {
        dynasm!(translator.emitter ; lea Rq(branch_target.id()), [Rq(rs1.id()) + imm]);
    }
    dynasm!(translator.emitter ; and Rq(branch_target.id()), -2 as i32);
    dynasm!(translator.emitter ; mov Rq(base_riscv_pc.id()), QWORD translator.cf.base_riscv_pc as i64);
    dynasm!(translator.emitter ; sub Rq(branch_target.id()), Rq(base_riscv_pc.id()));
    dynasm!(translator.emitter ; shr Rq(branch_target.id()), 2);

    // write return pc
    if !rd.is_zero() {
        let return_pc = translator.current_pc().wrapping_add(4);
        dynasm!(translator.emitter ; mov Rq(rd.id()), QWORD return_pc as i64);
        ctx.write_back(translator);
    } else {
        ctx.discard_zero_output(translator);
    }

    // free the base riscv pc temp
    drop(base_riscv_pc);

    let jump_table_base_reg = temps.allocate().unwrap();

    // extract the value at the jump table index
    dynasm!(translator.emitter ; lea Rq(jump_table_base_reg.id()), [=>translator.cf.jt_label]);
    dynasm!(translator.emitter ; jmp QWORD [Rq(jump_table_base_reg.id()) + Rq(branch_target.id()) * 8]);
}

/// RV64 `ecall`: environment call trap.
#[allow(unused_variables)]
pub(super) fn emit_ecall(translator: &mut Translator, temps: &TempAllocator) {
    // TODO: implement RV64 ecall trap emission.
}
