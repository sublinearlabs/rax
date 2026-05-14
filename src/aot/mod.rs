// I think I have the required ingredients for a mapping
// I have some rules that a mapping must adhere to
// 1. you cannot map two riscv registers to the same x86 register position
// 2. every riscv register must be mapped

// TODO: documentation philosophy
// TODO: document each type

#[derive(Clone, Copy)]
#[repr(u8)]
enum RiscvRegister {
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

enum X86Register {
    Gpr(X86Gpr),
    Xmm(X86Xmm),
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum X86Gpr {
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

#[derive(Clone, Copy)]
#[repr(u8)]
enum X86Xmm {
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

#[derive(Clone, Copy)]
enum XmmLane {
    Low,
    High,
}

enum MapTarget {
    ConstZero,
    Gpr(X86Gpr),
    XmmExclusive(X86Xmm),
    XmmShared { reg: X86Xmm, lane: XmmLane },
}

enum MapError {
    ConstZeroNonZero {
        riscv_idx: usize,
    },
    DupGpr {
        gpr: X86Gpr,
        first_idx: usize,
        second_idx: usize,
    },
    DupXmmLane {
        reg: X86Xmm,
        lane: XmmLane,
        first_idx: usize,
        second_idx: usize,
    },
    XmmExclusiveConflict {
        reg: X86Xmm,
        first_idx: usize,
        second_idx: usize,
    },
}

struct RegisterMap {
    targets: [MapTarget; 32],
}

impl RegisterMap {
    fn new(targets: [MapTarget; 32]) -> Result<Self, MapError> {
        let mut gpr_owner: [Option<usize>; 16] = [None; 16];
        let mut xmm_low_owner: [Option<usize>; 16] = [None; 16];
        let mut xmm_high_owner: [Option<usize>; 16] = [None; 16];

        for (i, target) in targets.iter().enumerate() {
            match target {
                MapTarget::ConstZero => {
                    if i != RiscvRegister::Zero as usize {
                        return Err(MapError::ConstZeroNonZero { riscv_idx: i });
                    }
                }
                MapTarget::Gpr(gpr) => {
                    let idx = *gpr as usize;
                    if let Some(first) = gpr_owner[idx] {
                        return Err(MapError::DupGpr {
                            gpr: *gpr,
                            first_idx: first,
                            second_idx: i,
                        });
                    }
                    gpr_owner[idx] = Some(i);
                }
                MapTarget::XmmExclusive(xmm) => {
                    let idx = *xmm as usize;
                    if let Some(first) = xmm_low_owner[idx].or(xmm_high_owner[idx]) {
                        return Err(MapError::XmmExclusiveConflict {
                            reg: *xmm,
                            first_idx: first,
                            second_idx: i,
                        });
                    }
                    xmm_low_owner[idx] = Some(i);
                    xmm_high_owner[idx] = Some(i);
                }
                MapTarget::XmmShared { reg, lane } => {
                    let idx = *reg as usize;
                    match lane {
                        XmmLane::Low => {
                            if let Some(first) = xmm_low_owner[idx] {
                                return Err(MapError::DupXmmLane {
                                    reg: *reg,
                                    lane: *lane,
                                    first_idx: first,
                                    second_idx: i,
                                });
                            }
                            xmm_low_owner[idx] = Some(i);
                        }
                        XmmLane::High => {
                            if let Some(first) = xmm_high_owner[idx] {
                                return Err(MapError::DupXmmLane {
                                    reg: *reg,
                                    lane: *lane,
                                    first_idx: first,
                                    second_idx: i,
                                });
                            }
                            xmm_high_owner[idx] = Some(i);
                        }
                    }
                }
            }
        }

        Ok(Self { targets })
    }

    fn get(&self, reg: RiscvRegister) -> &MapTarget {
        &self.targets[reg as usize]
    }
}
