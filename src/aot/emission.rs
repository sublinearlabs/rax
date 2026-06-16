use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};

use crate::aot::{
    classification::{
        classify_shadow_case, classify_unary_shadow_case, classify_unary_zero_case,
        classify_zero_case, ShadowCase, UnaryShadowCase, UnaryZeroCase, ZeroCase,
    },
    instruction_context::InstructionContextBuilder,
    registers::{RiscvRegister, X86Gpr},
    temp_alloc::TempAllocator,
    translator::Translator,
};
use crate::decode::{Instruction, Sh, B, I, J, R, S, U};

pub(super) fn emit_instruction(
    translator: &mut Translator,
    temps: &TempAllocator,
    insn: &Instruction,
) {
    match insn {
        Instruction::Add(R { rd, rs1, rs2 }) => {
            emit_add(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sub(R { rd, rs1, rs2 }) => {
            emit_sub(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Or(R { rd, rs1, rs2 }) => emit_or(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::Subw(R { rd, rs1, rs2 }) => {
            emit_subw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulhu(R { rd, rs1, rs2 }) => {
            emit_mulhu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Addi(I { rd, rs1, imm }) => {
            emit_addi(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Andi(I { rd, rs1, imm }) => {
            emit_andi(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Slli(Sh { rd, rs1, shamt }) => {
            emit_slli(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Sll(R { rd, rs1, rs2 }) => {
            emit_sll(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sb(S { rs1, rs2, imm }) => emit_sb(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Sd(S { rs1, rs2, imm }) => emit_sd(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Lui(U { rd, imm }) => emit_lui(translator, temps, rv(rd), *imm),
        Instruction::Auipc(U { rd, imm }) => emit_auipc(translator, temps, rv(rd), *imm),
        Instruction::Beq(B { rs1, rs2, imm }) => {
            emit_beq(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bne(B { rs1, rs2, imm }) => {
            emit_bne(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bltu(B { rs1, rs2, imm }) => {
            emit_bltu(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bgeu(B { rs1, rs2, imm }) => {
            emit_bgeu(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Jal(J { rd, imm }) => emit_jal(translator, temps, rv(rd), *imm),
        Instruction::Jalr(I { rd, rs1, imm }) => {
            emit_jalr(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Ecall => emit_ecall(translator, temps),
        Instruction::Csrrw(_) => {}
        _ => panic!("unknown opcode: {:?}", insn),
    }
}

fn rv(reg: &u8) -> RiscvRegister {
    RiscvRegister::from_index(*reg as usize).expect("invalid decoded RISC-V register")
}

/// RV64 `add`: 64-bit wrapping addition.
/// rd <- (rs1 + rs2) mod 2^64
fn emit_add(
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
            // x0 is hardwired to zero, writes can be ignored
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // add rd, 0, 0 -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // add rd, 0, rs2 -> rd = rs2

            // if rd and rs2 point to the same register
            // no need to waste a mov instruction
            if rd.id() == rs2.id() {
                ctx.write_back(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // add rd, rs1, 0 -> rd = rs1

            // if rd and rs1 point to the same register
            // no need to waste a mov instruction
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
            // add rd, rd, rd
            // implies rd += rd
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // add rd, rd, rs2
            // imples rd += rs2
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // add rd, rs1, rd
            // implies rd += rs1
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::Rs1EqRs2 => {
            // add rd, rs1, rs1
            // implies rd = rs1 + rs1
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
fn emit_sub(
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
            // x0 is hardwired to zero. writes can be ignored.
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // sub rd, 0, 0 -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // sub rd, 0, rs2 -> rd = -rs2
            if rd.id() != rs2.id() {
                dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs2.id()));
            }

            dynasm!(translator.emitter ; neg Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // sub rd, rs1, 0 -> rd = rs1

            // if rd and rs1 point to the same register
            // no need to waste a mov instruction
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
            // sub rd, rd, rd
            // -> rd = rd - rd
            // -> rd = 0
            //
            // sub rd, rs1, rs1
            // -> rd = rs1 - rs1
            // -> rd = 0

            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // sub rd, rd, rs2
            // -> rd -= rs2

            dynasm!(translator.emitter ; sub Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // sub rd, rs1, rd
            // -> rd = rs1 - rd
            // negate the rd
            // then add rs1

            dynasm!(translator.emitter ; neg Rq(rd.id()));
            dynasm!(translator.emitter ; add Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // sub rd, rs1, rs2

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; sub Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `or`: bitwise OR across all 64 bits.
/// rd <- rs1 | rs2
fn emit_or(
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
            // x0 is hardwired to zero. writes can be ignored.
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // or rd, 0, 0
            // -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // or rd, 0, rs2
            // -> rd = rs2

            // if they point to the same register
            // no need to waste a mov
            if rd.id() == rs2.id() {
                ctx.write_back(translator);
                return;
            }

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // or rd, rs1, 0
            // -> rd = rs1

            // if they point to the same register
            // no need to waste a mov
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
            // or rd, rd, rd
            // -> rd = rd | rd
            // -> rd = rd

            // no emission needed
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // or rd, rd, rs2
            // -> rd = rd | rs2

            dynasm!(translator.emitter ; or Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // or rd, rs1, rd
            // -> rd = rd | rs1

            dynasm!(translator.emitter ; or Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::Rs1EqRs2 => {
            // or rd, rs1, rs1
            // -> rd = rs1 | rs1
            // -> rd = rs1

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // or rd, rs1, rs2

            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; or Rq(rd.id()), Rq(rs2.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `subw`: subtract low 32 bits, then sign-extend to 64 bits.
/// rd <- sext32((rs1[31:0] - rs2[31:0]) mod 2^32)
fn emit_subw(
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
            // x0 is hardwired to zero, writes can be ignored
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // subw rd, 0, 0
            // -> rd = 0

            // zero out the rd register
            dynasm!(translator.emitter ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // subw rd, 0, rs2
            // -> rd = 0 - rs2
            // -> rd = low32(-rs2)

            if rd.id() != rs2.id() {
                dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs2.id()));
            }

            dynasm!(translator.emitter ; neg Rd(rd.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // subw rd, rs1, 0
            // -> rd = low32(rs1)

            if rd.id() != rs1.id() {
                dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs1.id()));
            }

            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(&rd, &rs1, &rs2) {
        ShadowCase::AllEqual | ShadowCase::Rs1EqRs2 => {
            // subw rd, rd, rd
            // -> rd = 0

            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // subw rd, rd, rs2
            // -> rd -= rs2

            dynasm!(translator.emitter ; sub Rd(rd.id()), Rd(rs2.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // subw rd, rs1, rd
            // -> rd = rs1 - rd
            // neg rd
            // add rs1

            dynasm!(translator.emitter ; neg Rd(rd.id()));
            dynasm!(translator.emitter ; add Rd(rd.id()), Rd(rs1.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // subw rd, rs1, rs2

            dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs1.id()));
            dynasm!(translator.emitter ; sub Rd(rd.id()), Rd(rs2.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `mulhu`: upper 64 bits of unsigned 64x64 multiply.
/// rd <- high64(unsigned(rs1) * unsigned(rs2))
fn emit_mulhu(
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

/// RV64 `addi`: 64-bit wrapping add with sign-extended immediate.
/// rd <- (rs1 + sext(imm)) mod 2^64
fn emit_addi(
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
            // x0 is hardwired to zero, writes can be ignored
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero => {
            // addi rd, 0, 0 -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::Rs1Zero => {
            // addi rd, 0, imm -> rd = imm
            dynasm!(translator.emitter ; mov Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // addi rd, rs1, 0 -> rd = rs1
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
            // addi rd, rd, imm
            dynasm!(translator.emitter ; add Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // addi rd, rs1, imm
            dynasm!(translator.emitter ; lea Rq(rd.id()), [Rq(rs1.id()) + imm]);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `andi`: bitwise AND with sign-extended immediate.
/// rd <- rs1 & sext(imm)
fn emit_andi(
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
            // x0 is hardwired to zero, writes can be ignored
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero | UnaryZeroCase::Rs1Zero | UnaryZeroCase::ImmZero => {
            // in all cases, rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::None => {}
    }

    // andi rd, rs1, -1 preserves all bits, so it is just a move/no-op.
    // Handle it before shadow lowering to avoid emitting `and rd, -1`.
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
            // andi rd, rd, imm
            dynasm!(translator.emitter ; and Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // andi rd, rs1, imm
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; and Rq(rd.id()), imm);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `slli`: logical left shift by immediate.
/// rd <- rs1 << shamt
fn emit_slli(
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
            // x0 is hardwired to zero, writes can be ignored
            ctx.discard_zero_output(translator);
            return;
        }

        UnaryZeroCase::Rs1ImmZero | UnaryZeroCase::Rs1Zero => {
            // Rs1ImmZero
            // slli rd, 0, 0 -> rd = 0
            //
            // Rs1Zero
            // slli rd, 0, imm -> rd = 0
            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        UnaryZeroCase::ImmZero => {
            // slli rd, rs1, 0 -> rd = rs1
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
            // slli rd, rd, imm
            dynasm!(translator.emitter ; shl Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }

        UnaryShadowCase::Distinct => {
            // slli rd, rs1, imm
            dynasm!(translator.emitter ; mov Rq(rd.id()), Rq(rs1.id()));
            dynasm!(translator.emitter ; shl Rq(rd.id()), shamt as i8);
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sll`: logical left shift by register low bits.
/// rd <- rs1 << (rs2 & 0x3f)
fn emit_sll(
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

/// RV64 `sb`: store low 8 bits of rs2 to memory at rs1 + sext(imm).
/// mem8[rs1 + sext(imm)] <- rs2[7:0]
fn emit_sb(
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

/// RV64 `sd`: store 64 bits of rs2 to memory at rs1 + sext(imm).
/// mem64[rs1 + sext(imm)] <- rs2
fn emit_sd(
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

/// RV64 `lui`: write U-immediate to upper bits.
/// rd <- sext(imm << 12)
fn emit_lui(translator: &mut Translator, temps: &TempAllocator, rd: RiscvRegister, imm: i32) {
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
fn emit_auipc(translator: &mut Translator, temps: &TempAllocator, rd: RiscvRegister, imm: i32) {
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
fn emit_beq(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    // we need to trigger some compare
    // and then based on the result
    // jump to some location

    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();

    dynasm!(translator.emitter ; cmp Rq(rs1.id()), Rq(rs2.id()));

    // compute the target riscv pc
    let branch_target = translator.current_pc().wrapping_add(imm as i64 as u64);

    // retrieve or create a new dynamic label for the riscv pc
    let target_label = translator.target_label(branch_target);

    dynasm!(translator.emitter ; je => target_label);
}

/// RV64 `bne`: branch if not equal.
/// if rs1 != rs2 then pc <- pc + sext(imm)
fn emit_bne(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_bne")
}

/// RV64 `bltu`: branch if unsigned rs1 < rs2.
/// if unsigned(rs1) < unsigned(rs2) then pc <- pc + sext(imm)
fn emit_bltu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_bltu")
}

/// RV64 `bgeu`: branch if unsigned rs1 >= rs2.
/// if unsigned(rs1) >= unsigned(rs2) then pc <- pc + sext(imm)
fn emit_bgeu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    imm: i32,
) {
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_bgeu")
}

/// RV64 `jal`: jump and link.
/// rd <- pc + 4; pc <- pc + sext(imm)
fn emit_jal(translator: &mut Translator, temps: &TempAllocator, rd: RiscvRegister, imm: i32) {
    let _ = (translator, temps, rd, imm);
    todo!("emit_jal")
}

/// RV64 `jalr`: indirect jump and link.
/// t <- pc + 4; pc <- (rs1 + sext(imm)) & !1; rd <- t
fn emit_jalr(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    imm: i32,
) {
    let _ = (translator, temps, rd, rs1, imm);
    todo!("emit_jalr")
}

/// RV64 `ecall`: environment call trap.
/// raise environment-call-from-U-mode
fn emit_ecall(translator: &mut Translator, temps: &TempAllocator) {
    let _ = (translator, temps);
    todo!("emit_ecall")
}
