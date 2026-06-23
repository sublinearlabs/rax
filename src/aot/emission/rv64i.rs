use cranelift_codegen::gimli::DW_LANG_Dylan;
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
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            if rd.id() == rs2.id() {
                ctx.write_back(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
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
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::Rs1EqRs2 => {
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
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            if rd.id() != rs2.id() {
                dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs2.id()));
            }

            dynasm!(translator.emitter ; neg Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
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
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            dynasm!(translator.emitter ; sub Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            dynasm!(translator.emitter ; neg Rq(rd.id()));
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; sub Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
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
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            if rd.id() == rs2.id() {
                ctx.write_back(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
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
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            dynasm!(translator.emitter ; or Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            dynasm!(translator.emitter ; or Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::Rs1EqRs2 => {
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; or Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `addi`: 64-bit wrapping add with sign-extended immediate.
/// rd <- (rs1 + sext(imm)) mod 2^64
pub(super) fn emit_addi(
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

    match classify_unary_zero_case(rd, rs1, imm) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero => {
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::Rs1Zero => {
            dynasm!(translator.emitter ; mov Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            dynasm!(translator.emitter ; add Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            dynasm!(translator.emitter ; lea Rq(rd.id()), [Rq(rs1.id()) + imm]);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `andi`: bitwise AND with sign-extended immediate.
/// rd <- rs1 & sext(imm)
pub(super) fn emit_andi(
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

    match classify_unary_zero_case(rd, rs1, imm) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero | UnaryZeroCase::Rs1Zero | UnaryZeroCase::ImmZero => {
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    if imm == -1 {
        if rd.id() == rs1.id() {
            ctx.commit_unchanged(translator);
            return;
        }

        dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
        ctx.write_back(translator);
        return;
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            dynasm!(translator.emitter ; and Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; and Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `slli`: logical left shift by immediate.
/// rd <- rs1 << shamt
pub(super) fn emit_slli(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, shamt as i32) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero | UnaryZeroCase::Rs1Zero => {
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            dynasm!(translator.emitter ; shl Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; shl Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sll`: logical left shift by register low bits.
/// rd <- rs1 << (rs2 & 0x3f)
pub(super) fn emit_sll(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rcx])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero => {
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
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

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = temps
            .allocate()
            .expect("emit_sb requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    if rs2.is_zero() {
        dynasm!(translator.emitter ; mov BYTE [Rq(addr_id) + addr_disp], 0_i8);
    } else {
        dynasm!(translator.emitter ; mov BYTE [Rq(addr_id) + addr_disp], Rb(rs2.id()));
    }
    ctx.complete_no_output(translator);
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

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = temps
            .allocate()
            .expect("emit_sd requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    if rs2.is_zero() {
        dynasm!(translator.emitter ; mov QWORD [Rq(addr_id) + addr_disp], 0_i32);
    } else {
        dynasm!(translator.emitter ; mov QWORD [Rq(addr_id) + addr_disp], Rq(rs2.id()));
    }

    ctx.complete_no_output(translator);
}

/// RV64 `lui`: write U-immediate to upper bits.
/// rd <- sext(imm << 12)
pub(super) fn emit_lui(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    imm: i32,
) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<0, 0>::new()
        .set_output(rd)
        .build(translator, temps);

    let rd = ctx.output();

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
        return;
    }

    let ctx = InstructionContextBuilder::<0, 0>::new()
        .set_output(rd)
        .build(translator, temps);

    let rd = ctx.output();

    let auipc_val = translator.current_pc().wrapping_add(imm as i64 as u64);

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

    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            dynasm!(translator.emitter ; jmp => target_label);
        }
        (true, false) => {
            dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; je => target_label);
        }
        (false, true) => {
            dynasm!(translator.emitter ; test Rq(rs1.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; je => target_label);
        }
        (false, false) => {
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

    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {}
        (true, false) => {
            dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; jne => target_label);
        }
        (false, true) => {
            dynasm!(translator.emitter ; test Rq(rs1.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; jne => target_label);
        }
        (false, false) => {
            dynasm!(translator.emitter ; cmp Rq(rs1.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; jne => target_label);
        }
    }

    ctx.complete_no_output(translator);
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

    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {}
        (true, false) => {
            dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; jne => target_label);
        }
        (false, true) => {}
        (false, false) => {
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

    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            dynasm!(translator.emitter ; jmp => target_label);
        }
        (true, false) => {
            dynasm!(translator.emitter ; test Rq(rs2.id()), Rq(rs2.id()));
            dynasm!(translator.emitter ; je => target_label);
        }
        (false, true) => {
            dynasm!(translator.emitter ; jmp => target_label);
        }
        (false, false) => {
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
        let return_pc = translator.current_pc().wrapping_add(4);
        dynasm!(translator.emitter ; mov Rq(rd.id()), QWORD return_pc as i64);
        ctx.write_back(translator);
    } else {
        ctx.discard_zero_output(translator);
    }

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

    let branch_target = temps.allocate().unwrap();
    let base_riscv_pc = temps.allocate().unwrap();

    if rs1.is_zero() {
        dynasm!(translator.emitter ; mov Rq(branch_target.id()), QWORD imm as i64);
    } else {
        dynasm!(translator.emitter ; lea Rq(branch_target.id()), [Rq(rs1.id()) + imm]);
    }
    dynasm!(translator.emitter ; and Rq(branch_target.id()), -2 as i32);
    dynasm!(translator.emitter ; mov Rq(base_riscv_pc.id()), QWORD translator.cf.base_riscv_pc as i64);
    dynasm!(translator.emitter ; sub Rq(branch_target.id()), Rq(base_riscv_pc.id()));
    dynasm!(translator.emitter ; shr Rq(branch_target.id()), 2);

    if !rd.is_zero() {
        let return_pc = translator.current_pc().wrapping_add(4);
        dynasm!(translator.emitter ; mov Rq(rd.id()), QWORD return_pc as i64);
        ctx.write_back(translator);
    } else {
        ctx.discard_zero_output(translator);
    }

    drop(base_riscv_pc);

    let jump_table_base_reg = temps.allocate().unwrap();

    dynasm!(translator.emitter ; lea Rq(jump_table_base_reg.id()), [=>translator.cf.jt_label]);
    dynasm!(translator.emitter ; jmp QWORD [Rq(jump_table_base_reg.id()) + Rq(branch_target.id()) * 8]);
}

/// RV64 `ecall`: environment call trap.
/// raise environment-call-from-U-mode
pub(super) fn emit_ecall(translator: &mut Translator, temps: &TempAllocator) {
    assert_eq!(
        translator.reg_map.get(&RiscvRegister::A7),
        &MapTarget::Gpr(X86Gpr::Rax)
    );
    assert_eq!(
        translator.reg_map.get(&RiscvRegister::A0),
        &MapTarget::Gpr(X86Gpr::Rdi)
    );
    assert_eq!(
        translator.reg_map.get(&RiscvRegister::A1),
        &MapTarget::Gpr(X86Gpr::Rsi)
    );
    assert_eq!(
        translator.reg_map.get(&RiscvRegister::A2),
        &MapTarget::Gpr(X86Gpr::Rdx)
    );

    let ctx = InstructionContextBuilder::<0, 2>::new()
        .ensure_no_clobber([X86Gpr::Rcx, X86Gpr::R11])
        .set_output(RiscvRegister::A0)
        .build(translator, temps);

    let rdx_temp = temps.allocate().unwrap();
    dynasm!(translator.emitter ; mov Rq(rdx_temp.id()), Rq(X86Gpr::Rdx.id()));

    dynasm!(translator.emitter ; sub Rq(X86Gpr::Rax.id()), 49);
    dynasm!(translator.emitter ; imul Rq(X86Gpr::Rax.id()), Rq(X86Gpr::Rax.id()));
    dynasm!(translator.emitter ; sub Rq(X86Gpr::Rax.id()), 196);
    dynasm!(translator.emitter ; cqo);

    let divisor_reg = temps.allocate().unwrap();
    dynasm!(translator.emitter ; mov Rq(divisor_reg.id()), 29);
    dynasm!(translator.emitter ; idiv Rq(divisor_reg.id()));

    dynasm!(translator.emitter ; mov Rq(X86Gpr::Rdx.id()), Rq(rdx_temp.id()));

    dynasm!(translator.emitter ; syscall);

    dynasm!(translator.emitter ; mov Rq(ctx.output().id()), Rq(X86Gpr::Rax.id()));

    ctx.write_back(translator);
}

/// RV64 `lb`: load 8-bit value (sign-extended).
/// rd <- sext(M[rs1 + imm][7:0])
pub(super) fn emit_lb(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = temps
            .allocate()
            .expect("emit_lb requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    dynasm!(translator.emitter ; movsx Rq(rd.id()), BYTE [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `lbu`: load 8-bit value (zero-extended).
/// rd <- M[rs1 + imm][7:0]
pub(super) fn emit_lbu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = temps
            .allocate()
            .expect("emit_lbu requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    dynasm!(translator.emitter ; movzx Rq(rd.id()), BYTE [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `lh`: load 16-bit value (sign-extended).
/// rd <- sext(M[rs1 + imm][15:0])
pub(super) fn emit_lh(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = temps
            .allocate()
            .expect("emit_lh requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    dynasm!(translator.emitter ; movsx Rq(rd.id()), WORD [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `lhu`: load 16-bit value (zero-extended).
/// rd <- M[rs1 + imm][15:0]
pub(super) fn emit_lhu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = temps
            .allocate()
            .expect("emit_lhu requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    dynasm!(translator.emitter ; movzx Rq(rd.id()), WORD [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `lw`: load 32-bit value (sign-extended).
/// rd <- sext(M[rs1 + imm][31:0])
pub(super) fn emit_lw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = temps
            .allocate()
            .expect("emit_lw requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    dynasm!(translator.emitter ; movsxd Rq(rd.id()), DWORD [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `lwu`: load 32-bit value (zero-extended).
/// rd <- M[rs1 + imm][31:0]
pub(super) fn emit_lwu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = temps
            .allocate()
            .expect("emit_lwu requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    dynasm!(translator.emitter ; mov Rd(rd.id()), DWORD [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `ld`: load 64-bit value.
/// rd <- M[rs1 + imm][63:0]
pub(super) fn emit_ld(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = temps
            .allocate()
            .expect("emit_ld requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    dynasm!(translator.emitter ; mov Rq(rd.id()), QWORD [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `sh`: store low 16 bits of rs2 to memory at rs1 + sext(imm).
/// mem16[rs1 + sext(imm)] <- rs2[15:0]
pub(super) fn emit_sh(
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

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = temps
            .allocate()
            .expect("emit_sh requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    if rs2.is_zero() {
        dynasm!(translator.emitter ; mov WORD [Rq(addr_id) + addr_disp], 0);
    } else {
        dynasm!(translator.emitter ; mov WORD [Rq(addr_id) + addr_disp], Rw(rs2.id()));
    }

    ctx.complete_no_output(translator);
}

/// RV64 `sw`: store low 32 bits of rs2 to memory at rs1 + sext(imm).
/// mem32[rs1 + sext(imm)] <- rs2[31:0]
pub(super) fn emit_sw(
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

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = temps
            .allocate()
            .expect("emit_sw requires a temp to materialize x0 + imm address");
        dynasm!(translator.emitter ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    if rs2.is_zero() {
        dynasm!(translator.emitter ; mov DWORD [Rq(addr_id) + addr_disp], 0);
    } else {
        dynasm!(translator.emitter ; mov DWORD [Rq(addr_id) + addr_disp], Rd(rs2.id()));
    }

    ctx.complete_no_output(translator);
}

/// RV64 `and`: bitwise AND across all 64 bits.
/// rd <- rs1 & rs2
pub(super) fn emit_and(
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

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero | ZeroCase::Rs2Zero => {
            // case 1
            // and rd, 0, 0 -> rd = 0
            //
            // case 2
            // and rd, 0, rs2 -> rd = 0
            //
            // case 3
            // and rd, rs1, 0 -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(rd, rs1, rs2) {
        ShadowCase::AllEqual => {
            // and rd, rd, rd
            // nothing changes
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // and rd, rd, rs2
            dynasm!(translator.emitter ; and Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // and rd, rs1, rd
            dynasm!(translator.emitter ; and Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::Rs1EqRs2 => {
            // and rd, rs1, rs1
            // rd = rs1
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // and rd, rs1, rs2
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; and Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `xor`: bitwise XOR across all 64 bits.
/// rd <- rs1 ^ rs2
pub(super) fn emit_xor(
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

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // xor rd, 0, 0 -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // xor rd, 0, rs2 -> rd = rs2
            if rd.id() == rs2.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // xor rd, rs1, 0 -> rd = rs1
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

    match classify_shadow_case(rd, rs1, rs2) {
        ShadowCase::AllEqual | ShadowCase::Rs1EqRs2 => {
            // case 1
            // xor rd, rd, rd -> rd = 0
            //
            // case 2
            // xor rd, rs1, rs1 -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // xor rd, rd, rs2
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // xor rd, rs1, rd
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // xor rd, rs1, rs2
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `ori`: bitwise OR with sign-extended immediate.
/// rd <- rs1 | sext(imm)
pub(super) fn emit_ori(
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

    match classify_unary_zero_case(rd, rs1, imm) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero => {
            // ori, rd, 0, 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::Rs1Zero => {
            // ori rd, 0, imm
            dynasm!(translator.emitter ; mov Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // ori rd, rs1, 0
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // ori rd, rd, imm
            dynasm!(translator.emitter ; or Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // ori rd, rs1, imm
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; or Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `xori`: bitwise XOR with sign-extended immediate.
/// rd <- rs1 ^ sext(imm)
pub(super) fn emit_xori(
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

    match classify_unary_zero_case(rd, rs1, imm) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero => {
            // xori rd, 0, 0 -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::Rs1Zero => {
            // xori rd, 0, imm -> rd = imm
            dynasm!(translator.emitter ; mov Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // xori rd, rs1, 0 -> rd = rs1
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // xori rd, rd, imm
            dynasm!(translator.emitter ; xor Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // xori rd, rs1, imm
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; xor Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `srl`: logical right shift by register low bits.
/// rd <- rs1 >> (rs2 & 0x3f)
pub(super) fn emit_srl(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rcx])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero => {
            // case 1
            // srl rd, 0, 0 -> rd == 0
            //
            // case 2
            // srl rd, 0, shamt -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // srl rd, rs1, 0
            // -> rd = rs1
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

    dynasm!(translator.emitter ; mov Rq(X86Gpr::Rcx.id()), Rq(rs2.id()));

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // srl rd, rd, shamt
            dynasm!(translator.emitter ; shr Rq(rd.id()), cl);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // srl rd, rs1, shamt
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; shr Rq(rd.id()), cl);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `srli`: logical right shift by immediate.
/// rd <- rs1 >> shamt
pub(super) fn emit_srli(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, shamt as i32) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero | UnaryZeroCase::Rs1Zero => {
            // case 1
            // srli rd, 0, 0 -> rd = 0
            //
            // case 2
            // srli rd, 0, shamt -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // srli rd, rs1, 0 -> rd = rs1
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // srli rd, rd, shamt
            dynasm!(translator.emitter ; shr Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // srli rd, rs1, shamt
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; shr Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `srai`: arithmetic right shift by immediate.
/// rd <- rs1 >>> shamt
pub(super) fn emit_srai(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    shamt: u8,
) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator, temps);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, shamt as i32) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero | UnaryZeroCase::Rs1Zero => {
            // case 1
            // srai rd, 0, 0 -> rd = 0
            //
            // case 2
            // srai rd, 0, shamt -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // srai rd, rs1, 0 -> rd = rs1
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // srai rd, rd, shamt
            dynasm!(translator.emitter ; sar Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // srai rd, rs1, shamt
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; sar Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `slt`: set if less than (signed).
/// rd <- 1 if signed(rs1) < signed(rs2) else 0
pub(super) fn emit_slt(
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

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // slt rd, 0, 0
            // since equal rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero | ZeroCase::Rs2Zero | ZeroCase::None => {
            // case 1
            // slt rd, 0, rs2
            //
            // case 2
            // slt rd, rs1, 0
            //
            // we just fall through to the generic handler
        }
    }

    if rs1.id() == rs2.id() {
        // since they are equal they can't be less than
        // hence we set rd = 0
        dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
        return;
    } else {
        dynasm!(translator.emitter ; cmp Rq(rs1.id()), Rq(rs2.id()));
        dynasm!(translator.emitter ; setl Rb(rd.id()));
        dynasm!(translator.emitter ; movzx Rq(rd.id()), Rb(rd.id()));
        ctx.write_back(translator);
        return;
    }
}

/// RV64 `sltu`: set if less than (unsigned).
/// rd <- 1 if unsigned(rs1) < unsigned(rs2) else 0
pub(super) fn emit_sltu(
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

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // sltu rd, 0, 0
            // both equal so rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero | ZeroCase::Rs2Zero | ZeroCase::None => {
            // case 1
            // sltu rd, 0, rs2
            //
            // case 2
            // sltu rd, rs1, 0
            //
            // because the unknown in both cases could be a
            // zero or some value greater than a zero
            // and there is no way to distinguish at compile time
            // it is better to just fall through to the generic handler
        }
    }

    if rs1.id() == rs2.id() {
        // since they are equal
        // rd = 0
        dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
        return;
    } else {
        dynasm!(translator.emitter ; cmp Rq(rs1.id()), Rq(rs2.id()));
        dynasm!(translator.emitter ; setb Rb(rd.id()));
        dynasm!(translator.emitter ; movzx Rq(rd.id()), Rb(rd.id()));
        ctx.write_back(translator);
        return;
    }
}

/// RV64 `slti`: set if less than immediate (signed).
/// rd <- 1 if signed(rs1) < sext(imm) else 0
pub(super) fn emit_slti(
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

    match classify_unary_zero_case(rd, rs1, imm) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero => {
            // slti rd, 0, 0
            // -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::Rs1Zero => {
            // slti rd, 0, imm
            if imm > 0 {
                // rd = 1
                dynasm!(translator.emitter ; mov Rq(rd.id()), 1);
                ctx.write_back(translator);
                return;
            } else {
                // rd = 0
                dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
                ctx.write_back(translator);
                return;
            }
        }

        UnaryZeroCase::ImmZero | UnaryZeroCase::None => {
            // slti rd, rs1, 0
            //
            // rs1 can take on different value classes
            // that we cannot distinguish at compile time
            // hence we fallthrough to the generic handler
        }
    }

    // not possible to compare between rs1 and immediate
    // at compile time, so we use the generic handler
    // directly.

    dynasm!(translator.emitter ; cmp Rq(rs1.id()), imm);
    dynasm!(translator.emitter ; setl Rb(rd.id()));
    dynasm!(translator.emitter ; movzx Rq(rd.id()), Rb(rd.id()));
    ctx.write_back(translator);
    return;
}

/// RV64 `sltiu`: set if less than immediate (unsigned).
/// rd <- 1 if unsigned(rs1) < sext(imm) else 0
#[allow(unused_variables)]
pub(super) fn emit_sltiu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
}

/// RV64 `blt`: branch if less than (signed).
/// if signed(rs1) < signed(rs2) then pc <- pc + sext(imm)
#[allow(unused_variables)]
pub(super) fn emit_blt(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
}

/// RV64 `bge`: branch if greater or equal (signed).
/// if signed(rs1) >= signed(rs2) then pc <- pc + sext(imm)
#[allow(unused_variables)]
pub(super) fn emit_bge(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
}
