use crate::decode::imm::imm_i;
use crate::decode::insn_formats::I;
use crate::decode::util::{funct3, rd, rs1};
use crate::decode::Instruction;

pub(crate) fn decode_system(insn: u32) -> Instruction {
    let imm = imm_i(insn);

    let operand = I {
        rd: rd(insn),
        rs1: rs1(insn),
        imm: imm_i(insn),
    };

    match (funct3(insn), imm) {
        (0x0, 0x0) => Instruction::Ecall,
        (0x0, 0x1) => Instruction::Ebreak,
        (0x1, _) => Instruction::Csrrw(operand),
        (0x2, _) => Instruction::Csrrs(operand),
        (0x3, _) => Instruction::Csrrc(operand),
        (0x5, _) => Instruction::Csrrwi(operand),
        (0x6, _) => Instruction::Csrrsi(operand),
        (0x7, _) => Instruction::Csrrci(operand),
        _ => Instruction::Illegal(insn),
    }
}
