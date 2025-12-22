mod insn_formats;

use insn_formats::{B, I, J, R, S, Sh, U};

use crate::util::mask32;

enum Instruction {
    // Base Instruction (I)
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
}

/// Extracts the opcode value from a 32 bit insn
#[inline]
fn opcode(insn: u32) -> u8 {
    (insn & mask32(7)) as u8
}

fn decode(insn: u32) -> Instruction {
    match opcode(insn) {
        _ => panic!("unimplemented"),
    }
}
