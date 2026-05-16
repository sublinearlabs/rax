/// RISC-V integer register identifiers in canonical index order (`x0..x31`).
///
/// The discriminant value matches the architectural register index.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum RiscvRegister {
    Zero = 0,
    Ra = 1,
    Sp = 2,
    Gp = 3,
    Tp = 4,
    T0 = 5,
    T1 = 6,
    T2 = 7,
    S0 = 8,
    S1 = 9,
    A0 = 10,
    A1 = 11,
    A2 = 12,
    A3 = 13,
    A4 = 14,
    A5 = 15,
    A6 = 16,
    A7 = 17,
    S2 = 18,
    S3 = 19,
    S4 = 20,
    S5 = 21,
    S6 = 22,
    S7 = 23,
    S8 = 24,
    S9 = 25,
    S10 = 26,
    S11 = 27,
    T3 = 28,
    T4 = 29,
    T5 = 30,
    T6 = 31,
}

impl RiscvRegister {
    /// Converts a 0-based index to a `RiscvRegister` enum variant.
    ///
    /// Intended for table/index-driven construction paths where registers are
    /// addressed by architectural number.
    ///
    /// Returns `None` if the index is out of bounds (i.e., >= 32).
    pub(crate) fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Zero),
            1 => Some(Self::Ra),
            2 => Some(Self::Sp),
            3 => Some(Self::Gp),
            4 => Some(Self::Tp),
            5 => Some(Self::T0),
            6 => Some(Self::T1),
            7 => Some(Self::T2),
            8 => Some(Self::S0),
            9 => Some(Self::S1),
            10 => Some(Self::A0),
            11 => Some(Self::A1),
            12 => Some(Self::A2),
            13 => Some(Self::A3),
            14 => Some(Self::A4),
            15 => Some(Self::A5),
            16 => Some(Self::A6),
            17 => Some(Self::A7),
            18 => Some(Self::S2),
            19 => Some(Self::S3),
            20 => Some(Self::S4),
            21 => Some(Self::S5),
            22 => Some(Self::S6),
            23 => Some(Self::S7),
            24 => Some(Self::S8),
            25 => Some(Self::S9),
            26 => Some(Self::S10),
            27 => Some(Self::S11),
            28 => Some(Self::T3),
            29 => Some(Self::T4),
            30 => Some(Self::T5),
            31 => Some(Self::T6),
            _ => None,
        }
    }
}

/// x86-64 register class used by lowering and mapping logic.
pub(crate) enum X86Register {
    Gpr(X86Gpr),
    Xmm(X86Xmm),
}

/// x86-64 general-purpose registers (`RAX..R15`) in canonical index order.
///
/// The discriminant value is a stable 0-based index for table lookups.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum X86Gpr {
    Rax = 0,
    Rcx = 1,
    Rdx = 2,
    Rbx = 3,
    Rsp = 4,
    Rbp = 5,
    Rsi = 6,
    Rdi = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
}

impl X86Gpr {
    /// Converts a 0-based index to an `X86Gpr` enum variant.
    ///
    /// Returns `None` if the index is out of bounds (i.e., >= 16).
    pub(crate) fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Rax),
            1 => Some(Self::Rcx),
            2 => Some(Self::Rdx),
            3 => Some(Self::Rbx),
            4 => Some(Self::Rsp),
            5 => Some(Self::Rbp),
            6 => Some(Self::Rsi),
            7 => Some(Self::Rdi),
            8 => Some(Self::R8),
            9 => Some(Self::R9),
            10 => Some(Self::R10),
            11 => Some(Self::R11),
            12 => Some(Self::R12),
            13 => Some(Self::R13),
            14 => Some(Self::R14),
            15 => Some(Self::R15),
            _ => None,
        }
    }

    /// Returns the dynasm dynamic register id for this GPR.
    ///
    /// Use with `Rq(...)` at emission sites.
    pub(crate) fn id(self) -> u8 {
        self as u8
    }
}

/// x86 SIMD XMM registers (`XMM0..XMM15`) in canonical index order.
///
/// The discriminant value is a stable 0-based index for table lookups.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum X86Xmm {
    Xmm0 = 0,
    Xmm1 = 1,
    Xmm2 = 2,
    Xmm3 = 3,
    Xmm4 = 4,
    Xmm5 = 5,
    Xmm6 = 6,
    Xmm7 = 7,
    Xmm8 = 8,
    Xmm9 = 9,
    Xmm10 = 10,
    Xmm11 = 11,
    Xmm12 = 12,
    Xmm13 = 13,
    Xmm14 = 14,
    Xmm15 = 15,
}

impl X86Xmm {
    /// Converts a 0-based index to an `X86Xmm` enum variant.
    ///
    /// Returns `None` if the index is out of bounds (i.e., >= 16).
    pub(crate) fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Xmm0),
            1 => Some(Self::Xmm1),
            2 => Some(Self::Xmm2),
            3 => Some(Self::Xmm3),
            4 => Some(Self::Xmm4),
            5 => Some(Self::Xmm5),
            6 => Some(Self::Xmm6),
            7 => Some(Self::Xmm7),
            8 => Some(Self::Xmm8),
            9 => Some(Self::Xmm9),
            10 => Some(Self::Xmm10),
            11 => Some(Self::Xmm11),
            12 => Some(Self::Xmm12),
            13 => Some(Self::Xmm13),
            14 => Some(Self::Xmm14),
            15 => Some(Self::Xmm15),
            _ => None,
        }
    }

    /// Returns the dynasm dynamic register id for this XMM register.
    ///
    /// Use with `Rx(...)` at emission sites.
    pub(crate) fn id(self) -> u8 {
        self as u8
    }
}
