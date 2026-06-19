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

pub(super) mod alu;
pub(super) mod atomic;
pub(super) mod control;
pub(super) mod immediate;
pub(super) mod m;
pub(super) mod memory;
pub(super) mod shift;
pub(super) mod word;

use self::{alu::*, atomic::*, control::*, immediate::*, m::*, memory::*, shift::*, word::*};

pub(super) fn emit_instruction(
    translator: &mut Translator,
    temps: &TempAllocator,
    insn: &Instruction,
) {
    match insn {
        Instruction::Nop => {}
        Instruction::Add(R { rd, rs1, rs2 }) => {
            emit_add(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sub(R { rd, rs1, rs2 }) => {
            emit_sub(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Xor(R { rd, rs1, rs2 }) => {
            emit_xor(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Or(R { rd, rs1, rs2 }) => emit_or(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::And(R { rd, rs1, rs2 }) => {
            emit_and(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Addw(R { rd, rs1, rs2 }) => {
            emit_addw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Subw(R { rd, rs1, rs2 }) => {
            emit_subw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sllw(R { rd, rs1, rs2 }) => {
            emit_sllw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Srlw(R { rd, rs1, rs2 }) => {
            emit_srlw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sraw(R { rd, rs1, rs2 }) => {
            emit_sraw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Slt(R { rd, rs1, rs2 }) => {
            emit_slt(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sltu(R { rd, rs1, rs2 }) => {
            emit_sltu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::LrW(R { rd, rs1, rs2 }) => {
            emit_lr_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::ScW(R { rd, rs1, rs2 }) => {
            emit_sc_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoSwapW(R { rd, rs1, rs2 }) => {
            emit_amo_swap_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoAddW(R { rd, rs1, rs2 }) => {
            emit_amo_add_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoXorW(R { rd, rs1, rs2 }) => {
            emit_amo_xor_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoAndW(R { rd, rs1, rs2 }) => {
            emit_amo_and_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoOrW(R { rd, rs1, rs2 }) => {
            emit_amo_or_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMinW(R { rd, rs1, rs2 }) => {
            emit_amo_min_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMaxW(R { rd, rs1, rs2 }) => {
            emit_amo_max_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMinuW(R { rd, rs1, rs2 }) => {
            emit_amo_minu_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMaxuW(R { rd, rs1, rs2 }) => {
            emit_amo_maxu_w(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::LrD(R { rd, rs1, rs2 }) => {
            emit_lr_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::ScD(R { rd, rs1, rs2 }) => {
            emit_sc_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoSwapD(R { rd, rs1, rs2 }) => {
            emit_amo_swap_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoAddD(R { rd, rs1, rs2 }) => {
            emit_amo_add_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoXorD(R { rd, rs1, rs2 }) => {
            emit_amo_xor_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoAndD(R { rd, rs1, rs2 }) => {
            emit_amo_and_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoOrD(R { rd, rs1, rs2 }) => {
            emit_amo_or_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMinD(R { rd, rs1, rs2 }) => {
            emit_amo_min_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMaxD(R { rd, rs1, rs2 }) => {
            emit_amo_max_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMinuD(R { rd, rs1, rs2 }) => {
            emit_amo_minu_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMaxuD(R { rd, rs1, rs2 }) => {
            emit_amo_maxu_d(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mul(R { rd, rs1, rs2 }) => {
            emit_mul(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulh(R { rd, rs1, rs2 }) => {
            emit_mulh(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulhsu(R { rd, rs1, rs2 }) => {
            emit_mulhsu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulhu(R { rd, rs1, rs2 }) => {
            emit_mulhu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Div(R { rd, rs1, rs2 }) => {
            emit_div(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Divu(R { rd, rs1, rs2 }) => {
            emit_divu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Rem(R { rd, rs1, rs2 }) => {
            emit_rem(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Remu(R { rd, rs1, rs2 }) => {
            emit_remu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulw(R { rd, rs1, rs2 }) => {
            emit_mulw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Divw(R { rd, rs1, rs2 }) => {
            emit_divw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Divuw(R { rd, rs1, rs2 }) => {
            emit_divuw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Remw(R { rd, rs1, rs2 }) => {
            emit_remw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Remuw(R { rd, rs1, rs2 }) => {
            emit_remuw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Addi(I { rd, rs1, imm }) => {
            emit_addi(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Andi(I { rd, rs1, imm }) => {
            emit_andi(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Xori(I { rd, rs1, imm }) => {
            emit_xori(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Ori(I { rd, rs1, imm }) => emit_ori(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Slti(I { rd, rs1, imm }) => {
            emit_slti(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Sltiu(I { rd, rs1, imm }) => {
            emit_sltiu(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Addiw(I { rd, rs1, imm }) => {
            emit_addiw(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Slli(Sh { rd, rs1, shamt }) => {
            emit_slli(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Srli(Sh { rd, rs1, shamt }) => {
            emit_srli(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Srai(Sh { rd, rs1, shamt }) => {
            emit_srai(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Slliw(Sh { rd, rs1, shamt }) => {
            emit_slliw(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Srliw(Sh { rd, rs1, shamt }) => {
            emit_srliw(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Sraiw(Sh { rd, rs1, shamt }) => {
            emit_sraiw(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Sll(R { rd, rs1, rs2 }) => {
            emit_sll(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Srl(R { rd, rs1, rs2 }) => {
            emit_srl(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sra(R { rd, rs1, rs2 }) => {
            emit_sra(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Lb(I { rd, rs1, imm }) => emit_lb(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lh(I { rd, rs1, imm }) => emit_lh(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lw(I { rd, rs1, imm }) => emit_lw(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lwu(I { rd, rs1, imm }) => emit_lwu(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Ld(I { rd, rs1, imm }) => emit_ld(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lbu(I { rd, rs1, imm }) => emit_lbu(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lhu(I { rd, rs1, imm }) => emit_lhu(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Sb(S { rs1, rs2, imm }) => emit_sb(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Sh(S { rs1, rs2, imm }) => emit_sh(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Sw(S { rs1, rs2, imm }) => emit_sw(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Sd(S { rs1, rs2, imm }) => emit_sd(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Lui(U { rd, imm }) => emit_lui(translator, temps, rv(rd), *imm),
        Instruction::Auipc(U { rd, imm }) => emit_auipc(translator, temps, rv(rd), *imm),
        Instruction::Beq(B { rs1, rs2, imm }) => {
            emit_beq(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bne(B { rs1, rs2, imm }) => {
            emit_bne(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Blt(B { rs1, rs2, imm }) => {
            emit_blt(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bge(B { rs1, rs2, imm }) => {
            emit_bge(translator, temps, rv(rs1), rv(rs2), *imm)
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
