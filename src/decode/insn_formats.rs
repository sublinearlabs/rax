/// R-tyoe register-register format
///
/// Semantics `rd <- f(rs1, rs2)`
pub(crate) struct R {
    pub(crate) rd: u8,
    pub(crate) rs1: u8,
    pub(crate) rs2: u8,
}

/// I-type register-immediate format
///
/// `imm` is a sign-extended immediate
pub(crate) struct I {
    pub(crate) rd: u8,
    pub(crate) rs1: u8,
    pub(crate) imm: i32,
}

/// Shift immediate sub-format (I type shifts)
///
/// `shamt` is the shift amount:
/// - RV32: 5 bits
/// - RV64: 6 bits
pub(crate) struct Sh {
    pub(crate) rd: u8,
    pub(crate) rs1: u8,
    pub(crate) shamt: u8,
}

/// S-type store format
///
/// `imm` is a sign-extended byte offset
///
/// Semantics `mem[rs1 + imm] <- rs2`
pub(crate) struct S {
    pub(crate) rs1: u8,
    pub(crate) rs2: u8,
    pub(crate) imm: i32,
}

/// B-type branch format
///
/// The branch target is PC-relative
/// - If condition holds: `pc <- pc + imm`
/// - Else: fall through to next instruction
///
/// `imm` is the sign-extended PC-relative byte offset
pub(crate) struct B {
    pub(crate) rs1: u8,
    pub(crate) rs2: u8,
    pub(crate) imm: i32,
}

/// J-type jump format
///
/// `imm` is the sign-extended PC-relative byte offset
///
/// Semantics:
/// - `rd <- next_pc`
/// - `pc <- pc + imm`
pub(crate) struct J {
    pub(crate) rd: u8,
    pub(crate) imm: i32,
}

/// U-type upper-immediate format
///
/// `imm` represents the sign-extended upper-immediate
///
/// stores the imm already shifted left by 12 bits
pub(crate) struct U {
    pub(crate) rd: u8,
    pub(crate) imm: i32,
}

/// RF format (floating-point)
///
/// Same register fields as R, but includes rounding mode `rm`
pub(crate) struct RF {
    pub(crate) rd: u8,
    pub(crate) rs1: u8,
    pub(crate) rs2: u8,
    pub(crate) rm: u8,
}

/// R4-type (fused multiply-add format)
pub(crate) struct R4 {
    pub(crate) rd: u8,
    pub(crate) rs1: u8,
    pub(crate) rs2: u8,
    pub(crate) rs3: u8,
    pub(crate) rm: u8,
}
