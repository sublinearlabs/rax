use crate::aot::instruction_context::{PreparedInput, PreparedOutput};

/// Normalized zero-related classification for `(rd, rs1, rs2)`.
///
/// Variants are mutually exclusive and exhaustive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ZeroCase {
    /// Destination is architectural zero (`rd == x0`).
    ///
    /// Output write is semantically elided and lowering should return early.
    RdZero,
    /// Both sources are architectural zero (`rs1 == x0 && rs2 == x0`),
    /// with `rd != x0`.
    Rs1Rs2Zero,
    /// First source is architectural zero and second is non-zero
    /// (`rs1 == x0 && rs2 != x0`), with `rd != x0`.
    Rs1Zero,
    /// Second source is architectural zero and first is non-zero
    /// (`rs2 == x0 && rs1 != x0`), with `rd != x0`.
    Rs2Zero,
    /// No zero-specific simplification applies
    /// (`rd != x0 && rs1 != x0 && rs2 != x0`).
    None,
}

/// Normalized alias/equality classification for `(rd, rs1, rs2)`.
///
/// This classification is intended for non-zero source paths, i.e. when
/// zero classification yields `ZeroCase::None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShadowCase {
    /// All operands name the same architectural register
    /// (`rd == rs1 && rs1 == rs2`).
    AllEqual,
    /// Destination aliases the first source (`rd == rs1`), with `rd != rs2`.
    RdEqRs1,
    /// Destination aliases the second source (`rd == rs2`), with `rd != rs1`.
    RdEqRs2,
    /// Sources are equal but destination is distinct (`rs1 == rs2`),
    /// with `rd != rs1`.
    Rs1EqRs2,
    /// All operands are pairwise distinct.
    AllDistinct,
}

/// Normalized zero-related classification for `(rd, rs1, imm)`.
///
/// Variants are mutually exclusive and exhaustive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UnaryZeroCase {
    /// Destination is architectural zero (`rd == x0`).
    ///
    /// Output write is semantically elided and lowering should return early.
    RdZero,
    /// Source is architectural zero and immediate is zero
    /// (`rs1 == x0 && imm == 0`), with `rd != x0`.
    Rs1ImmZero,
    /// Source is architectural zero and immediate is non-zero
    /// (`rs1 == x0 && imm != 0`), with `rd != x0`.
    Rs1Zero,
    /// Immediate is zero and source is non-zero
    /// (`imm == 0 && rs1 != x0`), with `rd != x0`.
    ImmZero,
    /// No zero-specific simplification applies
    /// (`rd != x0 && rs1 != x0 && imm != 0`).
    None,
}

/// Normalized alias/equality classification for `(rd, rs1)`.
///
/// This classification is intended for non-zero source paths, i.e. when
/// zero classification yields `UnaryZeroCase::None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UnaryShadowCase {
    /// Destination aliases the source (`rd == rs1`).
    RdEqRs1,
    /// Destination and source are distinct.
    Distinct,
}

/// Classifies zero-related simplification cases for prepared operands.
pub(super) fn classify_zero_case(
    rd: &PreparedOutput<'_>,
    rs1: &PreparedInput<'_>,
    rs2: &PreparedInput<'_>,
) -> ZeroCase {
    if rd.is_zero() {
        return ZeroCase::RdZero;
    }
    if rs1.is_zero() && rs2.is_zero() {
        return ZeroCase::Rs1Rs2Zero;
    }
    if rs1.is_zero() {
        return ZeroCase::Rs1Zero;
    }
    if rs2.is_zero() {
        return ZeroCase::Rs2Zero;
    }
    ZeroCase::None
}

/// Classifies alias/equality relationships for prepared non-zero operands.
pub(super) fn classify_shadow_case(
    rd: &PreparedOutput<'_>,
    rs1: &PreparedInput<'_>,
    rs2: &PreparedInput<'_>,
) -> ShadowCase {
    let rd_id = rd.id();
    let rs1_id = rs1.id();
    let rs2_id = rs2.id();

    if rd_id == rs1_id && rs1_id == rs2_id {
        return ShadowCase::AllEqual;
    }
    if rd_id == rs1_id {
        return ShadowCase::RdEqRs1;
    }
    if rd_id == rs2_id {
        return ShadowCase::RdEqRs2;
    }
    if rs1_id == rs2_id {
        return ShadowCase::Rs1EqRs2;
    }
    ShadowCase::AllDistinct
}

/// Classifies zero-related simplification cases for prepared unary operands.
pub(super) fn classify_unary_zero_case(
    rd: &PreparedOutput<'_>,
    rs1: &PreparedInput<'_>,
    imm: i32,
) -> UnaryZeroCase {
    if rd.is_zero() {
        return UnaryZeroCase::RdZero;
    }

    let imm_zero = imm == 0;

    if rs1.is_zero() && imm_zero {
        return UnaryZeroCase::Rs1ImmZero;
    }
    if rs1.is_zero() {
        return UnaryZeroCase::Rs1Zero;
    }
    if imm_zero {
        return UnaryZeroCase::ImmZero;
    }
    UnaryZeroCase::None
}

/// Classifies alias/equality relationships for prepared non-zero unary operands.
pub(super) fn classify_unary_shadow_case(
    rd: &PreparedOutput<'_>,
    rs1: &PreparedInput<'_>,
) -> UnaryShadowCase {
    if rd.id() == rs1.id() {
        return UnaryShadowCase::RdEqRs1;
    }

    UnaryShadowCase::Distinct
}
