mod rv64a;
mod rv64i;
mod rv64m;
mod rv64w;

use crate::aot::{registers::RiscvRegister, translator::Translator};
use rax_core::decode::{Instruction, Sh, B, I, J, R, S, U};

pub(super) fn emit_instruction(
    translator: &Translator,
    insn: &Instruction,
) {
    match insn {
        Instruction::Add(R { rd, rs1, rs2 }) => {
            rv64i::emit_add(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sub(R { rd, rs1, rs2 }) => {
            rv64i::emit_sub(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Or(R { rd, rs1, rs2 }) => {
            rv64i::emit_or(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Subw(R { rd, rs1, rs2 }) => {
            rv64w::emit_subw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulhu(R { rd, rs1, rs2 }) => {
            rv64m::emit_mulhu(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Addi(I { rd, rs1, imm }) => {
            rv64i::emit_addi(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Andi(I { rd, rs1, imm }) => {
            rv64i::emit_andi(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Slli(Sh { rd, rs1, shamt }) => {
            rv64i::emit_slli(translator, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Sll(R { rd, rs1, rs2 }) => {
            rv64i::emit_sll(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sb(S { rs1, rs2, imm }) => {
            rv64i::emit_sb(translator, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Sd(S { rs1, rs2, imm }) => {
            rv64i::emit_sd(translator, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Lui(U { rd, imm }) => rv64i::emit_lui(translator, rv(rd), *imm),
        Instruction::Auipc(U { rd, imm }) => rv64i::emit_auipc(translator, rv(rd), *imm),
        Instruction::Beq(B { rs1, rs2, imm }) => {
            rv64i::emit_beq(translator, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bne(B { rs1, rs2, imm }) => {
            rv64i::emit_bne(translator, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bltu(B { rs1, rs2, imm }) => {
            rv64i::emit_bltu(translator, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bgeu(B { rs1, rs2, imm }) => {
            rv64i::emit_bgeu(translator, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Jal(J { rd, imm }) => rv64i::emit_jal(translator, rv(rd), *imm),
        Instruction::Jalr(I { rd, rs1, imm }) => {
            rv64i::emit_jalr(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Ecall => rv64i::emit_ecall(translator),
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
        Instruction::Lb(I { rd, rs1, imm }) => {
            rv64i::emit_lb(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Lbu(I { rd, rs1, imm }) => {
            rv64i::emit_lbu(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Lh(I { rd, rs1, imm }) => {
            rv64i::emit_lh(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Lhu(I { rd, rs1, imm }) => {
            rv64i::emit_lhu(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Lw(I { rd, rs1, imm }) => {
            rv64i::emit_lw(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Lwu(I { rd, rs1, imm }) => {
            rv64i::emit_lwu(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Ld(I { rd, rs1, imm }) => {
            rv64i::emit_ld(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Sh(S { rs1, rs2, imm }) => {
            rv64i::emit_sh(translator, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Sw(S { rs1, rs2, imm }) => {
            rv64i::emit_sw(translator, rv(rs1), rv(rs2), *imm)
        }
        Instruction::And(R { rd, rs1, rs2 }) => {
            rv64i::emit_and(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Xor(R { rd, rs1, rs2 }) => {
            rv64i::emit_xor(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Ori(I { rd, rs1, imm }) => {
            rv64i::emit_ori(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Xori(I { rd, rs1, imm }) => {
            rv64i::emit_xori(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Srl(R { rd, rs1, rs2 }) => {
            rv64i::emit_srl(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sra(R { rd, rs1, rs2 }) => {
            rv64i::emit_sra(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Srli(Sh { rd, rs1, shamt }) => {
            rv64i::emit_srli(translator, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Srai(Sh { rd, rs1, shamt }) => {
            rv64i::emit_srai(translator, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Sraiw(Sh { rd, rs1, shamt }) => {
            rv64w::emit_sraiw(translator, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Sllw(R { rd, rs1, rs2 }) => {
            rv64w::emit_sllw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Srlw(R { rd, rs1, rs2 }) => {
            rv64w::emit_srlw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sraw(R { rd, rs1, rs2 }) => {
            rv64w::emit_sraw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Slliw(Sh { rd, rs1, shamt }) => {
            rv64w::emit_slliw(translator, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Srliw(Sh { rd, rs1, shamt }) => {
            rv64w::emit_srliw(translator, rv(rd), rv(rs1), *shamt)
        }
        Instruction::Slt(R { rd, rs1, rs2 }) => {
            rv64i::emit_slt(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Sltu(R { rd, rs1, rs2 }) => {
            rv64i::emit_sltu(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Slti(I { rd, rs1, imm }) => {
            rv64i::emit_slti(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Sltiu(I { rd, rs1, imm }) => {
            rv64i::emit_sltiu(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Addw(R { rd, rs1, rs2 }) => {
            rv64w::emit_addw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Addiw(I { rd, rs1, imm }) => {
            rv64w::emit_addiw(translator, rv(rd), rv(rs1), *imm)
        }
        Instruction::Blt(B { rs1, rs2, imm }) => {
            rv64i::emit_blt(translator, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Bge(B { rs1, rs2, imm }) => {
            rv64i::emit_bge(translator, rv(rs1), rv(rs2), *imm)
        }
        Instruction::Mul(R { rd, rs1, rs2 }) => {
            rv64m::emit_mul(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulh(R { rd, rs1, rs2 }) => {
            rv64m::emit_mulh(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulhsu(R { rd, rs1, rs2 }) => {
            rv64m::emit_mulhsu(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Mulw(R { rd, rs1, rs2 }) => {
            rv64m::emit_mulw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Div(R { rd, rs1, rs2 }) => {
            rv64m::emit_div(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Divu(R { rd, rs1, rs2 }) => {
            rv64m::emit_divu(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Rem(R { rd, rs1, rs2 }) => {
            rv64m::emit_rem(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Remu(R { rd, rs1, rs2 }) => {
            rv64m::emit_remu(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Divw(R { rd, rs1, rs2 }) => {
            rv64m::emit_divw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Divuw(R { rd, rs1, rs2 }) => {
            rv64m::emit_divuw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Remw(R { rd, rs1, rs2 }) => {
            rv64m::emit_remw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::Remuw(R { rd, rs1, rs2 }) => {
            rv64m::emit_remuw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::LrW(R { rd, rs1, rs2 }) => {
            rv64a::emit_lrw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::LrD(R { rd, rs1, rs2 }) => {
            rv64a::emit_lrd(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::ScW(R { rd, rs1, rs2 }) => {
            rv64a::emit_scw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::ScD(R { rd, rs1, rs2 }) => {
            rv64a::emit_scd(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoAddW(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoaddw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoSwapW(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoswapw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoXorW(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoxorw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoAndW(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoandw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoAddD(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoaddd(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoSwapD(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoswapd(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoXorD(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoxord(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoAndD(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoandd(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoOrW(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoorw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoOrD(R { rd, rs1, rs2 }) => {
            rv64a::emit_amoodd(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMinW(R { rd, rs1, rs2 }) => {
            rv64a::emit_amominw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMinD(R { rd, rs1, rs2 }) => {
            rv64a::emit_amomind(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMaxW(R { rd, rs1, rs2 }) => {
            rv64a::emit_amomaxw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMaxD(R { rd, rs1, rs2 }) => {
            rv64a::emit_amomaxd(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMinuW(R { rd, rs1, rs2 }) => {
            rv64a::emit_amominuw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMinuD(R { rd, rs1, rs2 }) => {
            rv64a::emit_amominud(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMaxuW(R { rd, rs1, rs2 }) => {
            rv64a::emit_amomaxuw(translator, rv(rd), rv(rs1), rv(rs2))
        }
        Instruction::AmoMaxuD(R { rd, rs1, rs2 }) => {
            rv64a::emit_amomaxud(translator, rv(rd), rv(rs1), rv(rs2))
        }
        _ => panic!("unknown opcode: {:?}", insn),
    }
}

fn rv(reg: &u8) -> RiscvRegister {
    RiscvRegister::from_index(*reg as usize).expect("invalid decoded RISC-V register")
}
