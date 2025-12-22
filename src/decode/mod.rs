mod insn_formats;

use insn_formats::{B, I, J, R, S, Sh, U};

use crate::util::mask32;

// TODO add better error handling

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

    // Illegal Instruction
    Illegal(u32),
}

/// Extracts the opcode value from a 32 bit insn
#[inline]
fn opcode(insn: u32) -> u8 {
    (insn & mask32(7)) as u8
}

#[inline]
fn funct3(insn: u32) -> u8 {
    ((insn >> 12) & mask32(3)) as u8
}

#[inline]
fn funct7(insn: u32) -> u8 {
    ((insn >> 25) & mask32(7)) as u8
}

#[inline]
fn rd(insn: u32) -> u8 {
    ((insn >> 7) & mask32(5)) as u8
}

#[inline]
fn rs1(insn: u32) -> u8 {
    ((insn >> 15) & mask32(5)) as u8
}

#[inline]
fn rs2(insn: u32) -> u8 {
    ((insn >> 20) & mask32(5)) as u8
}

#[inline]
fn imm_i(insn: u32) -> i32 {
    todo!()
}

#[inline]
fn imm_s(insn: u32) -> i32 {
    todo!()
}

#[inline]
fn imm_b(insn: u32) -> i32 {
    todo!()
}

#[inline]
fn imm_j(insn: u32) -> i32 {
    todo!()
}

#[inline]
fn imm_u(insn: u32) -> i32 {
    todo!()
}

#[inline]
fn shamt5(insn: u32) -> u8 {
    todo!()
}

#[inline]
fn shamt6(insn: u32) -> u8 {
    todo!()
}
fn decode(insn: u32) -> Instruction {
    match opcode(insn) {
        0b0110011 => decode_op(insn),
        0b0010011 => decode_op_imm(insn),
        0b0000011 => decode_load(insn),
        0b0100011 => decode_store(insn),
        0b1100011 => decode_branch(insn),
        0b1101111 => decode_jal(insn),
        0b1100111 => decode_jalr(insn),
        0b0110111 => decode_lui(insn),
        0b0010111 => decode_auipc(insn),
        0b1110011 => decode_system(insn),
        0b0001111 => decode_fence(insn),
        _ => Instruction::Illegal(insn),
    }
}

fn decode_op(insn: u32) -> Instruction {
    let insn_operands = R {
        rd: rd(insn),
        rs1: rs1(insn),
        rs2: rs2(insn),
    };

    match (funct3(insn), funct7(insn)) {
        (0x0, 0x00) => Instruction::Add(insn_operands),
        (0x0, 0x20) => Instruction::Sub(insn_operands),
        (0x4, 0x00) => Instruction::Xor(insn_operands),
        (0x6, 0x00) => Instruction::Or(insn_operands),
        (0x7, 0x00) => Instruction::And(insn_operands),
        (0x1, 0x00) => Instruction::Sll(insn_operands),
        (0x5, 0x00) => Instruction::Srl(insn_operands),
        (0x5, 0x20) => Instruction::Sra(insn_operands),
        (0x2, 0x00) => Instruction::Slt(insn_operands),
        (0x3, 0x00) => Instruction::Sltu(insn_operands),
        _ => Instruction::Illegal(insn),
    }
}

fn decode_op_imm(insn: u32) -> Instruction {
    let rd = rd(insn);
    let rs1 = rs1(insn);
    let imm = imm_i(insn);

    let i_operands = I { rd, rs1, imm };
    let s_operands = Sh {
        rd,
        rs1,
        shamt: shamt6(insn),
    };

    match funct3(insn) {
        0x0 => Instruction::Addi(i_operands),
        0x4 => Instruction::Xori(i_operands),
        0x6 => Instruction::Ori(i_operands),
        0x7 => Instruction::Andi(i_operands),
        0x2 => Instruction::Slti(i_operands),
        0x3 => Instruction::Sltiu(i_operands),
        0x1 | 0x5 => match (funct3(insn), funct7(insn)) {
            (0x0, 0x00) => Instruction::Slli(s_operands),
            (0x5, 0x00) => Instruction::Srli(s_operands),
            (0x5, 0x20) => Instruction::Srai(s_operands),
            _ => Instruction::Illegal(insn),
        },
        _ => Instruction::Illegal(insn),
    }
}

fn decode_load(insn: u32) -> Instruction {
    let operand = I {
        rd: rd(insn),
        rs1: rs1(insn),
        imm: imm_i(insn),
    };

    match funct3(insn) {
        0x0 => Instruction::Lb(operand),
        0x1 => Instruction::Lh(operand),
        0x2 => Instruction::Lw(operand),
        0x4 => Instruction::Lbu(operand),
        0x5 => Instruction::Lhu(operand),
        _ => Instruction::Illegal(insn),
    }
}

fn decode_store(insn: u32) -> Instruction {
    let operand = S {
        rs1: rs1(insn),
        rs2: rs2(insn),
        imm: imm_s(insn),
    };

    match funct3(insn) {
        0x0 => Instruction::Sb(operand),
        0x1 => Instruction::Sh(operand),
        0x2 => Instruction::Sw(operand),
        _ => Instruction::Illegal(insn),
    }
}

fn decode_branch(insn: u32) -> Instruction {
    let operand = B {
        rs1: rs1(insn),
        rs2: rs2(insn),
        imm: imm_s(insn),
    };

    match funct3(insn) {
        0x0 => Instruction::Beq(operand),
        0x1 => Instruction::Bne(operand),
        0x4 => Instruction::Blt(operand),
        0x5 => Instruction::Bge(operand),
        0x6 => Instruction::Bltu(operand),
        0x7 => Instruction::Bgeu(operand),
        _ => Instruction::Illegal(insn),
    }
}

fn decode_jal(insn: u32) -> Instruction {
    Instruction::Jal(J {
        rd: rd(insn),
        imm: imm_j(insn),
    })
}
