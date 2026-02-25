use crate::decode::Instruction;
use crate::decode::imm::imm_i;
use crate::decode::insn_formats::I;
use crate::decode::util::{funct3, funct7, rd, rs1};

pub(crate) fn decode_system(insn: u32) -> Instruction {
    let imm = imm_i(insn);
    let imm12 = (imm as u32) & 0xfff;

    let operand = I {
        rd: rd(insn),
        rs1: rs1(insn),
        imm: imm_i(insn),
    };

    if funct3(insn) == 0x0 && funct7(insn) == 0x09 {
        return Instruction::SfenceVma;
    }

    match (funct3(insn), imm12) {
        (0x0, 0x000) => Instruction::Ecall,
        (0x0, 0x001) => Instruction::Ebreak,
        (0x0, 0x302) => Instruction::Mret,
        (0x0, 0x102) => Instruction::Sret,
        (0x0, 0x002) => Instruction::Uret,
        (0x0, 0x105) => Instruction::Wfi,
        (0x1, _) => Instruction::Csrrw(operand),
        (0x2, _) => Instruction::Csrrs(operand),
        (0x3, _) => Instruction::Csrrc(operand),
        (0x5, _) => Instruction::Csrrwi(operand),
        (0x6, _) => Instruction::Csrrsi(operand),
        (0x7, _) => Instruction::Csrrci(operand),
        _ => Instruction::Illegal(insn),
    }
}
