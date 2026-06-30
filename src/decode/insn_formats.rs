use serde::Deserialize;

/// R-type register-register format
///
/// Semantics `rd <- f(rs1, rs2)`
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct R {
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
}

/// I-type register-immediate format
///
/// `imm` is a sign-extended immediate
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct I {
    pub rd: u8,
    pub rs1: u8,
    pub imm: i32,
}

/// Shift immediate sub-format (I type shifts)
///
/// `shamt` is the shift amount:
/// - RV32: 5 bits
/// - RV64: 6 bits
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct Sh {
    pub rd: u8,
    pub rs1: u8,
    pub shamt: u8,
}

/// S-type store format
///
/// `imm` is a sign-extended byte offset
///
/// Semantics `mem[rs1 + imm] <- rs2`
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct S {
    pub rs1: u8,
    pub rs2: u8,
    pub imm: i32,
}

/// B-type branch format
///
/// The branch target is PC-relative
/// - If condition holds: `pc <- pc + imm`
/// - Else: fall through to next instruction
///
/// `imm` is the sign-extended PC-relative byte offset
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct B {
    pub rs1: u8,
    pub rs2: u8,
    pub imm: i32,
}

/// J-type jump format
///
/// `imm` is the sign-extended PC-relative byte offset
///
/// Semantics:
/// - `rd <- next_pc`
/// - `pc <- pc + imm`
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct J {
    pub rd: u8,
    pub imm: i32,
}

/// U-type upper-immediate format
///
/// `imm` represents the sign-extended upper-immediate
///
/// stores the imm already shifted left by 12 bits
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct U {
    pub rd: u8,
    pub imm: i32,
}

/// RF format (floating-point)
///
/// Same register fields as R, but includes rounding mode `rm`
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct RF {
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub rm: u8,
}

/// R4-type (fused multiply-add format)
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct R4 {
    pub rd: u8,
    pub rs1: u8,
    pub rs2: u8,
    pub rs3: u8,
    pub rm: u8,
}
