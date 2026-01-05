use crate::util::{mask16, mask32};

#[inline]
pub(crate) fn imm_i(insn: u32) -> i32 {
    // insn[31:20] => imm[11:0]
    (insn as i32) >> 20
}

#[inline]
pub(crate) fn imm_s(insn: u32) -> i32 {
    // insn[31:25] => imm[11:5]
    let imm11_5 = (insn >> 25) & mask32(7);
    // insn[11:7] => imm[4:0]
    let imm4_0 = (insn >> 7) & mask32(5);
    // place imm parts
    let imm = (imm11_5 << 5) | imm4_0;
    // sign extend 12 bits
    // does this by placing bit at pos 11 as the sign
    // then performing an arithmetic shift (preserves the sign bit)
    ((imm as i32) << 20) >> 20
}

#[inline]
pub(crate) fn imm_b(insn: u32) -> i32 {
    // insn[31] => imm[12]
    let imm12 = (insn >> 31) & mask32(1);
    // insn[7] => imm[11]
    let imm11 = (insn >> 7) & mask32(1);
    // insn[30:25] => imm[10:5]
    let imm10_5 = (insn >> 25) & mask32(6);
    // insn[11:8] => imm[4:1]
    let imm4_1 = (insn >> 8) & mask32(4);

    let imm = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);
    ((imm as i32) << 19) >> 19
}

#[inline]
pub(crate) fn imm_j(insn: u32) -> i32 {
    // insn[31] => imm[20]
    let imm20 = (insn >> 31) & mask32(1);
    // insn[19:12] => imm[19:12]
    let imm19_12 = (insn >> 12) & mask32(8);
    // insn[20] => imm[11]
    let imm11 = (insn >> 20) & mask32(1);
    // insn[30:21] => imm[10:1]
    let imm10_1 = (insn >> 21) & mask32(10);

    let imm = (imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1);
    ((imm as i32) << 11) >> 11
}

#[inline]
pub(crate) fn imm_u(insn: u32) -> i32 {
    // insn[31:12] => imm[31:12]
    // just zero out the last 12 bits
    (insn & !mask32(12)) as i32
}

#[inline]
pub(crate) fn shamt5(insn: u32) -> u8 {
    let imm = (insn >> 20) & mask32(5);
    imm as u8
}

#[inline]
pub(crate) fn shamt6(insn: u32) -> u8 {
    let imm = (insn >> 20) & mask32(6);
    imm as u8
}

// Compressed Immediate Extraction

#[inline]
pub(crate) fn imm_ciw_addi4spn(insn: u16) -> i32 {
    // insn [12 11 | 10 9 8 7 | 6 | 5]
    // imm  [5   4 |  9 8 7 6 | 2 | 3]
    let imm5_4 = ((insn >> 11) & mask16(2)) << 4;
    let imm9_6 = ((insn >> 7) & mask16(4)) << 6;
    let imm2 = ((insn >> 6) & mask16(1)) << 2;
    let imm3 = ((insn >> 5) & mask16(1)) << 3;
    let imm = imm9_6 | imm5_4 | imm3 | imm2;
    imm as i32
}
