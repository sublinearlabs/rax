use serde::{Deserialize, Serialize};

use crate::{Instruction, Opcode};

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

impl InstrFlags {
    /// Create flags from an opcode.
    pub fn from_opcode(opcode: &Opcode) -> Self {
        let mut flags = Self::default();

        match opcode {
            // Basic ALU R-type
            Opcode::Add
            | Opcode::Sub
            | Opcode::Xor
            | Opcode::Or
            | Opcode::And
            | Opcode::Sll
            | Opcode::Srl
            | Opcode::Sra
            | Opcode::Slt
            | Opcode::Sltu => {
                flags.is_alu = true;
            }

            // ALU R-type word
            Opcode::Addw | Opcode::Subw | Opcode::Sllw | Opcode::Srlw | Opcode::Sraw => {
                flags.is_alu_word = true;
            }

            // ALU I-type
            Opcode::Addi
            | Opcode::Xori
            | Opcode::Ori
            | Opcode::Andi
            | Opcode::Slli
            | Opcode::Srli
            | Opcode::Srai
            | Opcode::Slti
            | Opcode::Sltiu => {
                flags.is_alu_imm = true;
            }

            // ALU I-type word
            Opcode::Addiw | Opcode::Slliw | Opcode::Srliw | Opcode::Sraiw => {
                flags.is_alu_imm_word = true;
            }

            // Loads
            Opcode::Lb
            | Opcode::Lh
            | Opcode::Lw
            | Opcode::Ld
            | Opcode::Lbu
            | Opcode::Lhu
            | Opcode::Lwu => {
                flags.is_load = true;
            }

            // Stores
            Opcode::Sb | Opcode::Sh | Opcode::Sw | Opcode::Sd => {
                flags.is_store = true;
            }

            // Branches
            Opcode::Beq | Opcode::Bne | Opcode::Blt | Opcode::Bge | Opcode::Bltu | Opcode::Bgeu => {
                flags.is_branch = true;
            }

            // Jumps
            Opcode::Jal => flags.is_jal = true,
            Opcode::Jalr => flags.is_jalr = true,

            // Upper immediates
            Opcode::Lui => flags.is_lui = true,
            Opcode::Auipc => flags.is_auipc = true,

            // M-extension multiply
            Opcode::Mul | Opcode::Mulh | Opcode::Mulhsu | Opcode::Mulhu => {
                flags.is_mul = true;
            }
            Opcode::Mulw => flags.is_mul_word = true,

            // M-extension divide
            Opcode::Div | Opcode::Divu => flags.is_div = true,
            Opcode::Divw | Opcode::Divuw => flags.is_div_word = true,

            // M-extension remainder
            Opcode::Rem | Opcode::Remu => flags.is_rem = true,
            Opcode::Remw | Opcode::Remuw => flags.is_rem_word = true,

            // A-extension load-reserved
            Opcode::LrW | Opcode::LrD => flags.is_lr = true,

            // A-extension store-conditional
            Opcode::ScW | Opcode::ScD => flags.is_sc = true,

            // A-extension atomic operations
            Opcode::AmoswapW
            | Opcode::AmoaddW
            | Opcode::AmoxorW
            | Opcode::AmoandW
            | Opcode::AmoorW
            | Opcode::AmominW
            | Opcode::AmomaxW
            | Opcode::AmominuW
            | Opcode::AmomaxuW
            | Opcode::AmoswapD
            | Opcode::AmoaddD
            | Opcode::AmoxorD
            | Opcode::AmoandD
            | Opcode::AmoorD
            | Opcode::AmominD
            | Opcode::AmomaxD
            | Opcode::AmominuD
            | Opcode::AmomaxuD => {
                flags.is_amo = true;
            }

            // System
            Opcode::Ecall => flags.is_ecall = true,
            Opcode::Ebreak => flags.is_ebreak = true,
            Opcode::Fence => flags.is_fence = true,
            Opcode::Eother => {}
        }

        flags
    }
}

/// A single row of the execution trace.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceRow {
    /// Clock cycle / step number.
    pub clk: u64,
    /// Program counter before this instruction.
    pub pc: u64,
    /// Next program counter (after this instruction).
    pub next_pc: u64,
    /// Raw 32-bit instruction encoding.
    pub raw_instr: u32,
    /// Decoded opcode.
    pub opcode: Opcode,
    /// Instruction classification flags.
    pub flags: InstrFlags,
    /// Register values BEFORE this instruction (x0..x31).
    pub regs: [u64; 32],
    /// Source register 1 index.
    pub rs1: u8,
    /// Source register 2 index.
    pub rs2: u8,
    /// Destination register index (0 if no write).
    pub rd: u8,
    /// Immediate value.
    pub imm: u64,
    /// Value of rs1.
    pub rs1_val: u64,
    /// Value of rs2.
    pub rs2_val: u64,
    /// Value written to rd (if any).
    pub rd_val: u64,
    /// Memory operation (if any).
    pub mem_op: MemOp,
    /// For M-extension: low 64 bits of 128-bit intermediate (for MUL verification).
    pub mul_lo: u64,
    /// For M-extension: high 64 bits of 128-bit intermediate.
    pub mul_hi: u64,
    /// For A-extension: reservation set address (for LR/SC verification).
    pub reservation_addr: u64,
    /// Whether the instruction caused a halt.
    pub halted: bool,
}

impl TraceRow {
    /// Create a new trace row with default values.
    pub fn new(clk: u64, pc: u64, regs: [u64; 32]) -> Self {
        Self {
            clk,
            pc,
            next_pc: pc + 4,
            raw_instr: 0x00000013, // NOP (addi x0, x0, 0)
            opcode: Opcode::Addi,
            flags: InstrFlags::default(),
            regs,
            rs1: 0,
            rs2: 0,
            rd: 0,
            imm: 0,
            rs1_val: 0,
            rs2_val: 0,
            rd_val: 0,
            mem_op: MemOp::None,
            mul_lo: 0,
            mul_hi: 0,
            reservation_addr: 0,
            halted: false,
        }
    }

    /// Create a trace row from an instruction.
    pub fn from_instruction(
        clk: u64,
        pc: u64,
        raw_instr: u32,
        instr: &Instruction,
        regs: [u64; 32],
    ) -> Self {
        let flags = InstrFlags::from_opcode(&instr.opcode);
        let rs1_val = if instr.rs1 == 0 { 0 } else { regs[instr.rs1] };
        let rs2_val = if instr.rs2 == 0 { 0 } else { regs[instr.rs2] };

        Self {
            clk,
            pc,
            next_pc: pc + 4,
            raw_instr,
            opcode: instr.opcode,
            flags,
            regs,
            rs1: instr.rs1 as u8,
            rs2: instr.rs2 as u8,
            rd: instr.rd as u8,
            imm: instr.imm,
            rs1_val,
            rs2_val,
            rd_val: 0,
            mem_op: MemOp::None,
            mul_lo: 0,
            mul_hi: 0,
            reservation_addr: 0,
            halted: false,
        }
    }

    /// Set the destination register value.
    pub fn with_rd_val(mut self, val: u64) -> Self {
        self.rd_val = val;
        self
    }

    /// Set the next PC.
    pub fn with_next_pc(mut self, next_pc: u64) -> Self {
        self.next_pc = next_pc;
        self
    }

    /// Set the memory operation.
    pub fn with_mem_op(mut self, mem_op: MemOp) -> Self {
        self.mem_op = mem_op;
        self
    }

    /// Set multiplication intermediate values.
    pub fn with_mul_intermediate(mut self, lo: u64, hi: u64) -> Self {
        self.mul_lo = lo;
        self.mul_hi = hi;
        self
    }

    /// Set reservation address for LR/SC.
    pub fn with_reservation(mut self, addr: u64) -> Self {
        self.reservation_addr = addr;
        self
    }

    /// Mark as halted.
    pub fn with_halt(mut self) -> Self {
        self.halted = true;
        self
    }
}