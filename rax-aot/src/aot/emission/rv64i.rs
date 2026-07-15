use crate::aot::emit_asm;

use crate::aot::{
    classification::{
        classify_shadow_case, classify_unary_shadow_case, classify_unary_zero_case,
        classify_zero_case, ShadowCase, UnaryShadowCase, UnaryZeroCase, ZeroCase,
    },
    instruction_context::InstructionContextBuilder,
    register_mapping::MapTarget,
    registers::{RiscvRegister, X86Gpr},
    translator::Translator,
};

/// RV64 `add`: 64-bit wrapping addition.
/// rd <- (rs1 + rs2) mod 2^64
pub(super) fn emit_add(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator);
    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            if rd.id() == rs2.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(&rd, &rs1, &rs2) {
        ShadowCase::AllEqual => {
            emit_asm!(translator ; add Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            emit_asm!(translator ; add Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            emit_asm!(translator ; add Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::Rs1EqRs2 => {
            // Avoid LEA here: x86-64 cannot encode RSP as an index register, so a
            // source operand carried in RSP would be misencoded or omitted.
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; add Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // Avoid LEA here: x86-64 cannot encode RSP as an index register, so a
            // source operand carried in RSP would be misencoded or omitted.
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; add Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sub`: 64-bit wrapping subtraction.
/// rd <- (rs1 - rs2) mod 2^64
pub(super) fn emit_sub(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator);
    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            if rd.id() != rs2.id() {
                emit_asm!(translator ; mov Rq(rd.id()), Rq(rs2.id()));
            }

            emit_asm!(translator ; neg Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(&rd, &rs1, &rs2) {
        ShadowCase::AllEqual | ShadowCase::Rs1EqRs2 => {
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            emit_asm!(translator ; sub Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            emit_asm!(translator ; neg Rq(rd.id()));
            emit_asm!(translator ; add Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; sub Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `or`: bitwise OR across all 64 bits.
/// rd <- rs1 | rs2
pub(super) fn emit_or(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator);
    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            if rd.id() == rs2.id() {
                ctx.write_back(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            if rd.id() == rs1.id() {
                ctx.write_back(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
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
            emit_asm!(translator ; or Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            emit_asm!(translator ; or Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::Rs1EqRs2 => {
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; or Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `addi`: 64-bit wrapping add with sign-extended immediate.
/// rd <- (rs1 + sext(imm)) mod 2^64
pub(super) fn emit_addi(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);
    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, imm) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero => {
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::Rs1Zero => {
            emit_asm!(translator ; mov Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            emit_asm!(translator ; add Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            emit_asm!(translator ; lea Rq(rd.id()), [Rq(rs1.id()) + imm]);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `andi`: bitwise AND with sign-extended immediate.
/// rd <- rs1 & sext(imm)
pub(super) fn emit_andi(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, imm) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero | UnaryZeroCase::Rs1Zero | UnaryZeroCase::ImmZero => {
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
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

        emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
        ctx.write_back(translator);
        return;
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            emit_asm!(translator ; and Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; and Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `slli`: logical left shift by immediate.
/// rd <- rs1 << shamt
pub(super) fn emit_slli(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, shamt: u8) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, shamt as i32) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero | UnaryZeroCase::Rs1Zero => {
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            emit_asm!(translator ; shl Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; shl Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sll`: logical left shift by register low bits.
/// rd <- rs1 << (rs2 & 0x3f)
pub(super) fn emit_sll(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rcx])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero => {
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    emit_asm!(translator ; mov Rq(X86Gpr::Rcx.id()), Rq(rs2.id()));

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            emit_asm!(translator ; shl Rq(rd.id()), cl);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; shl Rq(rd.id()), cl);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sb`: store low 8 bits of rs2 to memory at rs1 + sext(imm).
/// mem8[rs1 + sext(imm)] <- rs2[7:0]
pub(super) fn emit_sb(translator: &Translator, rs1: RiscvRegister, rs2: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = translator
            .temp_pool
            .allocate()
            .expect("emit_sb requires a temp to materialize x0 + imm address");
        emit_asm!(translator ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    if rs2.is_zero() {
        emit_asm!(translator ; mov BYTE [Rq(addr_id) + addr_disp], 0_i8);
    } else {
        emit_asm!(translator ; mov BYTE [Rq(addr_id) + addr_disp], Rb(rs2.id()));
    }
    ctx.complete_no_output(translator);
}

/// RV64 `sd`: store 64 bits of rs2 to memory at rs1 + sext(imm).
/// mem64[rs1 + sext(imm)] <- rs2
pub(super) fn emit_sd(translator: &Translator, rs1: RiscvRegister, rs2: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = translator
            .temp_pool
            .allocate()
            .expect("emit_sd requires a temp to materialize x0 + imm address");
        emit_asm!(translator ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    if rs2.is_zero() {
        emit_asm!(translator ; mov QWORD [Rq(addr_id) + addr_disp], 0_i32);
    } else {
        emit_asm!(translator ; mov QWORD [Rq(addr_id) + addr_disp], Rq(rs2.id()));
    }

    ctx.complete_no_output(translator);
}

/// RV64 `lui`: write U-immediate to upper bits.
/// rd <- sext(imm << 12)
pub(super) fn emit_lui(translator: &Translator, rd: RiscvRegister, imm: i32) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<0, 0>::new()
        .set_output(rd)
        .build(translator);

    let rd = ctx.output();

    emit_asm!(translator ; mov Rq(rd.id()), imm);

    ctx.write_back(translator);
}

/// RV64 `auipc`: add U-immediate (<<12) to current PC.
/// rd <- pc + sext(imm << 12)
pub(super) fn emit_auipc(translator: &Translator, rd: RiscvRegister, imm: i32) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<0, 0>::new()
        .set_output(rd)
        .build(translator);

    let rd = ctx.output();

    let auipc_val = translator.current_pc().wrapping_add(imm as i64 as u64);

    emit_asm!(translator ; mov Rq(rd.id()), QWORD auipc_val as i64);

    ctx.write_back(translator);
}

/// RV64 `beq`: branch if equal.
/// if rs1 == rs2 then pc <- pc + sext(imm)
pub(super) fn emit_beq(translator: &Translator, rs1: RiscvRegister, rs2: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();

    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            emit_asm!(translator ; jmp => target_label);
        }
        (true, false) => {
            emit_asm!(translator ; test Rq(rs2.id()), Rq(rs2.id()));
            emit_asm!(translator ; je => target_label);
        }
        (false, true) => {
            emit_asm!(translator ; test Rq(rs1.id()), Rq(rs1.id()));
            emit_asm!(translator ; je => target_label);
        }
        (false, false) => {
            emit_asm!(translator ; cmp Rq(rs1.id()), Rq(rs2.id()));
            emit_asm!(translator ; je => target_label);
        }
    }

    ctx.complete_no_output(translator);
}

/// RV64 `bne`: branch if not equal.
/// if rs1 != rs2 then pc <- pc + sext(imm)
pub(super) fn emit_bne(translator: &Translator, rs1: RiscvRegister, rs2: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();

    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {}
        (true, false) => {
            emit_asm!(translator ; test Rq(rs2.id()), Rq(rs2.id()));
            emit_asm!(translator ; jne => target_label);
        }
        (false, true) => {
            emit_asm!(translator ; test Rq(rs1.id()), Rq(rs1.id()));
            emit_asm!(translator ; jne => target_label);
        }
        (false, false) => {
            emit_asm!(translator ; cmp Rq(rs1.id()), Rq(rs2.id()));
            emit_asm!(translator ; jne => target_label);
        }
    }

    ctx.complete_no_output(translator);
}

/// RV64 `bltu`: branch if unsigned rs1 < rs2.
/// if unsigned(rs1) < unsigned(rs2) then pc <- pc + sext(imm)
pub(super) fn emit_bltu(translator: &Translator, rs1: RiscvRegister, rs2: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();

    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {}
        (true, false) => {
            emit_asm!(translator ; test Rq(rs2.id()), Rq(rs2.id()));
            emit_asm!(translator ; jne => target_label);
        }
        (false, true) => {}
        (false, false) => {
            emit_asm!(translator ; cmp Rq(rs1.id()), Rq(rs2.id()));
            emit_asm!(translator ; jb => target_label);
        }
    }
    ctx.complete_no_output(translator);
}

/// RV64 `bgeu`: branch if unsigned rs1 >= rs2.
/// if unsigned(rs1) >= unsigned(rs2) then pc <- pc + sext(imm)
pub(super) fn emit_bgeu(translator: &Translator, rs1: RiscvRegister, rs2: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();

    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            emit_asm!(translator ; jmp => target_label);
        }
        (true, false) => {
            emit_asm!(translator ; test Rq(rs2.id()), Rq(rs2.id()));
            emit_asm!(translator ; je => target_label);
        }
        (false, true) => {
            emit_asm!(translator ; jmp => target_label);
        }
        (false, false) => {
            emit_asm!(translator ; cmp Rq(rs1.id()), Rq(rs2.id()));
            emit_asm!(translator ; jae => target_label);
        }
    }

    ctx.complete_no_output(translator);
}

/// RV64 `jal`: jump and link.
/// rd <- pc + 4; pc <- pc + sext(imm)
pub(super) fn emit_jal(translator: &Translator, rd: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<0, 0>::new()
        .set_output(rd)
        .build(translator);

    let rd = ctx.output();

    if !rd.is_zero() {
        let return_pc = translator.current_pc().wrapping_add(4);
        emit_asm!(translator ; mov Rq(rd.id()), QWORD return_pc as i64);
        ctx.write_back(translator);
    } else {
        ctx.discard_zero_output(translator);
    }

    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);
    let target_label = translator.target_label(branch_target);
    emit_asm!(translator ; jmp => target_label);
}

/// RV64 `jalr`: indirect jump and link.
/// t <- pc + 4; pc <- (rs1 + sext(imm)) & !1; rd <- t
pub(super) fn emit_jalr(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let branch_target = translator.temp_pool.allocate().unwrap();
    let base_riscv_pc = translator.temp_pool.allocate().unwrap();

    if rs1.is_zero() {
        emit_asm!(translator ; mov Rq(branch_target.id()), QWORD imm as i64);
    } else {
        emit_asm!(translator ; lea Rq(branch_target.id()), [Rq(rs1.id()) + imm]);
    }
    emit_asm!(translator ; and Rq(branch_target.id()), -2 as i32);
    emit_asm!(translator ; mov Rq(base_riscv_pc.id()), QWORD translator.cf.base_riscv_pc as i64);
    emit_asm!(translator ; sub Rq(branch_target.id()), Rq(base_riscv_pc.id()));
    emit_asm!(translator ; shr Rq(branch_target.id()), 2);

    if !rd.is_zero() {
        let return_pc = translator.current_pc().wrapping_add(4);
        emit_asm!(translator ; mov Rq(rd.id()), QWORD return_pc as i64);
        ctx.write_back(translator);
    } else {
        ctx.discard_zero_output(translator);
    }

    drop(base_riscv_pc);

    let jump_table_base_reg = translator.temp_pool.allocate().unwrap();

    emit_asm!(translator ; lea Rq(jump_table_base_reg.id()), [=>translator.cf.jt_label]);
    emit_asm!(translator ; jmp QWORD [Rq(jump_table_base_reg.id()) + Rq(branch_target.id()) * 8]);
}

/// RV64 `ecall`: environment call trap.
/// raise environment-call-from-U-mode
pub(super) fn emit_ecall(translator: &Translator) {
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

    let ctx = InstructionContextBuilder::<0, 3>::new()
        .ensure_no_clobber([X86Gpr::Rcx, X86Gpr::R11, X86Gpr::Rax])
        .set_output(RiscvRegister::A0)
        .build(translator);

    let rdx_temp = translator.temp_pool.allocate().unwrap();
    emit_asm!(translator ; mov Rq(rdx_temp.id()), Rq(X86Gpr::Rdx.id()));

    emit_asm!(translator ; sub Rq(X86Gpr::Rax.id()), 49);
    emit_asm!(translator ; imul Rq(X86Gpr::Rax.id()), Rq(X86Gpr::Rax.id()));
    emit_asm!(translator ; sub Rq(X86Gpr::Rax.id()), 196);
    emit_asm!(translator ; cqo);

    let divisor_reg = translator.temp_pool.allocate().unwrap();
    emit_asm!(translator ; mov Rq(divisor_reg.id()), 29);
    emit_asm!(translator ; idiv Rq(divisor_reg.id()));

    emit_asm!(translator ; mov Rq(X86Gpr::Rdx.id()), Rq(rdx_temp.id()));

    emit_asm!(translator ; syscall);

    emit_asm!(translator ; mov Rq(ctx.output().id()), Rq(X86Gpr::Rax.id()));

    ctx.write_back(translator);
}

/// RV64 `lb`: load 8-bit value (sign-extended).
/// rd <- sext(M[rs1 + imm][7:0])
pub(super) fn emit_lb(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = translator
            .temp_pool
            .allocate()
            .expect("emit_lb requires a temp to materialize x0 + imm address");
        emit_asm!(translator ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    emit_asm!(translator ; movsx Rq(rd.id()), BYTE [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `lbu`: load 8-bit value (zero-extended).
/// rd <- M[rs1 + imm][7:0]
pub(super) fn emit_lbu(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = translator
            .temp_pool
            .allocate()
            .expect("emit_lbu requires a temp to materialize x0 + imm address");
        emit_asm!(translator ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    emit_asm!(translator ; movzx Rq(rd.id()), BYTE [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `lh`: load 16-bit value (sign-extended).
/// rd <- sext(M[rs1 + imm][15:0])
pub(super) fn emit_lh(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = translator
            .temp_pool
            .allocate()
            .expect("emit_lh requires a temp to materialize x0 + imm address");
        emit_asm!(translator ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    emit_asm!(translator ; movsx Rq(rd.id()), WORD [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `lhu`: load 16-bit value (zero-extended).
/// rd <- M[rs1 + imm][15:0]
pub(super) fn emit_lhu(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = translator
            .temp_pool
            .allocate()
            .expect("emit_lhu requires a temp to materialize x0 + imm address");
        emit_asm!(translator ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    emit_asm!(translator ; movzx Rq(rd.id()), WORD [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `lw`: load 32-bit value (sign-extended).
/// rd <- sext(M[rs1 + imm][31:0])
pub(super) fn emit_lw(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = translator
            .temp_pool
            .allocate()
            .expect("emit_lw requires a temp to materialize x0 + imm address");
        emit_asm!(translator ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    emit_asm!(translator ; movsxd Rq(rd.id()), DWORD [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `lwu`: load 32-bit value (zero-extended).
/// rd <- M[rs1 + imm][31:0]
pub(super) fn emit_lwu(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = translator
            .temp_pool
            .allocate()
            .expect("emit_lwu requires a temp to materialize x0 + imm address");
        emit_asm!(translator ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    emit_asm!(translator ; mov Rd(rd.id()), DWORD [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `ld`: load 64-bit value.
/// rd <- M[rs1 + imm][63:0]
pub(super) fn emit_ld(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    if rd.is_zero() {
        return;
    }

    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = translator
            .temp_pool
            .allocate()
            .expect("emit_ld requires a temp to materialize x0 + imm address");
        emit_asm!(translator ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    emit_asm!(translator ; mov Rq(rd.id()), QWORD [Rq(addr_id) + addr_disp]);

    ctx.write_back(translator);
}

/// RV64 `sh`: store low 16 bits of rs2 to memory at rs1 + sext(imm).
/// mem16[rs1 + sext(imm)] <- rs2[15:0]
pub(super) fn emit_sh(translator: &Translator, rs1: RiscvRegister, rs2: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = translator
            .temp_pool
            .allocate()
            .expect("emit_sh requires a temp to materialize x0 + imm address");
        emit_asm!(translator ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    if rs2.is_zero() {
        emit_asm!(translator ; mov WORD [Rq(addr_id) + addr_disp], 0);
    } else {
        emit_asm!(translator ; mov WORD [Rq(addr_id) + addr_disp], Rw(rs2.id()));
    }

    ctx.complete_no_output(translator);
}

/// RV64 `sw`: store low 32 bits of rs2 to memory at rs1 + sext(imm).
/// mem32[rs1 + sext(imm)] <- rs2[31:0]
pub(super) fn emit_sw(translator: &Translator, rs1: RiscvRegister, rs2: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();

    let addr_temp;
    let (addr_id, addr_disp) = if rs1.is_zero() {
        addr_temp = translator
            .temp_pool
            .allocate()
            .expect("emit_sw requires a temp to materialize x0 + imm address");
        emit_asm!(translator ; mov Rq(addr_temp.id()), QWORD imm as i64);
        (addr_temp.id(), 0)
    } else {
        (rs1.id(), imm)
    };

    if rs2.is_zero() {
        emit_asm!(translator ; mov DWORD [Rq(addr_id) + addr_disp], 0);
    } else {
        emit_asm!(translator ; mov DWORD [Rq(addr_id) + addr_disp], Rd(rs2.id()));
    }

    ctx.complete_no_output(translator);
}

/// RV64 `and`: bitwise AND across all 64 bits.
/// rd <- rs1 & rs2
pub(super) fn emit_and(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator);

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
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
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
            emit_asm!(translator ; and Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // and rd, rs1, rd
            emit_asm!(translator ; and Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::Rs1EqRs2 => {
            // and rd, rs1, rs1
            // rd = rs1
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // and rd, rs1, rs2
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; and Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `xor`: bitwise XOR across all 64 bits.
/// rd <- rs1 ^ rs2
pub(super) fn emit_xor(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // xor rd, 0, 0 -> rd = 0
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // xor rd, 0, rs2 -> rd = rs2
            if rd.id() == rs2.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // xor rd, rs1, 0 -> rd = rs1
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
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
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // xor rd, rd, rs2
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // xor rd, rs1, rd
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // xor rd, rs1, rs2
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `ori`: bitwise OR with sign-extended immediate.
/// rd <- rs1 | sext(imm)
pub(super) fn emit_ori(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, imm) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero => {
            // ori, rd, 0, 0
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::Rs1Zero => {
            // ori rd, 0, imm
            emit_asm!(translator ; mov Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // ori rd, rs1, 0
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // ori rd, rd, imm
            emit_asm!(translator ; or Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // ori rd, rs1, imm
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; or Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `xori`: bitwise XOR with sign-extended immediate.
/// rd <- rs1 ^ sext(imm)
pub(super) fn emit_xori(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, imm) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero => {
            // xori rd, 0, 0 -> rd = 0
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::Rs1Zero => {
            // xori rd, 0, imm -> rd = imm
            emit_asm!(translator ; mov Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // xori rd, rs1, 0 -> rd = rs1
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // xori rd, rd, imm
            emit_asm!(translator ; xor Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // xori rd, rs1, imm
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; xor Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `srl`: logical right shift by register low bits.
/// rd <- rs1 >> (rs2 & 0x3f)
pub(super) fn emit_srl(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rcx])
        .build(translator);

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
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
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

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    emit_asm!(translator ; mov Rq(X86Gpr::Rcx.id()), Rq(rs2.id()));

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // srl rd, rd, shamt
            emit_asm!(translator ; shr Rq(rd.id()), cl);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // srl rd, rs1, shamt
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; shr Rq(rd.id()), cl);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sra`: arithmetic right shift by register low bits.
/// rd <- signed(rs1) >> (rs2 & 0x3f)
pub(super) fn emit_sra(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .ensure_no_clobber([X86Gpr::Rcx])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(rd, rs1, rs2) {
        ZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero | ZeroCase::Rs1Zero => {
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    emit_asm!(translator ; mov Rq(X86Gpr::Rcx.id()), Rq(rs2.id()));

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            emit_asm!(translator ; sar Rq(rd.id()), cl);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; sar Rq(rd.id()), cl);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `srli`: logical right shift by immediate.
/// rd <- rs1 >> shamt
pub(super) fn emit_srli(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, shamt: u8) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

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
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // srli rd, rs1, 0 -> rd = rs1
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // srli rd, rd, shamt
            emit_asm!(translator ; shr Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // srli rd, rs1, shamt
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; shr Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `srai`: arithmetic right shift by immediate.
/// rd <- rs1 >>> shamt
pub(super) fn emit_srai(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, shamt: u8) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

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
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // srai rd, rs1, 0 -> rd = rs1
            if rd.id() == rs1.id() {
                ctx.commit_unchanged(translator);
                return;
            }

            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    match classify_unary_shadow_case(rd, rs1) {
        UnaryShadowCase::RdEqRs1 => {
            // srai rd, rd, shamt
            emit_asm!(translator ; sar Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // srai rd, rs1, shamt
            emit_asm!(translator ; mov Rq(rd.id()), Rq(rs1.id()));
            emit_asm!(translator ; sar Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `slt`: set if less than (signed).
/// rd <- 1 if signed(rs1) < signed(rs2) else 0
pub(super) fn emit_slt(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator);

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
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // 0 < rs2 iff rs2 > 0.
            emit_asm!(translator ; test Rq(rs2.id()), Rq(rs2.id()));
            emit_asm!(translator ; setg Rb(rd.id()));
            emit_asm!(translator ; movzx Rq(rd.id()), Rb(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // rs1 < 0.
            emit_asm!(translator ; test Rq(rs1.id()), Rq(rs1.id()));
            emit_asm!(translator ; setl Rb(rd.id()));
            emit_asm!(translator ; movzx Rq(rd.id()), Rb(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {
            // Fall through to the generic handler.
        }
    }

    if rs1.id() == rs2.id() {
        // since they are equal they can't be less than
        // hence we set rd = 0
        emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
        return;
    } else {
        emit_asm!(translator ; cmp Rq(rs1.id()), Rq(rs2.id()));
        emit_asm!(translator ; setl Rb(rd.id()));
        emit_asm!(translator ; movzx Rq(rd.id()), Rb(rd.id()));
        ctx.write_back(translator);
        return;
    }
}

/// RV64 `sltu`: set if less than (unsigned).
/// rd <- 1 if unsigned(rs1) < unsigned(rs2) else 0
pub(super) fn emit_sltu(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator);

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
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // unsigned 0 < rs2 iff rs2 != 0.
            emit_asm!(translator ; test Rq(rs2.id()), Rq(rs2.id()));
            emit_asm!(translator ; setne Rb(rd.id()));
            emit_asm!(translator ; movzx Rq(rd.id()), Rb(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // unsigned rs1 < 0 is always false.
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {
            // Fall through to the generic handler.
        }
    }

    if rs1.id() == rs2.id() {
        // since they are equal
        // rd = 0
        emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
        return;
    } else {
        emit_asm!(translator ; cmp Rq(rs1.id()), Rq(rs2.id()));
        emit_asm!(translator ; setb Rb(rd.id()));
        emit_asm!(translator ; movzx Rq(rd.id()), Rb(rd.id()));
        ctx.write_back(translator);
        return;
    }
}

/// RV64 `slti`: set if less than immediate (signed).
/// rd <- 1 if signed(rs1) < sext(imm) else 0
pub(super) fn emit_slti(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

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
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::Rs1Zero => {
            // slti rd, 0, imm
            if imm > 0 {
                // rd = 1
                emit_asm!(translator ; mov Rq(rd.id()), 1);
                ctx.write_back(translator);
                return;
            } else {
                // rd = 0
                emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
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

    emit_asm!(translator ; cmp Rq(rs1.id()), imm);
    emit_asm!(translator ; setl Rb(rd.id()));
    emit_asm!(translator ; movzx Rq(rd.id()), Rb(rd.id()));
    ctx.write_back(translator);
    return;
}

/// RV64 `sltiu`: set if less than immediate (unsigned).
/// rd <- 1 if unsigned(rs1) < sext(imm) else 0
pub(super) fn emit_sltiu(translator: &Translator, rd: RiscvRegister, rs1: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<1, 0>::new()
        .set_inputs([rs1])
        .set_output(rd)
        .build(translator);

    let [rs1] = ctx.inputs();
    let rd = ctx.output();

    match classify_unary_zero_case(rd, rs1, imm) {
        UnaryZeroCase::RdZero => {
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero => {
            // sltiu rd, 0, 0
            // -> rd = 0
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::Rs1Zero => {
            // sltiu rd, 0, imm
            // since we are dealing with unsigned
            // 0 is the lowest value
            // hence rd = 0 only when imm also equals 0
            //
            // given that we already handled the Rs1ImmZero case
            // then we can be sure that imm != 0
            //
            // hence:
            //
            // rd = 1
            emit_asm!(translator ; mov Rq(rd.id()), 1);
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // sltiu rd, rs1, 0
            //
            // rs1 cannot be a zero because
            // Rs1ImmZero didn't execute
            // so rs1 > imm
            // which means rd = 0
            emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    emit_asm!(translator ; cmp Rq(rs1.id()), imm);
    emit_asm!(translator ; setb Rb(rd.id()));
    emit_asm!(translator ; movzx Rq(rd.id()), Rb(rd.id()));
    ctx.write_back(translator);
    return;
}

/// RV64 `blt`: branch if less than (signed).
/// if signed(rs1) < signed(rs2) then pc <- pc + sext(imm)
pub(super) fn emit_blt(translator: &Translator, rs1: RiscvRegister, rs2: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();

    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            // rs1 == rs2
            // so we don't jump
        }
        (true, false) => {
            // 0 < rs2 iff rs2 > 0.
            emit_asm!(translator ; test Rq(rs2.id()), Rq(rs2.id()));
            emit_asm!(translator ; jg => target_label);
        }
        (false, true) => {
            // rs1 < 0.
            emit_asm!(translator ; test Rq(rs1.id()), Rq(rs1.id()));
            emit_asm!(translator ; jl => target_label);
        }
        (false, false) => {
            emit_asm!(translator ; cmp Rq(rs1.id()), Rq(rs2.id()));
            emit_asm!(translator ; jl => target_label);
        }
    }

    ctx.complete_no_output(translator);
}

/// RV64 `bge`: branch if greater or equal (signed).
/// if signed(rs1) >= signed(rs2) then pc <- pc + sext(imm)
pub(super) fn emit_bge(translator: &Translator, rs1: RiscvRegister, rs2: RiscvRegister, imm: i32) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator);

    let [rs1, rs2] = ctx.inputs();

    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    let target_label = translator.target_label(branch_target);

    match (rs1.is_zero(), rs2.is_zero()) {
        (true, true) => {
            // since equal we should jump
            emit_asm!(translator ; jmp => target_label);
        }
        (true, false) => {
            // 0 >= rs2 iff rs2 <= 0.
            emit_asm!(translator ; test Rq(rs2.id()), Rq(rs2.id()));
            emit_asm!(translator ; jle => target_label);
        }
        (false, true) => {
            // rs1 >= 0.
            emit_asm!(translator ; test Rq(rs1.id()), Rq(rs1.id()));
            emit_asm!(translator ; jge => target_label);
        }
        (false, false) => {
            emit_asm!(translator ; cmp Rq(rs1.id()), Rq(rs2.id()));
            emit_asm!(translator ; jge => target_label);
        }
    }

    ctx.complete_no_output(translator);
}
