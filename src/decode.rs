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
    match funct3 {
        0x0 => match funct7 {
            0x00 => Opcode::Add,
            0x20 => Opcode::Sub,
            _ => panic!("unknown opcode"),
        },
        0x4 => Opcode::Xor,
        0x6 => Opcode::Or,
        0x7 => Opcode::And,
        0x1 => Opcode::Sll,
        0x5 => match funct7 {
            0x00 => Opcode::Srl,
            0x20 => Opcode::Sra,
            _ => panic!("unknown opcode"),
        },
        0x2 => Opcode::Slt,
        0x3 => Opcode::Sltu,
        _ => panic!("unknown opcode"),
    }
}

fn decode_i_insn(opcode_value: u32, funct3: u32, imm: u64) -> Opcode {
    todo!()
}

fn decode_s_insn(funct3: u32) -> Opcode {
    match funct3 {
        0x0 => Opcode::Sb,
        0x1 => Opcode::Sh,
        0x2 => Opcode::Sw,
        _ => panic!("unknown opcode"),
    }
}

fn decode_b_insn(funct3: u32) -> Opcode {
    match funct3 {
        0x0 => Opcode::Beq,
        0x1 => Opcode::Bne,
        0x4 => Opcode::Blt,
        0x5 => Opcode::Bge,
        0x6 => Opcode::Bltu,
        0x7 => Opcode::Bgeu,
        _ => panic!("unknown opcode"),
    }
}

fn decode_u_opcode(opcode_value: u32) -> Opcode {
    match opcode_value {
        0b0110111 => Opcode::Lui,
        0b0010111 => Opcode::Auipc,
        _ => panic!("unknwon opcode"),
    }
}
