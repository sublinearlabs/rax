use crate::util::{map_range, mask, mask32, sext};
use serde::{Deserialize, Serialize};

// RISCV Opcodes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opcode {
    Add,
    Addw,
    Sub,
    Subw,
    Mul,
    Mulh,
    Mulhsu,
    Mulhu,
    Mulw,
    Div,
    Divu,
    Divw,
    Divuw,
    Rem,
    Remu,
    Remw,
    Remuw,
    Xor,
    Or,
    And,
    Sll,
    Sllw,
    Srl,
    Srlw,
    Sra,
    Sraw,
    Slt,
    Sltu,

    Addi,
    Addiw,
    Xori,
    Ori,
    Andi,
    Slli,
    Slliw,
    Srli,
    Srliw,
    Srai,
    Sraiw,
    Slti,
    Sltiu,

    Lb,
    Lh,
    Lw,
    Lbu,
    Lhu,
    Lwu,
    Ld,

    Sb,
    Sh,
    Sw,
    Sd,

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

    LrW,
    ScW,

    LrD,
    ScD,

    AmoswapW,
    AmoaddW,
    AmoxorW,
    AmoandW,
    AmoorW,
    AmominW,
    AmomaxW,
    AmominuW,
    AmomaxuW,

    AmoswapD,
    AmoaddD,
    AmoxorD,
    AmoandD,
    AmoorD,
    AmominD,
    AmomaxD,
    AmominuD,
    AmomaxuD,

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

// RISCV instruction
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instruction {
    pub opcode: Opcode,
    pub rd: usize,
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u64,
}

impl Instruction {
    pub fn new(opcode: Opcode) -> Self {
        Self {
            opcode,
            rd: 0,
            rs1: 0,
            rs2: 0,
            imm: 0,
        }
    }

    pub fn rd(self, val: usize) -> Self {
        Self { rd: val, ..self }
    }

    pub fn rs1(self, val: usize) -> Self {
        Self { rs1: val, ..self }
    }

    pub fn rs2(self, val: usize) -> Self {
        Self { rs2: val, ..self }
    }

    pub fn imm(self, val: u64) -> Self {
        Self { imm: val, ..self }
    }
}

pub fn decode_insn(insn: u32) -> Instruction {
    let opcode_value = insn & mask32(7);

    let insn_type = match opcode_value {
        0b0110011 | 0b0111011 | 0b0101111 => InstructionType::R,
        0b0010011 | 0b0000011 | 0b1100111 | 0b1110011 | 0b0011011 => InstructionType::I,
        0b0100011 => InstructionType::S,
        0b1100011 => InstructionType::B,
        0b0110111 | 0b0010111 => InstructionType::U,
        0b1101111 => InstructionType::J,
        0b0001111 => InstructionType::FENCE,
        _ => panic!("unsupported instruction type"),
    };

    let rd = (insn >> 7) & mask32(5);
    let rs1 = (insn >> 15) & mask32(5);
    let rs2 = (insn >> 20) & mask32(5);
    let funct3 = (insn >> 12) & mask32(3);
    let funct7 = (insn >> 25) & mask32(7);

    let imm = decode_imm(insn, &insn_type);
    let opcode = decode_opcode(opcode_value, insn_type, funct3, funct7, imm);

    Instruction {
        opcode,
        rd: rd as usize,
        rs1: rs1 as usize,
        rs2: rs2 as usize,
        imm,
    }
}

fn decode_imm(insn: u32, insn_type: &InstructionType) -> u64 {
    let mut imm = 0u32;
    match insn_type {
        InstructionType::R | InstructionType::FENCE => imm as u64,
        InstructionType::I => {
            // inst[31:20] => imm[11:0]
            imm = map_range(insn, imm, 31, 11, 12);
            // highest imm bit = 11 (so len = 12)
            sext(imm as u64, 12)
        }
        InstructionType::S => {
            // inst[11:7] => imm[4:0]
            imm = map_range(insn, imm, 11, 4, 5);
            // inst[31:25] => imm[11:5]
            imm = map_range(insn, imm, 31, 11, 7);
            // highest imm bit = 11 (so len = 12)
            sext(imm as u64, 12)
        }
        InstructionType::B => {
            // inst[7] => imm[11]
            imm = map_range(insn, imm, 7, 11, 1);
            // inst[11:8] => imm[4:1]
            imm = map_range(insn, imm, 11, 4, 4);
            // inst[30:25] => imm[10:5]
            imm = map_range(insn, imm, 30, 10, 6);
            // inst[31] => imm[12]
            imm = map_range(insn, imm, 31, 12, 1);
            // highest imm bit = 12 (so len = 13)
            sext(imm as u64, 13)
        }
        InstructionType::U => {
            // inst[31:12] => imm[31:12]
            imm = map_range(insn, imm, 31, 31, 20);
            // highest imm bit = 31 (so len = 32)
            sext(imm as u64, 32)
        }
        InstructionType::J => {
            // inst[19:12] => imm[19:12]
            imm = map_range(insn, imm, 19, 19, 8);
            // inst[20] => imm[11]
            imm = map_range(insn, imm, 20, 11, 1);
            // inst[30:21] => imm[10:1]
            imm = map_range(insn, imm, 30, 10, 10);
            // inst[31] => imm[20]
            imm = map_range(insn, imm, 31, 20, 1);
            // highest imm bit = 20 (so len = 21)
            sext(imm as u64, 21)
        }
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
        InstructionType::R => decode_r_insn(opcode_value, funct3, funct7),
        InstructionType::I => decode_i_insn(opcode_value, funct3, imm),
        InstructionType::S => decode_s_insn(funct3),
        InstructionType::B => decode_b_insn(funct3),
        InstructionType::U => decode_u_opcode(opcode_value),
        InstructionType::J => Opcode::Jal,
        InstructionType::FENCE => Opcode::Fence,
    }
}

fn decode_r_insn(opcode_value: u32, funct3: u32, funct7: u32) -> Opcode {
    match opcode_value {
        0b0110011 => match funct3 {
            0x0 => match funct7 {
                0x00 => Opcode::Add,
                0x01 => Opcode::Mul,
                0x20 => Opcode::Sub,
                _ => panic!("unknown opcode"),
            },
            0x4 => match funct7 {
                0x0 => Opcode::Xor,
                0x1 => Opcode::Div,
                _ => panic!("unknown opcode"),
            },
            0x6 => match funct7 {
                0x0 => Opcode::Or,
                0x1 => Opcode::Rem,
                _ => panic!("unknown opcode"),
            },
            0x7 => match funct7 {
                0x0 => Opcode::And,
                0x1 => Opcode::Remu,
                _ => panic!("unknown opcode"),
            },
            0x1 => match funct7 {
                0x0 => Opcode::Sll,
                0x1 => Opcode::Mulh,
                _ => panic!("unknown opcode"),
            },
            0x5 => match funct7 {
                0x00 => Opcode::Srl,
                0x01 => Opcode::Divu,
                0x20 => Opcode::Sra,
                _ => panic!("unknown opcode"),
            },
            0x2 => match funct7 {
                0x0 => Opcode::Slt,
                0x1 => Opcode::Mulhsu,
                _ => panic!("unknown opcode"),
            },
            0x3 => match funct7 {
                0x0 => Opcode::Sltu,
                0x1 => Opcode::Mulhu,
                _ => panic!("unknown opcode"),
            },
            _ => panic!("unknown opcode"),
        },
        0b0111011 => match funct3 {
            0x0 => match funct7 {
                0x00 => Opcode::Addw,
                0x01 => Opcode::Mulw,
                0x20 => Opcode::Subw,
                _ => panic!("unknown opcode"),
            },
            0x1 => Opcode::Sllw,
            0x4 => match funct7 {
                0x1 => Opcode::Divw,
                _ => panic!("unknown opcode"),
            },
            0x5 => match funct7 {
                0x00 => Opcode::Srlw,
                0x01 => Opcode::Divuw,
                0x20 => Opcode::Sraw,
                _ => panic!("unknown opcode"),
            },
            0x6 => match funct7 {
                0x1 => Opcode::Remw,
                _ => panic!("unknown opcode"),
            },
            0x7 => match funct7 {
                0x1 => Opcode::Remuw,
                _ => panic!("unknown opcode"),
            },
            _ => panic!("unknown opcode"),
        },
        0b0101111 => match funct3 {
            0x2 => match funct7 >> 2 {
                0x00 => Opcode::AmoaddW,
                0x01 => Opcode::AmoswapW,
                0x02 => Opcode::LrW,
                0x03 => Opcode::ScW,
                0x04 => Opcode::AmoxorW,
                0x08 => Opcode::AmoorW,
                0x0C => Opcode::AmoandW,
                0x10 => Opcode::AmominW,
                0x14 => Opcode::AmomaxW,
                0x18 => Opcode::AmominuW,
                0x1C => Opcode::AmomaxuW,
                _ => panic!("unknown opcode detected: {}", funct7 >> 2),
            },
            0x3 => match funct7 >> 2 {
                0x00 => Opcode::AmoaddD,
                0x01 => Opcode::AmoswapD,
                0x02 => Opcode::LrD,
                0x03 => Opcode::ScD,
                0x04 => Opcode::AmoxorD,
                0x08 => Opcode::AmoorD,
                0x0C => Opcode::AmoandD,
                0x10 => Opcode::AmominD,
                0x14 => Opcode::AmomaxD,
                0x18 => Opcode::AmominuD,
                0x1C => Opcode::AmomaxuD,
                _ => panic!("unknown opcode"),
            },
            _ => panic!("unknown opcode"),
        },
        _ => panic!("unknown opcode"),
    }
}

fn decode_i_insn(opcode_value: u32, funct3: u32, imm: u64) -> Opcode {
    match opcode_value {
        0b0010011 => match funct3 {
            0x0 => Opcode::Addi,
            0x4 => Opcode::Xori,
            0x6 => Opcode::Ori,
            0x7 => Opcode::Andi,
            0x1 => Opcode::Slli,
            0x5 => match (imm >> 5) & mask(7) {
                0x00 => Opcode::Srli,
                0x20 => Opcode::Srai,
                _ => panic!("unknown opcode"),
            },
            0x2 => Opcode::Slti,
            0x3 => Opcode::Sltiu,
            _ => panic!("unknown opcode"),
        },
        0b000011 => match funct3 {
            0x0 => Opcode::Lb,
            0x1 => Opcode::Lh,
            0x2 => Opcode::Lw,
            0x4 => Opcode::Lbu,
            0x5 => Opcode::Lhu,
            0x6 => Opcode::Lwu,
            0x3 => Opcode::Ld,
            _ => panic!("unknown opcode"),
        },
        0b1100111 => Opcode::Jalr,
        0b1110011 => match imm {
            0x0 => Opcode::Ecall,
            0x1 => Opcode::Ebreak,
            _ => Opcode::Eother,
        },
        0b0011011 => match funct3 {
            0x0 => Opcode::Addiw,
            0x1 => Opcode::Slliw,
            0x5 => match (imm >> 5) & mask(7) {
                0x00 => Opcode::Srliw,
                0x20 => Opcode::Sraiw,
                _ => panic!("unknown opcode"),
            },
            _ => panic!("unknown opcode"),
        },
        _ => panic!("unknown opcode"),
    }
}

fn decode_s_insn(funct3: u32) -> Opcode {
    match funct3 {
        0x0 => Opcode::Sb,
        0x1 => Opcode::Sh,
        0x2 => Opcode::Sw,
        0x3 => Opcode::Sd,
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

#[cfg(test)]
mod tests {
    use crate::decode_old::decode_insn;

    #[test]
    fn test_immediate_decoding() {
        // addi x10 x11 12 (I Type)
        assert_eq!(decode_insn(0x00C58513).imm, 12);
        // sw x8, 6(x4) (S Type)
        assert_eq!(decode_insn(0x00822323).imm, 6);
        // sw x8, -6(x4) (S Type)
        assert_eq!(decode_insn(0xfe822d23).imm, -6_i32 as u64);
        // beq x5, x6, 20 (B Type)
        assert_eq!(decode_insn(0x00628a63).imm, 20);
        // lui x5, 164 (U Type)
        assert_eq!(decode_insn(0x000a42b7).imm >> 12, 164);
        // jal x5, 44 (J Type)
        assert_eq!(decode_insn(0x02c002ef).imm, 44);
    }
}
