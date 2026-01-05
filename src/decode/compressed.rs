use crate::decode::{
    Instruction,
    util::{c_funct3, quadrant},
};

fn decode_compressed(insn: u16) -> Instruction {
    let quad = quadrant(insn);
    let funct3 = c_funct3(insn);

    match (quad, funct3) {
        _ => Instruction::Illegal(insn as u32),
    }
}

#[cfg(test)]
mod tests {
    use crate::decode::{I, Instruction, compressed::decode_compressed};

    #[test]
    fn test_decode_compressed() {
        // c.nop (0x0001) expands to addi x0, x0, 0
        let compressed_instruction: u16 = 0x0001;
        let insn = decode_compressed(compressed_instruction);
        assert_eq!(
            insn,
            Instruction::Addi(I {
                rd: 0,
                rs1: 0,
                imm: 0
            })
        );
    }
}
