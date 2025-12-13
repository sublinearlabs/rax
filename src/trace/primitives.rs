use serde::{Deserialize, Serialize};

/// Memory operation type for RV64IMAC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize,Default)]
pub enum MemOp {
    /// No memory operation this cycle.
    #[default]
    None,
    /// Load byte (LB/LBU).
    LoadByte { addr: u64, value: u8, signed: bool },
    /// Load halfword (LH/LHU).
    LoadHalf { addr: u64, value: u16, signed: bool },
    /// Load word (LW/LWU).
    LoadWord { addr: u64, value: u32, signed: bool },
    /// Load doubleword (LD).
    LoadDouble { addr: u64, value: u64 },
    /// Store byte (SB).
    StoreByte { addr: u64, value: u8 },
    /// Store halfword (SH).
    StoreHalf { addr: u64, value: u16 },
    /// Store word (SW).
    StoreWord { addr: u64, value: u32 },
    /// Store doubleword (SD).
    StoreDouble { addr: u64, value: u64 },
    /// Load-reserved word (LR.W).
    LoadReservedWord { addr: u64, value: u32 },
    /// Load-reserved doubleword (LR.D).
    LoadReservedDouble { addr: u64, value: u64 },
    /// Store-conditional word (SC.W).
    StoreConditionalWord {
        addr: u64,
        value: u32,
        success: bool,
    },
    /// Store-conditional doubleword (SC.D).
    StoreConditionalDouble {
        addr: u64,
        value: u64,
        success: bool,
    },
    /// Atomic memory operation word (AMO*.W).
    AtomicWord {
        addr: u64,
        read_value: u32,
        write_value: u32,
    },
    /// Atomic memory operation doubleword (AMO*.D).
    AtomicDouble {
        addr: u64,
        read_value: u64,
        write_value: u64,
    },
}


/// Flags indicating instruction class for AIR constraint selection.
/// 
/// @dev it is easy to go this way rather than using a separate enum for each instruction.
/// this would blot the table with is directly propostional to the proof size and proof time. 
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstrFlags {
    // Basic RV64I flags
    /// ALU operation (ADD, SUB, AND, OR, XOR, SLT, etc.)
    pub is_alu: bool,
    /// ALU immediate operation (ADDI, ANDI, etc.)
    pub is_alu_imm: bool,
    /// Word-sized ALU operation (ADDW, SUBW, etc.)
    pub is_alu_word: bool,
    /// Word-sized ALU immediate operation (ADDIW, etc.)
    pub is_alu_imm_word: bool,
    /// Load instruction.
    pub is_load: bool,
    /// Store instruction.
    pub is_store: bool,
    /// Branch instruction.
    pub is_branch: bool,
    /// JAL instruction.
    pub is_jal: bool,
    /// JALR instruction.
    pub is_jalr: bool,
    /// LUI instruction.
    pub is_lui: bool,
    /// AUIPC instruction.
    pub is_auipc: bool,

    // M-extension flags
    /// M-extension multiply (MUL, MULH, MULHU, MULHSU).
    pub is_mul: bool,
    /// M-extension multiply word (MULW).
    pub is_mul_word: bool,
    /// M-extension divide (DIV, DIVU).
    pub is_div: bool,
    /// M-extension divide word (DIVW, DIVUW).
    pub is_div_word: bool,
    /// M-extension remainder (REM, REMU).
    pub is_rem: bool,
    /// M-extension remainder word (REMW, REMUW).
    pub is_rem_word: bool,

    // A-extension flags
    /// Load-reserved instruction (LR.W, LR.D).
    pub is_lr: bool,
    /// Store-conditional instruction (SC.W, SC.D).
    pub is_sc: bool,
    /// Atomic memory operation (AMO*).
    pub is_amo: bool,

    // System flags
    /// ECALL instruction.
    pub is_ecall: bool,
    /// EBREAK instruction.
    pub is_ebreak: bool,
    /// FENCE instruction. (Might hav eto to take this out because, VM has no intension to run in a multi-core fashion)
    pub is_fence: bool,
}