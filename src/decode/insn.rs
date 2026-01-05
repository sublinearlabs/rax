use serde::{Deserialize, Serialize};

use crate::decode::insn_formats::{B, I, J, R, R4, RF, S, Sh, U};

#[derive(Debug, Clone, Deserialize)]
pub(crate) enum Instruction {
    // RV32I
    // Integer Register Register
    Add(R),
    Sub(R),
    Sll(R),
    Slt(R),
    Sltu(R),
    Xor(R),
    Srl(R),
    Sra(R),
    Or(R),
    And(R),
    // Integer Register Immediate
    Addi(I),
    Slti(I),
    Sltiu(I),
    Xori(I),
    Ori(I),
    Andi(I),
    Slli(Sh),
    Srli(Sh),
    Srai(Sh),
    // Loads
    Lb(I),
    Lh(I),
    Lw(I),
    Lbu(I),
    Lhu(I),
    // Stores
    Sb(S),
    Sh(S),
    Sw(S),
    // Branches
    Beq(B),
    Bne(B),
    Blt(B),
    Bge(B),
    Bltu(B),
    Bgeu(B),
    // Jumps
    Jal(J),
    Jalr(I),
    // Upper Immediates
    Lui(U),
    Auipc(U),
    // System
    Ecall,
    Ebreak,
    Eother,
    // Fence
    Fence,

    // RV64I
    // Register-Register
    Addw(R),
    Subw(R),
    Sllw(R),
    Srlw(R),
    Sraw(R),
    // Register-Immediate
    Addiw(I),
    Slliw(Sh),
    Srliw(Sh),
    Sraiw(Sh),
    // Loads
    Ld(I),
    Lwu(I),
    // Stores
    Sd(S),

    // M extension
    // RV32M
    Mul(R),
    Mulh(R),
    Mulhsu(R),
    Mulhu(R),
    Div(R),
    Divu(R),
    Rem(R),
    Remu(R),

    // RV64M
    Mulw(R),
    Divw(R),
    Divuw(R),
    Remw(R),
    Remuw(R),

    // A extension
    // RV32A
    LrW(R),
    ScW(R),
    AmoSwapW(R),
    AmoAddW(R),
    AmoXorW(R),
    AmoAndW(R),
    AmoOrW(R),
    AmoMinW(R),
    AmoMaxW(R),
    AmoMinuW(R),
    AmoMaxuW(R),

    // RV64A
    LrD(R),
    ScD(R),
    AmoSwapD(R),
    AmoAddD(R),
    AmoXorD(R),
    AmoAndD(R),
    AmoOrD(R),
    AmoMinD(R),
    AmoMaxD(R),
    AmoMinuD(R),
    AmoMaxuD(R),

    // F extension
    // RV32F
    Flw(I),
    Fsw(S),
    FmaddS(R4),
    FmsubS(R4),
    FnmsubS(R4),
    FnmaddS(R4),
    FaddS(RF),
    FsubS(RF),
    FmulS(RF),
    FdivS(RF),
    FsqrtS(RF),
    FsgnjS(RF),
    FsgnjnS(RF),
    FsgnjxS(RF),
    FminS(RF),
    FmaxS(RF),
    FeqS(RF),
    FltS(RF),
    FleS(RF),
    FcvtWS(RF),
    FcvtWuS(RF),
    FcvtSW(RF),
    FcvtSWu(RF),
    FmvXW(RF),
    FmvWX(RF),
    FclassS(RF),

    // RV64F
    FcvtLS(RF),
    FcvtLuS(RF),
    FcvtSL(RF),
    FcvtSLu(RF),

    // D extension
    // RV32D
    Fld(I),
    Fsd(S),
    FmaddD(R4),
    FmsubD(R4),
    FnmsubD(R4),
    FnmaddD(R4),
    FaddD(RF),
    FsubD(RF),
    FmulD(RF),
    FdivD(RF),
    FsqrtD(RF),
    FsgnjD(RF),
    FsgnjnD(RF),
    FsgnjxD(RF),
    FminD(RF),
    FmaxD(RF),
    FeqD(RF),
    FltD(RF),
    FleD(RF),
    FcvtWD(RF),
    FcvtWuD(RF),
    FcvtDW(RF),
    FcvtDWu(RF),
    FcvtSD(RF),
    FcvtDS(RF),
    FclassD(RF),

    // RV64D
    FcvtLD(RF),
    FcvtLuD(RF),
    FcvtDL(RF),
    FcvtDLu(RF),
    FmvXD(RF),
    FmvDX(RF),

    // Zicsr
    // CSR Register
    Csrrw(I),
    Csrrs(I),
    Csrrc(I),

    // CSR Register Immediate
    Csrrwi(I),
    Csrrsi(I),
    Csrrci(I),

    // Illegal Instruction
    Illegal(u32),
}

impl Instruction {
    pub fn rs1(&self) -> u8 {
        match self {
            Instruction::Add(r)
            | Instruction::Sub(r)
            | Instruction::Xor(r)
            | Instruction::Or(r)
            | Instruction::And(r)
            | Instruction::Sll(r)
            | Instruction::Srl(r)
            | Instruction::Sra(r)
            | Instruction::Slt(r)
            | Instruction::Sltu(r) => r.rs1,

            Instruction::Addi(i)
            | Instruction::Xori(i)
            | Instruction::Ori(i)
            | Instruction::Andi(i)
            | Instruction::Slti(i)
            | Instruction::Sltiu(i)
            | Instruction::Lb(i)
            | Instruction::Lh(i)
            | Instruction::Lw(i)
            | Instruction::Lbu(i)
            | Instruction::Lhu(i)
            | Instruction::Jalr(i) => i.rs1,

            Instruction::Slli(sh) | Instruction::Srli(sh) | Instruction::Srai(sh) => sh.rs1,

            Instruction::Sb(s) | Instruction::Sh(s) | Instruction::Sw(s) => s.rs1,

            Instruction::Beq(b)
            | Instruction::Bne(b)
            | Instruction::Blt(b)
            | Instruction::Bge(b)
            | Instruction::Bltu(b)
            | Instruction::Bgeu(b) => b.rs1,

            _ => 0,
        }
    }

    pub fn rs2(&self) -> u8 {
        match self {
            Instruction::Add(r)
            | Instruction::Sub(r)
            | Instruction::Xor(r)
            | Instruction::Or(r)
            | Instruction::And(r)
            | Instruction::Sll(r)
            | Instruction::Srl(r)
            | Instruction::Sra(r)
            | Instruction::Slt(r)
            | Instruction::Sltu(r) => r.rs2,

            Instruction::Sb(s) | Instruction::Sh(s) | Instruction::Sw(s) => s.rs2,

            Instruction::Beq(b)
            | Instruction::Bne(b)
            | Instruction::Blt(b)
            | Instruction::Bge(b)
            | Instruction::Bltu(b)
            | Instruction::Bgeu(b) => b.rs2,

            _ => 0,
        }
    }

    pub fn rs3(&self) -> u8 {
        match self {
            Instruction::FmaddS(r4)
            | Instruction::FmsubS(r4)
            | Instruction::FnmsubS(r4)
            | Instruction::FnmaddS(r4)
            | Instruction::FmaddD(r4)
            | Instruction::FmsubD(r4)
            | Instruction::FnmsubD(r4)
            | Instruction::FnmaddD(r4) => r4.rs3,
            _ => 0,
        }
    }

    pub fn rd(&self) -> u8 {
        match self {
            Instruction::Add(r)
            | Instruction::Sub(r)
            | Instruction::Xor(r)
            | Instruction::Or(r)
            | Instruction::And(r)
            | Instruction::Sll(r)
            | Instruction::Srl(r)
            | Instruction::Sra(r)
            | Instruction::Slt(r)
            | Instruction::Sltu(r) => r.rd as u8,

            Instruction::Addi(i)
            | Instruction::Xori(i)
            | Instruction::Ori(i)
            | Instruction::Andi(i)
            | Instruction::Slti(i)
            | Instruction::Sltiu(i)
            | Instruction::Lb(i)
            | Instruction::Lh(i)
            | Instruction::Lw(i)
            | Instruction::Lbu(i)
            | Instruction::Lhu(i)
            | Instruction::Jalr(i) => i.rd as u8,

            Instruction::Slli(sh) | Instruction::Srli(sh) | Instruction::Srai(sh) => sh.rd as u8,

            Instruction::Lui(u) | Instruction::Auipc(u) => u.rd as u8,

            Instruction::Jal(j) => j.rd as u8,
            _ => 0,
        }
    }

    pub fn imm(&self) -> u64 {
        match self {
            Instruction::Addi(i)
            | Instruction::Xori(i)
            | Instruction::Ori(i)
            | Instruction::Andi(i)
            | Instruction::Slti(i)
            | Instruction::Sltiu(i)
            | Instruction::Lb(i)
            | Instruction::Lh(i)
            | Instruction::Lw(i)
            | Instruction::Lbu(i)
            | Instruction::Lhu(i)
            | Instruction::Jalr(i) => i.imm as u64,

            Instruction::Slli(sh) | Instruction::Srli(sh) | Instruction::Srai(sh) => {
                sh.shamt as u64
            }

            Instruction::Sb(s) | Instruction::Sh(s) | Instruction::Sw(s) => s.imm as u64,

            Instruction::Beq(b)
            | Instruction::Bne(b)
            | Instruction::Blt(b)
            | Instruction::Bge(b)
            | Instruction::Bltu(b)
            | Instruction::Bgeu(b) => b.imm as u64,

            Instruction::Lui(u) | Instruction::Auipc(u) => u.imm as u64,

            Instruction::Jal(j) => j.imm as u64,

            _ => 0,
        }
    }

    pub fn rm(&self) -> u8 {
        match self {
            Instruction::FaddS(rf)
            | Instruction::FsubS(rf)
            | Instruction::FmulS(rf)
            | Instruction::FdivS(rf)
            | Instruction::FaddD(rf)
            | Instruction::FsubD(rf)
            | Instruction::FmulD(rf)
            | Instruction::FdivD(rf)
            | Instruction::FsqrtS(rf)
            | Instruction::FsqrtD(rf)
            | Instruction::FsgnjS(rf)
            | Instruction::FsgnjnS(rf)
            | Instruction::FsgnjxS(rf)
            | Instruction::FsgnjD(rf)
            | Instruction::FsgnjnD(rf)
            | Instruction::FsgnjxD(rf)
            | Instruction::FminS(rf)
            | Instruction::FmaxS(rf)
            | Instruction::FminD(rf)
            | Instruction::FmaxD(rf)
            | Instruction::FleS(rf)
            | Instruction::FltS(rf)
            | Instruction::FeqS(rf)
            | Instruction::FleD(rf)
            | Instruction::FltD(rf)
            | Instruction::FeqD(rf)
            | Instruction::FcvtWS(rf)
            | Instruction::FcvtWuS(rf)
            | Instruction::FcvtLS(rf)
            | Instruction::FcvtLuS(rf)
            | Instruction::FcvtWD(rf)
            | Instruction::FcvtWuD(rf)
            | Instruction::FcvtLD(rf)
            | Instruction::FcvtLuD(rf)
            | Instruction::FcvtSW(rf)
            | Instruction::FcvtSWu(rf)
            | Instruction::FcvtSL(rf)
            | Instruction::FcvtSLu(rf)
            | Instruction::FcvtDW(rf)
            | Instruction::FcvtDWu(rf)
            | Instruction::FcvtDL(rf)
            | Instruction::FcvtDLu(rf)
            | Instruction::FcvtSD(rf)
            | Instruction::FcvtDS(rf)
            | Instruction::FmvXW(rf)
            | Instruction::FclassS(rf)
            | Instruction::FmvWX(rf)
            | Instruction::FmvXD(rf)
            | Instruction::FclassD(rf)
            | Instruction::FmvDX(rf) => rf.rm,

            Instruction::FmaddS(r4)
            | Instruction::FmsubS(r4)
            | Instruction::FnmsubS(r4)
            | Instruction::FnmaddS(r4)
            | Instruction::FmaddD(r4)
            | Instruction::FmsubD(r4)
            | Instruction::FnmsubD(r4)
            | Instruction::FnmaddD(r4) => r4.rm,

            _ => 0,
        }
    }
}
