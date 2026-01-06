use crate::{
    decode::{
        I, Instruction, S,
        imm::{imm_ci_signed, imm_ciw_addi4spn, imm_cl_d, imm_cl_w},
        insn,
        util::{c_funct3, quadrant},
    },
    util::mask16,
};

fn decode_compressed(insn: u16) -> Instruction {
    let quad = quadrant(insn);
    let funct3 = c_funct3(insn);

    match (quad, funct3) {
        // quadrant 0 (00)
        (0b00, 0b000) => dec_c_addi4spn(insn),
        (0b00, 0b001) => dec_c_fld(insn),
        (0b00, 0b010) => dec_c_lw(insn),
        (0b00, 0b011) => dec_c_ld(insn),
        (0b00, 0b100) => Instruction::Illegal(insn as u32),
        (0b00, 0b101) => dec_c_fsd(insn),
        (0b00, 0b110) => dec_c_sw(insn),
        (0b00, 0b111) => dec_c_sd(insn),

        // quadrant 1 (01)
        (0b01, 0b000) => dec_c_addi_nop(insn),
        (0b01, 0b001) => dec_c_addiw(insn),
        (0b01, 0b010) => dec_c_li(insn),
        (0b01, 0b011) => dec_c_addi16sp_lui(insn),
        (0b01, 0b100) => dec_c_alu(insn),
        (0b01, 0b101) => dec_c_j(insn),
        (0b01, 0b110) => dec_c_beqz(insn),
        (0b01, 0b111) => dec_c_bnez(insn),

        // quadrant 2 (10)
        _ => Instruction::Illegal(insn as u32),
    }
}

// Quadrant 0

fn dec_c_addi4spn(insn: u16) -> Instruction {
    // rd' insn[4:2]
    // rd = rd' + 8
    let rd = (((insn >> 2) & mask16(3)) + 8) as u8;

    // extract immediate
    // this is the only instruction that implements CIW (format)
    // nzuimm[5:4|9:6|2|3]
    let imm = imm_ciw_addi4spn(insn);

    if imm == 0 {
        return Instruction::Illegal(insn as u32);
    }

    Instruction::Addi(I {
        rd,
        rs1: 2, // sp
        imm,
    })
}

fn dec_c_fld(insn: u16) -> Instruction {
    // rd' insn[4:2]
    // rd = rd' + 8
    let rd = (((insn >> 2) & mask16(3)) + 8) as u8;

    // rs1' insn[9:7]
    // rs1 = rs1' + 8
    let rs1 = (((insn >> 7) & mask16(3)) + 8) as u8;

    let imm = imm_cl_d(insn);

    Instruction::Fld(I { rd, rs1, imm })
}

fn dec_c_lw(insn: u16) -> Instruction {
    // rd' insn[4:2]
    // rd = rd' + 8
    let rd = (((insn >> 2) & mask16(3)) + 8) as u8;

    // rs1' insn[9:7]
    // rs1 = rs1' + 8
    let rs1 = (((insn >> 7) & mask16(3)) + 8) as u8;

    let imm = imm_cl_w(insn);

    Instruction::Lw(I { rd, rs1, imm })
}

fn dec_c_ld(insn: u16) -> Instruction {
    // rd' insn[4:2]
    // rd = rd' + 8
    let rd = (((insn >> 2) & mask16(3)) + 8) as u8;

    // rs1' insn[9:7]
    // rs1 = rs1' + 8
    let rs1 = (((insn >> 7) & mask16(3)) + 8) as u8;

    let imm = imm_cl_d(insn);

    Instruction::Ld(I { rd, rs1, imm })
}

fn dec_c_fsd(insn: u16) -> Instruction {
    // rs1' insn[9:7]
    // rs1 = rs1' + 8
    let rs1 = (((insn >> 7) & mask16(3)) + 8) as u8;

    // rs2' insn[4:2]
    // rs2 = rd' + 8
    let rs2 = (((insn >> 2) & mask16(3)) + 8) as u8;

    let imm = imm_cl_d(insn);

    Instruction::Fsd(S { rs1, rs2, imm })
}

fn dec_c_sw(insn: u16) -> Instruction {
    // rs1' insn[9:7]
    // rs1 = rs1' + 8
    let rs1 = (((insn >> 7) & mask16(3)) + 8) as u8;

    // rs2' insn[4:2]
    // rs2 = rd' + 8
    let rs2 = (((insn >> 2) & mask16(3)) + 8) as u8;

    let imm = imm_cl_w(insn);

    Instruction::Sw(S { rs1, rs2, imm })
}

fn dec_c_sd(insn: u16) -> Instruction {
    // rs1' insn[9:7]
    // rs1 = rs1' + 8
    let rs1 = (((insn >> 7) & mask16(3)) + 8) as u8;

    // rs2' insn[4:2]
    // rs2 = rd' + 8
    let rs2 = (((insn >> 2) & mask16(3)) + 8) as u8;

    let imm = imm_cl_d(insn);

    Instruction::Sd(S { rs1, rs2, imm })
}

// Quadrant 1

fn dec_c_addi_nop(insn: u16) -> Instruction {
    // rd = insn[11:7]
    let rd = ((insn >> 7) & mask16(5)) as u8;
    let imm = imm_ci_signed(insn);

    // if rd == x0 then imm == 0
    match (rd, imm) {
        // C.NOP
        (0, 0) => Instruction::Addi(I {
            rd: 0,
            rs1: 0,
            imm: 0,
        }),
        (0, _) => Instruction::Illegal(insn as u32),
        _ => Instruction::Addi(I { rd, rs1: rd, imm }),
    }
}

fn dec_c_addiw(insn: u16) -> Instruction {
    // rd = insn[11:7]
    let rd = ((insn >> 7) & mask16(5)) as u8;

    if rd == 0 {
        return Instruction::Illegal(insn as u32);
    }

    let imm = imm_ci_signed(insn);

    Instruction::Addiw(I { rd, rs1: rd, imm })
}

fn dec_c_li(insn: u16) -> Instruction {
    // rd = insn[11:7]
    let rd = ((insn >> 7) & mask16(5)) as u8;

    if rd == 0 {
        return Instruction::Illegal(insn as u32);
    }

    let imm = imm_ci_signed(insn);

    Instruction::Addi(I { rd, rs1: 0, imm })
}

fn dec_c_addi16sp_lui(insn: u16) -> Instruction {
    // rd = insn[11:7]
    let rd = ((insn >> 7) & mask16(5)) as u8;

    if rd == 0 {
        return Instruction::Illegal(insn as u32);
    }

    if rd == 2 {
        // decode addi16sp
    }

    // decode lui

    todo!()
}

fn dec_c_alu(insn: u16) -> Instruction {
    todo!()
}

fn dec_c_j(insn: u16) -> Instruction {
    todo!()
}

fn dec_c_beqz(insn: u16) -> Instruction {
    todo!()
}

fn dec_c_bnez(insn: u16) -> Instruction {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::decode::{I, Instruction, S, compressed::decode_compressed};

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

    #[test]
    fn test_c_fld() {
        let compressed_instruction = 0x2000;
        let insn = decode_compressed(compressed_instruction);
        assert_eq!(
            insn,
            Instruction::Fld(I {
                rd: 8,
                rs1: 8,
                imm: 0
            })
        );

        let compressed_instruction = 0x2400;
        let insn = decode_compressed(compressed_instruction);
        assert_eq!(
            insn,
            Instruction::Fld(I {
                rd: 8,
                rs1: 8,
                imm: 8
            })
        );
    }

    #[test]
    fn test_c_lw() {
        let compressed_instruction = 0x4000;
        let insn = decode_compressed(compressed_instruction);
        assert_eq!(
            insn,
            Instruction::Lw(I {
                rd: 8,
                rs1: 8,
                imm: 0
            })
        );

        let compressed_instruction = 0x4040;
        let insn = decode_compressed(compressed_instruction);
        assert_eq!(
            insn,
            Instruction::Lw(I {
                rd: 8,
                rs1: 8,
                imm: 4
            })
        );
    }

    #[test]
    fn test_c_fsd() {
        let ci: u16 = 0xA000;
        let insn = decode_compressed(ci);
        assert_eq!(
            insn,
            Instruction::Fsd(S {
                rs1: 8,
                rs2: 8,
                imm: 0
            })
        );

        let ci: u16 = 0xA040;
        let insn = decode_compressed(ci);
        assert_eq!(
            insn,
            Instruction::Fsd(S {
                rs1: 8,
                rs2: 8,
                imm: 128
            })
        );
    }

    #[test]
    fn test_c_sw() {
        let ci: u16 = 0xC000;
        let insn = decode_compressed(ci);
        assert_eq!(
            insn,
            Instruction::Sw(S {
                rs1: 8,
                rs2: 8,
                imm: 0
            })
        );

        let ci: u16 = 0xC040;
        let insn = decode_compressed(ci);
        assert_eq!(
            insn,
            Instruction::Sw(S {
                rs1: 8,
                rs2: 8,
                imm: 4
            })
        );
    }

    #[test]
    fn test_c_sd() {
        let ci: u16 = 0xE000;
        let insn = decode_compressed(ci);
        assert_eq!(
            insn,
            Instruction::Sd(S {
                rs1: 8,
                rs2: 8,
                imm: 0
            })
        );

        let ci: u16 = 0xE400;
        let insn = decode_compressed(ci);
        assert_eq!(
            insn,
            Instruction::Sd(S {
                rs1: 8,
                rs2: 8,
                imm: 8
            })
        );
    }

    #[test]
    fn test_c_addiw() {
        let ci = 0x2081;
        let insn = decode_compressed(ci);
        assert_eq!(
            insn,
            Instruction::Addiw(I {
                rd: 1,
                rs1: 1,
                imm: 0
            })
        );

        let ci = 0x2085;
        let insn = decode_compressed(ci);
        assert_eq!(
            insn,
            Instruction::Addiw(I {
                rd: 1,
                rs1: 1,
                imm: 1
            })
        );
    }

    #[test]
    fn test_c_li() {
        let ci = 0x4081;
        let insn = decode_compressed(ci);
        assert_eq!(
            insn,
            Instruction::Addi(I {
                rd: 1,
                rs1: 0,
                imm: 0
            })
        );

        let ci = 0x4085;
        let insn = decode_compressed(ci);
        assert_eq!(
            insn,
            Instruction::Addi(I {
                rd: 1,
                rs1: 0,
                imm: 1
            })
        )
    }
}
