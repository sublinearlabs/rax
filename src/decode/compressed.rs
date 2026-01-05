use crate::decode::{
    Instruction,
    util::{c_funct3, quadrant},
};

fn decode_compressed(insn: u16) -> Instruction {
    let quad = quadrant(insn);
    let funct3 = c_funct3(insn);

    match (quad, funct3) {
        // quadrant 0 (00)
        (0b00, 0b000) => dec_c_addi4spn(insn),
        (0b00, 0b001) => dec_c_fld(insn),
        (0b00, 0b010) => dec_c_lw(insn),
        (0b00, 0b011) => dec_c_flw_ld(insn),
        (0b00, 0b100) => todo!(),
        (0b00, 0b101) => dec_c_fsd(insn),
        (0b00, 0b110) => dec_c_sw(insn),
        (0b00, 0b111) => dec_c_fsw_sd(insn),

        // quadrant 1 (01)
        _ => Instruction::Illegal(insn as u32),
    }
}

fn dec_c_addi4spn(insn: u16) -> Instruction {
    todo!()
}

fn dec_c_fld(insn: u16) -> Instruction {
    todo!()
}

fn dec_c_lw(insn: u16) -> Instruction {
    todo!()
}

fn dec_c_flw_ld(insn: u16) -> Instruction {
    todo!()
}

fn dec_c_fsd(insn: u16) -> Instruction {
    todo!()
}

fn dec_c_sw(insn: u16) -> Instruction {
    todo!()
}

fn dec_c_fsw_sd(insn: u16) -> Instruction {
    todo!()
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
