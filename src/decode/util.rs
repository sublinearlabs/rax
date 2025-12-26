use crate::util::mask32;

/// Extracts the opcode value from a 32 bit insn
#[inline]
pub(crate) fn opcode(insn: u32) -> u8 {
    (insn & mask32(7)) as u8
}

#[inline]
pub(crate) fn funct3(insn: u32) -> u8 {
    ((insn >> 12) & mask32(3)) as u8
}

#[inline]
pub(crate) fn funct6(insn: u32) -> u8 {
    ((insn >> 26) & mask32(6)) as u8
}

#[inline]
pub(crate) fn funct7(insn: u32) -> u8 {
    ((insn >> 25) & mask32(7)) as u8
}

#[inline]
pub(crate) fn rd(insn: u32) -> u8 {
    ((insn >> 7) & mask32(5)) as u8
}

#[inline]
pub(crate) fn rs1(insn: u32) -> u8 {
    ((insn >> 15) & mask32(5)) as u8
}

#[inline]
pub(crate) fn rs2(insn: u32) -> u8 {
    ((insn >> 20) & mask32(5)) as u8
}

#[inline]
pub(crate) fn rs3(insn: u32) -> u8 {
    ((insn >> 27) & mask32(5)) as u8
}
