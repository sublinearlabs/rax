use crate::decode::insn_formats::{B, I, J, R, S, Sh, U};

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

    // Illegal Instruction
    Illegal(u32),
}
