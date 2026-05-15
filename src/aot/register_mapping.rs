use crate::aot::registers::{X86Gpr, X86Xmm};

/// Maps RiscvRegisters to x86 Locations
struct RegisterMapping {
    mapping: [MapTarget; 32],
}

/// Possible mapping targets on the x86 hardware
enum MapTarget {
    /// Concept for a register that is always 0
    ///
    /// Note:
    /// One can avoid materializing this register
    /// on physical hardware, instead handle it at
    /// translation level, this will lead to more
    /// efficient assembly output
    ConstZero,
    Gpr(X86Gpr),
    XmmShared {
        reg: X86Xmm,
        lane: XmmLane,
    },
    XmmExclusive(X86Xmm),
}

/// High or Low 64 bit lanes for a 128 bit Xmm Register
enum XmmLane {
    Low,
    High,
}
