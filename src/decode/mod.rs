mod imm;
mod insn;
mod insn_formats;
mod util;

use imm::{imm_b, imm_i, imm_j, imm_s, imm_u, shamt6};
use insn::Instruction;
use insn_formats::{B, I, J, R, S, Sh, U};
use util::{funct3, funct7, opcode, rd, rs1, rs2};

use crate::decode::imm::shamt5;

fn decode(insn: u32) -> Instruction {
    match opcode(insn) {
        0b0110011 => decode_op(insn),
        0b0010011 => decode_op_imm(insn),
        0b0111011 => decode_op_32(insn),
        0b0011011 => decode_op_imm_32(insn),

        0b0000011 => decode_load(insn),
        0b0100011 => decode_store(insn),
        0b1100011 => decode_branch(insn),
        0b1101111 => decode_jal(insn),
        0b1100111 => decode_jalr(insn),
        0b0110111 => decode_lui(insn),
        0b0010111 => decode_auipc(insn),
        0b1110011 => decode_system(insn),
        0b0001111 => decode_fence(insn),

        0b0101111 => decode_atomics(insn),

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

        (0x0, 0x01) => Instruction::Mul(insn_operands),
        (0x1, 0x01) => Instruction::Mulh(insn_operands),
        (0x2, 0x01) => Instruction::Mulhsu(insn_operands),
        (0x3, 0x01) => Instruction::Mulhu(insn_operands),
        (0x4, 0x01) => Instruction::Div(insn_operands),
        (0x5, 0x01) => Instruction::Divu(insn_operands),
        (0x6, 0x01) => Instruction::Rem(insn_operands),
        (0x7, 0x01) => Instruction::Remu(insn_operands),

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

fn decode_op_32(insn: u32) -> Instruction {
    let operands = R {
        rd: rd(insn),
        rs1: rs1(insn),
        rs2: rs2(insn),
    };

    match (funct3(insn), funct7(insn)) {
        (0x0, 0x00) => Instruction::Addw(operands),
        (0x0, 0x20) => Instruction::Subw(operands),
        (0x1, 0x00) => Instruction::Sllw(operands),
        (0x5, 0x00) => Instruction::Srlw(operands),
        (0x5, 0x20) => Instruction::Sraw(operands),

        (0x0, 0x01) => Instruction::Mulw(operands),
        (0x4, 0x01) => Instruction::Divw(operands),
        (0x5, 0x01) => Instruction::Divuw(operands),
        (0x6, 0x01) => Instruction::Remw(operands),
        (0x7, 0x01) => Instruction::Remuw(operands),

        _ => Instruction::Illegal(insn),
    }
}

fn decode_op_imm_32(insn: u32) -> Instruction {
    let rd = rd(insn);
    let rs1 = rs1(insn);
    let imm = imm_i(insn);

    let i_operands = I { rd, rs1, imm };
    let s_operands = Sh {
        rd,
        rs1,
        shamt: shamt5(insn),
    };

    match (funct3(insn), funct7(insn)) {
        (0x0, _) => Instruction::Addiw(i_operands),
        (0x1, 0x00) => Instruction::Slliw(s_operands),
        (0x5, 0x00) => Instruction::Srliw(s_operands),
        (0x5, 0x20) => Instruction::Sraiw(s_operands),
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
        0x3 => Instruction::Ld(operand),
        0x4 => Instruction::Lbu(operand),
        0x5 => Instruction::Lhu(operand),
        0x6 => Instruction::Lwu(operand),
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
        0x3 => Instruction::Sd(operand),
        _ => Instruction::Illegal(insn),
    }
}

fn decode_branch(insn: u32) -> Instruction {
    let operand = B {
        rs1: rs1(insn),
        rs2: rs2(insn),
        imm: imm_b(insn),
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

fn decode_jalr(insn: u32) -> Instruction {
    match funct3(insn) {
        0x0 => Instruction::Jalr(I {
            rd: rd(insn),
            rs1: rs1(insn),
            imm: imm_i(insn),
        }),
        _ => Instruction::Illegal(insn),
    }
}

fn decode_lui(insn: u32) -> Instruction {
    Instruction::Lui(U {
        rd: rd(insn),
        imm: imm_u(insn),
    })
}

fn decode_auipc(insn: u32) -> Instruction {
    Instruction::Auipc(U {
        rd: rd(insn),
        imm: imm_u(insn),
    })
}

fn decode_system(insn: u32) -> Instruction {
    let imm = imm_i(insn);
    match (funct3(insn), imm) {
        (0x0, 0x0) => Instruction::Ecall,
        (0x0, 0x1) => Instruction::Ebreak,
        _ => Instruction::Illegal(insn),
    }
}

fn decode_fence(_insn: u32) -> Instruction {
    Instruction::Fence
}

fn decode_atomics(insn: u32) -> Instruction {
    let funct5 = funct7(insn) >> 2;

    let rd = rd(insn);
    let rs1 = rs1(insn);
    let rs2 = rs2(insn);

    let operand = R { rd, rs1, rs2 };

    match (funct3(insn), funct5) {
        (0x2, 0x02) if rs2 == 0 => Instruction::LrW(operand),
        (0x2, 0x03) => Instruction::ScW(operand),
        (0x2, 0x01) => Instruction::AmoSwapW(operand),
        (0x2, 0x00) => Instruction::AmoAddW(operand),
        (0x2, 0x04) => Instruction::AmoXorW(operand),
        (0x2, 0x0c) => Instruction::AmoAndW(operand),
        (0x2, 0x08) => Instruction::AmoOrW(operand),
        (0x2, 0x10) => Instruction::AmoMinW(operand),
        (0x2, 0x14) => Instruction::AmoMaxW(operand),
        (0x2, 0x18) => Instruction::AmoMinuW(operand),
        (0x2, 0x1c) => Instruction::AmoMaxuW(operand),

        (0x3, 0x02) if rs2 == 0 => Instruction::LrD(operand),
        (0x3, 0x03) => Instruction::ScD(operand),
        (0x3, 0x01) => Instruction::AmoSwapD(operand),
        (0x3, 0x00) => Instruction::AmoAddD(operand),
        (0x3, 0x04) => Instruction::AmoXorD(operand),
        (0x3, 0x0C) => Instruction::AmoAndD(operand),
        (0x3, 0x08) => Instruction::AmoOrD(operand),
        (0x3, 0x10) => Instruction::AmoMinD(operand),
        (0x3, 0x14) => Instruction::AmoMaxD(operand),
        (0x3, 0x18) => Instruction::AmoMinuD(operand),
        (0x3, 0x1c) => Instruction::AmoMaxuD(operand),

        _ => Instruction::Illegal(insn),
    }
}
