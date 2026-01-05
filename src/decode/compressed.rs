use crate::decode::Instruction;

fn decode_compressed(insn: u16) -> Instruction {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::decode::{I, Instruction, compressed::decode_compressed};

    #[test]
    fn test_decode_compressed() {
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
