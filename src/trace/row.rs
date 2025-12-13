use crate::{Instruction, trace::primitives::{InstrFlags, MemOp, TraceRow}};



#[derive(Clone, Debug)]
pub struct TraceRowBuilder {
    row: TraceRow,
}

impl TraceRowBuilder {
    pub fn new(clk: u64, pc: u64, regs: [u64; 32]) -> Self {
        Self {
            row: TraceRow::new(clk, pc, regs),
        }
    }
    
    /// Set the instruction from raw encoding and decoded instruction.
    pub fn instruction(mut self, raw_instr: u32, instr: &Instruction) -> Self {
        self.row.raw_instr = raw_instr;
        self.row.opcode = instr.opcode;
        self.row.flags = InstrFlags::from_opcode(&instr.opcode);
        self.row.rs1 = instr.rs1 as u8;
        self.row.rs2 = instr.rs2 as u8;
        self.row.rd = instr.rd as u8;
        self.row.imm = instr.imm;

        // Capture source register values
        self.row.rs1_val = if instr.rs1 == 0 {
            0
        } else {
            self.row.regs[instr.rs1]
        };
        self.row.rs2_val = if instr.rs2 == 0 {
            0
        } else {
            self.row.regs[instr.rs2]
        };

        self
    }
    
    /// Set destination register and its value.
    pub fn rd_write(mut self, rd: u8, value: u64) -> Self {
        self.row.rd = rd;
        self.row.rd_val = value;
        self
    }
    
    /// Set the next PC.
    pub fn next_pc(mut self, next_pc: u64) -> Self {
        self.row.next_pc = next_pc;
        self
    }
    
    /// Set the memory operation.
    pub fn mem_op(mut self, mem_op: MemOp) -> Self {
        self.row.mem_op = mem_op;
        self
    }

    /// Set multiplication intermediate values for verification.
    pub fn mul_intermediate(mut self, lo: u64, hi: u64) -> Self {
        self.row.mul_lo = lo;
        self.row.mul_hi = hi;
        self
    }
    
    /// Set the reservation address for LR/SC.
    pub fn reservation(mut self, addr: u64) -> Self {
        self.row.reservation_addr = addr;
        self
    }

    /// Mark this instruction as causing a halt.
    pub fn halt(mut self) -> Self {
        self.row.halted = true;
        self
    }

    /// Build the final TraceRow.
    pub fn build(self) -> TraceRow {
        self.row
    }
}