use crate::util::{mask, mask32};

// RISCV Opcodes
pub(crate) enum Opcode {
    Add,
    Sub,
    Xor,
    Or,
    And,
    Sll,
    Srl,
    Sra,
    Slt,
    Sltu,

    Addi,
    Xori,
    Ori,
    Andi,
    Slli,
    Srli,
    Srai,
    Slti,
    Sltiu,

    Lb,
    Lh,
    Lw,
    Lbu,
    Lhu,

    Sb,
    Sh,
    Sw,

    Beq,
    Bne,
    Blt,
    Bge,
    Bltu,
    Bgeu,

    Jal,
    Jalr,

    Lui,
    Auipc,

    Ecall,
    Ebreak,
    Eother,

    Fence,
}

enum InstructionType {
    R,
    I,
    S,
    B,
    U,
    J,
    FENCE,
}

// RISCV insturction
pub(crate) struct Instruction {
    pub(crate) opcode: Opcode,
    pub(crate) rd: usize,
    pub(crate) rs1: usize,
    pub(crate) rs2: usize,
    pub(crate) imm: u64,
}

impl Instruction {
    pub(crate) fn new(opcode: Opcode) -> Self {
        Self {
            opcode,
            rd: 0,
            rs1: 0,
            rs2: 0,
            imm: 0,
        }
    }

    pub(crate) fn rd(self, val: usize) -> Self {
        Self { rd: val, ..self }
    }

    pub(crate) fn rs1(self, val: usize) -> Self {
        Self { rs1: val, ..self }
    }

    pub(crate) fn rs2(self, val: usize) -> Self {
        Self { rs2: val, ..self }
    }

    pub(crate) fn imm(self, val: u64) -> Self {
        Self { imm: val, ..self }
    }
}

pub(crate) fn decode_insn(insn: u32) -> Instruction {
    let opcode_value = insn & mask32(7);

    let insn_type = match opcode_value {
        0b0110011 => InstructionType::R,
        0b0010011 | 0b0000011 | 0b1100111 | 0b1110011 => InstructionType::I,
        0b0100011 => InstructionType::S,
        0b1100011 => InstructionType::B,
        0b0110111 => InstructionType::U,
        0b1101111 => InstructionType::J,
        0b0001111 => InstructionType::FENCE,
        _ => panic!("unsupported instruction type"),
    };

    let rd = (insn >> 7) & mask32(5);
    let rs1 = (insn >> 15) & mask32(5);
    let rs2 = (insn >> 20) & mask32(5);
    let funct3 = (insn >> 12) & mask32(3);
    let funct7 = (insn >> 25) & mask32(7);

    // TODO decode the immediate
    let imm = 0;

    Instruction {
        opcode: decode_opcode(opcode_value, insn_type, funct3, funct7, imm),
        rd: rd as usize,
        rs1: rs1 as usize,
        rs2: rs2 as usize,
        imm,
    }
}

fn decode_opcode(
    opcode_value: u32,
    insn_type: InstructionType,
    funct3: u32,
    funct7: u32,
    imm: u64,
) -> Opcode {
    match insn_type {
        InstructionType::R => decode_r_insn(funct3, funct7),
        InstructionType::I => decode_i_insn(opcode_value, funct3, imm),
        InstructionType::S => decode_s_insn(funct3),
        InstructionType::B => decode_b_insn(funct3),
        InstructionType::U => decode_u_opcode(opcode_value),
        InstructionType::J => Opcode::Jal,
        InstructionType::FENCE => Opcode::Fence,
    }
}

fn decode_r_insn(funct3: u32, funct7: u32) -> Opcode {
    todo!()
}

fn decode_i_insn(opcode_value: u32, funct3: u32, imm: u64) -> Opcode {
    todo!()
}

fn decode_s_insn(funct3: u32) -> Opcode {
    todo!()
}

fn decode_b_insn(funct3: u32) -> Opcode {
    todo!()
}

fn decode_u_opcode(opcode_value: u32) -> Opcode {
    todo!()
}
