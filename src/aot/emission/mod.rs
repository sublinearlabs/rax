mod rv64i;
mod rv64w;
mod rv64m;
mod rv64a;

use crate::aot::{registers::RiscvRegister, temp_alloc::TempAllocator, translator::Translator};
use crate::decode::{Instruction, Sh, B, I, J, R, S, U};

pub(super) fn emit_instruction(
    translator: &mut Translator,
    temps: &TempAllocator,
    insn: &Instruction,
) {
    match insn {
        Instruction::Add(R { rd, rs1, rs2 }) => {
            rv64i::emit_add(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sub(R { rd, rs1, rs2 }) => {
            rv64i::emit_sub(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Or(R { rd, rs1, rs2 }) => rv64i::emit_or(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::Subw(R { rd, rs1, rs2 }) => {
            rv64w::emit_subw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulhu(R { rd, rs1, rs2 }) => {
            rv64m::emit_mulhu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Addi(I { rd, rs1, imm }) => {
            rv64i::emit_addi(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Andi(I { rd, rs1, imm }) => {
            rv64i::emit_andi(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Slli(Sh { rd, rs1, shamt }) => {
            rv64i::emit_slli(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Sll(R { rd, rs1, rs2 }) => {
            rv64i::emit_sll(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sb(S { rs1, rs2, imm }) => rv64i::emit_sb(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Sd(S { rs1, rs2, imm }) => rv64i::emit_sd(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Lui(U { rd, imm }) => rv64i::emit_lui(translator, temps, rv(rd), *imm),
        Instruction::Auipc(U { rd, imm }) => rv64i::emit_auipc(translator, temps, rv(rd), *imm),
        Instruction::Beq(B { rs1, rs2, imm }) => {
            rv64i::emit_beq(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bne(B { rs1, rs2, imm }) => {
            rv64i::emit_bne(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bltu(B { rs1, rs2, imm }) => {
            rv64i::emit_bltu(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bgeu(B { rs1, rs2, imm }) => {
            rv64i::emit_bgeu(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Jal(J { rd, imm }) => rv64i::emit_jal(translator, temps, rv(rd), *imm),
        Instruction::Jalr(I { rd, rs1, imm }) => {
            rv64i::emit_jalr(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Ecall => rv64i::emit_ecall(translator, temps),
        Instruction::Nop => {}
        Instruction::Ebreak => {}
        Instruction::Eother => {}
        Instruction::Illegal(_) => {}
        Instruction::Csrrw(_) => {}
        Instruction::Csrrs(_) => {}
        Instruction::Csrrc(_) => {}
        Instruction::Csrrwi(_) => {}
        Instruction::Csrrsi(_) => {}
        Instruction::Csrrci(_) => {}
        Instruction::Lb(I { rd, rs1, imm }) => rv64i::emit_lb(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lbu(I { rd, rs1, imm }) => rv64i::emit_lbu(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lh(I { rd, rs1, imm }) => rv64i::emit_lh(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lhu(I { rd, rs1, imm }) => rv64i::emit_lhu(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lw(I { rd, rs1, imm }) => rv64i::emit_lw(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Lwu(I { rd, rs1, imm }) => rv64i::emit_lwu(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Ld(I { rd, rs1, imm }) => rv64i::emit_ld(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Sh(S { rs1, rs2, imm }) => rv64i::emit_sh(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::Sw(S { rs1, rs2, imm }) => rv64i::emit_sw(translator, temps, rv(rs1), rv(rs2), *imm),
        Instruction::And(R { rd, rs1, rs2 }) => rv64i::emit_and(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::Xor(R { rd, rs1, rs2 }) => rv64i::emit_xor(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::Ori(I { rd, rs1, imm }) => rv64i::emit_ori(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Xori(I { rd, rs1, imm }) => rv64i::emit_xori(translator, temps, rv(rd), rv(rs1), *imm),
        Instruction::Srl(R { rd, rs1, rs2 }) => rv64i::emit_srl(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::Srli(Sh { rd, rs1, shamt }) => {
            rv64i::emit_srli(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Srai(Sh { rd, rs1, shamt }) => {
            rv64i::emit_srai(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Sraiw(Sh { rd, rs1, shamt }) => {
            rv64w::emit_sraiw(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Sllw(R { rd, rs1, rs2 }) => {
            rv64w::emit_sllw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Srlw(R { rd, rs1, rs2 }) => {
            rv64w::emit_srlw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Slliw(Sh { rd, rs1, shamt }) => {
            rv64w::emit_slliw(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Srliw(Sh { rd, rs1, shamt }) => {
            rv64w::emit_srliw(translator, temps, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Slt(R { rd, rs1, rs2 }) => rv64i::emit_slt(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::Sltu(R { rd, rs1, rs2 }) => {
            rv64i::emit_sltu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Slti(I { rd, rs1, imm }) => {
            rv64i::emit_slti(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Sltiu(I { rd, rs1, imm }) => {
            rv64i::emit_sltiu(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Addw(R { rd, rs1, rs2 }) => {
            rv64w::emit_addw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Addiw(I { rd, rs1, imm }) => {
            rv64w::emit_addiw(translator, temps, rv(rd), rv(rs1), *imm)
        }
        Instruction::Blt(B { rs1, rs2, imm }) => {
            rv64i::emit_blt(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bge(B { rs1, rs2, imm }) => {
            rv64i::emit_bge(translator, temps, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Mul(R { rd, rs1, rs2 }) => rv64m::emit_mul(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::Divu(R { rd, rs1, rs2 }) => {
            rv64m::emit_divu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Remu(R { rd, rs1, rs2 }) => {
            rv64m::emit_remu(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::LrW(R { rd, rs1, rs2 }) => rv64a::emit_lrw(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::LrD(R { rd, rs1, rs2 }) => rv64a::emit_lrd(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::ScW(R { rd, rs1, rs2 }) => rv64a::emit_scw(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::ScD(R { rd, rs1, rs2 }) => rv64a::emit_scd(translator, temps, rv(rd), rv(rs1), rv(rs2)),
        Instruction::AmoAddW(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoaddw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoAddD(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoaddd(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoOrW(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoorw(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoOrD(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoodd(translator, temps, rv(rd), rv(rs1), rv(rs2))
        }
        _ => panic!("unknown opcode: {:?}", insn),
    }
}

fn rv(reg: &u8) -> RiscvRegister {
    RiscvRegister::from_index(*reg as usize).expect("invalid decoded RISC-V register")
}
