use crate::VM;
use crate::decode::{Instruction, Opcode};
use crate::trace::{MemOp, Tracer};
use crate::util::{mask, sext};

impl<T: Tracer> VM<T> {
    pub(crate) fn execute_instruction(&mut self, insn: Instruction) {
        match insn.opcode {
            // Register Opcodes
            Opcode::Add => {
                let result = self.reg(insn.rs1).wrapping_add(self.reg(insn.rs2));
                self.write_rd(insn.rd, result);
            }

            Opcode::Sub => {
                let result = self.reg(insn.rs1).wrapping_sub(self.reg(insn.rs2));
                self.write_rd(insn.rd, result);
            }

            Opcode::Xor => {
                let result = self.reg(insn.rs1) ^ self.reg(insn.rs2);
                self.write_rd(insn.rd, result);
            }

            Opcode::Or => {
                let result = self.reg(insn.rs1) | self.reg(insn.rs2);
                self.write_rd(insn.rd, result);
            }

            Opcode::And => {
                let result = self.reg(insn.rs1) & self.reg(insn.rs2);
                self.write_rd(insn.rd, result);
            }

            Opcode::Sll => {
                let result = self.reg(insn.rs1) << (self.reg(insn.rs2) & mask(6));
                self.write_rd(insn.rd, result);
            }

            Opcode::Srl => {
                let result = self.reg(insn.rs1) >> (self.reg(insn.rs2) & mask(6));
                self.write_rd(insn.rd, result);
            }

            Opcode::Sra => {
                let val = self.reg(insn.rs1) as i64;
                let result = (val >> (self.reg(insn.rs2) & mask(6))) as u64;
                self.write_rd(insn.rd, result);
            }

            Opcode::Slt => {
                let result = if (self.reg(insn.rs1) as i64) < (self.reg(insn.rs2) as i64) {
                    1
                } else {
                    0
                };
                self.write_rd(insn.rd, result);
            }

            Opcode::Sltu => {
                let result = if self.reg(insn.rs1) < self.reg(insn.rs2) {
                    1
                } else {
                    0
                };
                self.write_rd(insn.rd, result);
            }

            // Immediate Opcodes
            Opcode::Addi => {
                let result = self.reg(insn.rs1).wrapping_add(insn.imm);
                self.write_rd(insn.rd, result);
            }

            Opcode::Xori => {
                let result = self.reg(insn.rs1) ^ insn.imm;
                self.write_rd(insn.rd, result);
            }

            Opcode::Ori => {
                let result = self.reg(insn.rs1) | insn.imm;
                self.write_rd(insn.rd, result);
            }

            Opcode::Andi => {
                let result = self.reg(insn.rs1) & insn.imm;
                self.write_rd(insn.rd, result);
            }

            Opcode::Slli => {
                let result = self.reg(insn.rs1) << insn.imm;
                self.write_rd(insn.rd, result);
            }

            Opcode::Srli => {
                let result = self.reg(insn.rs1) >> (insn.imm & mask(6));
                self.write_rd(insn.rd, result);
            }

            Opcode::Srai => {
                let shift = insn.imm & mask(6);
                let val = self.reg(insn.rs1) as i64;
                let result = (val >> shift) as u64;
                self.write_rd(insn.rd, result);
            }

            Opcode::Slti => {
                let result = if (self.reg(insn.rs1) as i64) < (insn.imm as i64) {
                    1
                } else {
                    0
                };
                self.write_rd(insn.rd, result);
            }

            Opcode::Sltiu => {
                let result = if self.reg(insn.rs1) < insn.imm { 1 } else { 0 };
                self.write_rd(insn.rd, result);
            }

            // Load Opcodes
            Opcode::Lb => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                let raw_value = self.mem(addr as usize) & mask(8);
                let result = sext(raw_value, 8);
                self.tracer.record_mem_op(MemOp::LoadByte {
                    addr,
                    value: raw_value as u8,
                    signed: true,
                });
                self.write_rd(insn.rd, result);
            }

            Opcode::Lbu => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                let result = self.mem(addr as usize) & mask(8);
                self.tracer.record_mem_op(MemOp::LoadByte {
                    addr,
                    value: result as u8,
                    signed: false,
                });
                self.write_rd(insn.rd, result);
            }

            Opcode::Lh => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                let raw_value = self.mem(addr as usize) & mask(16);
                let result = sext(raw_value, 16);
                self.tracer.record_mem_op(MemOp::LoadHalf {
                    addr,
                    value: raw_value as u16,
                    signed: true,
                });
                self.write_rd(insn.rd, result);
            }

            Opcode::Lhu => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                let result = self.mem(addr as usize) & mask(16);
                self.tracer.record_mem_op(MemOp::LoadHalf {
                    addr,
                    value: result as u16,
                    signed: false,
                });
                self.write_rd(insn.rd, result);
            }

            Opcode::Lw => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                let raw_value = self.mem(addr as usize) & mask(32);
                let result = sext(raw_value, 32);
                self.tracer.record_mem_op(MemOp::LoadWord {
                    addr,
                    value: raw_value as u32,
                    signed: true,
                });
                self.write_rd(insn.rd, result);
            }

            Opcode::Lwu => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                let result = self.mem(addr as usize) & mask(32);
                self.tracer.record_mem_op(MemOp::LoadWord {
                    addr,
                    value: result as u32,
                    signed: false,
                });
                self.write_rd(insn.rd, result);
            }

            Opcode::Ld => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                let result = self.mem(addr as usize);
                self.tracer.record_mem_op(MemOp::LoadDouble {
                    addr,
                    value: result,
                });
                self.write_rd(insn.rd, result);
            }

            // Store Opcodes
            Opcode::Sb => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                let value = self.reg(insn.rs2) & mask(8);
                *self.mem_mut(addr as usize) = value as u8;
                self.tracer.record_mem_op(MemOp::StoreByte {
                    addr,
                    value: value as u8,
                });
            }

            Opcode::Sh => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                let value = self.reg(insn.rs2) & mask(16);
                for i in 0..2 {
                    *self.mem_mut(addr as usize + i) = ((value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::StoreHalf {
                    addr,
                    value: value as u16,
                });
            }

            Opcode::Sw => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                let value = self.reg(insn.rs2) & mask(32);
                for i in 0..4 {
                    *self.mem_mut(addr as usize + i) = ((value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::StoreWord {
                    addr,
                    value: value as u32,
                });
            }

            Opcode::Sd => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                let value = self.reg(insn.rs2);
                for i in 0..8 {
                    *self.mem_mut(addr as usize + i) = ((value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer
                    .record_mem_op(MemOp::StoreDouble { addr, value });
            }

            // Branch Opcodes
            Opcode::Beq => {
                if self.reg(insn.rs1) == self.reg(insn.rs2) {
                    self.pc = self.pc.wrapping_add(insn.imm);
                    return;
                }
            }

            Opcode::Bne => {
                if self.reg(insn.rs1) != self.reg(insn.rs2) {
                    self.pc = self.pc.wrapping_add(insn.imm);
                    return;
                }
            }

            Opcode::Blt => {
                if (self.reg(insn.rs1) as i64) < (self.reg(insn.rs2) as i64) {
                    self.pc = self.pc.wrapping_add(insn.imm);
                    return;
                }
            }

            Opcode::Bltu => {
                if self.reg(insn.rs1) < self.reg(insn.rs2) {
                    self.pc = self.pc.wrapping_add(insn.imm);
                    return;
                }
            }

            Opcode::Bge => {
                if (self.reg(insn.rs1) as i64) >= (self.reg(insn.rs2) as i64) {
                    self.pc = self.pc.wrapping_add(insn.imm);
                    return;
                }
            }

            Opcode::Bgeu => {
                if self.reg(insn.rs1) >= self.reg(insn.rs2) {
                    self.pc = self.pc.wrapping_add(insn.imm);
                    return;
                };
            }

            // Jump opcodes
            Opcode::Jal => {
                let result = self.pc.wrapping_add(4);
                self.write_rd(insn.rd, result);
                self.pc = self.pc.wrapping_add(insn.imm);
                return;
            }

            Opcode::Jalr => {
                let target = self.reg(insn.rs1).wrapping_add(insn.imm);
                let result = self.pc.wrapping_add(4);
                self.write_rd(insn.rd, result);
                self.pc = target;
                return;
            }

            // Lui and Auipc
            Opcode::Lui => {
                self.write_rd(insn.rd, insn.imm);
            }

            Opcode::Auipc => {
                let result = self.pc.wrapping_add(insn.imm);
                self.write_rd(insn.rd, result);
            }

            // RV64I Instructions
            Opcode::Addiw => {
                let res = self.reg(insn.rs1).wrapping_add(insn.imm) & mask(32);
                let result = sext(res, 32);
                self.write_rd(insn.rd, result);
            }

            Opcode::Slliw => {
                let val = self.reg(insn.rs1) << (insn.imm & mask(5));
                let result = sext(val & mask(32), 32);
                self.write_rd(insn.rd, result);
            }

            Opcode::Srliw => {
                let result = sext((self.reg(insn.rs1) & mask(32)) >> insn.imm, 32);
                self.write_rd(insn.rd, result);
            }

            Opcode::Sraiw => {
                let shift = (insn.imm & mask(5)) as i32;
                let a = (self.reg(insn.rs1) & mask(32)) as i32;
                let result = (a >> shift) as i64 as u64;
                self.write_rd(insn.rd, result);
            }

            Opcode::Addw => {
                let result = sext(
                    self.reg(insn.rs1).wrapping_add(self.reg(insn.rs2)) & mask(32),
                    32,
                );
                self.write_rd(insn.rd, result);
            }

            Opcode::Subw => {
                let a = self.reg(insn.rs1) as i32;
                let b = self.reg(insn.rs2) as i32;
                let result = a.wrapping_sub(b) as i64 as u64;
                self.write_rd(insn.rd, result);
            }

            Opcode::Sllw => {
                let a = self.reg(insn.rs1);
                let shift = self.reg(insn.rs2) & mask(5);
                let result = sext((a << shift) & mask(32), 32);
                self.write_rd(insn.rd, result);
            }

            Opcode::Srlw => {
                let a = self.reg(insn.rs1) & mask(32);
                let shift = self.reg(insn.rs2) & mask(5);
                let result = sext(a >> shift, 32);
                self.write_rd(insn.rd, result);
            }

            Opcode::Sraw => {
                let a = (self.reg(insn.rs1) & mask(32)) as i32;
                let shift = self.reg(insn.rs2) & mask(5);
                let result = (a >> shift) as i64 as u64;
                self.write_rd(insn.rd, result);
            }

            // M Extension - Multiplication
            Opcode::Mul => {
                let a = self.reg(insn.rs1) as i64;
                let b = self.reg(insn.rs2) as i64;
                let full = (a as i128).wrapping_mul(b as i128);
                let result = a.wrapping_mul(b) as u64;
                self.tracer.record_mul(result, (full >> 64) as u64);
                self.write_rd(insn.rd, result);
            }

            Opcode::Mulh => {
                let a = (self.reg(insn.rs1) as i64) as i128;
                let b = (self.reg(insn.rs2) as i64) as i128;
                let full = a.wrapping_mul(b);
                let lo = full as u64;
                let hi = (full >> 64) as u64;
                self.tracer.record_mul(lo, hi);
                self.write_rd(insn.rd, hi);
            }

            Opcode::Mulhsu => {
                let a = (self.reg(insn.rs1) as i64) as i128;
                let b = (self.reg(insn.rs2) as u128) as i128;
                let full = a.wrapping_mul(b);
                let lo = full as u64;
                let hi = (full >> 64) as u64;
                self.tracer.record_mul(lo, hi);
                self.write_rd(insn.rd, hi);
            }

            Opcode::Mulhu => {
                let a = self.reg(insn.rs1) as u128;
                let b = self.reg(insn.rs2) as u128;
                let full = a.wrapping_mul(b);
                let lo = full as u64;
                let hi = (full >> 64) as u64;
                self.tracer.record_mul(lo, hi);
                self.write_rd(insn.rd, hi);
            }

            Opcode::Mulw => {
                let a = self.reg(insn.rs1);
                let b = self.reg(insn.rs2);
                let product = a.wrapping_mul(b);
                let result = (((product & mask(32)) as i32) as i64) as u64;
                self.tracer.record_mul(product & mask(32), 0);
                self.write_rd(insn.rd, result);
            }

            // M Extension - Division
            Opcode::Div => {
                let dividend = self.reg(insn.rs1) as i64;
                let divisor = self.reg(insn.rs2) as i64;
                let result = if divisor == 0 {
                    u64::MAX
                } else if dividend == i64::MIN && divisor == -1 {
                    dividend as u64
                } else {
                    dividend.wrapping_div(divisor) as u64
                };
                self.write_rd(insn.rd, result);
            }

            Opcode::Divu => {
                let dividend = self.reg(insn.rs1);
                let divisor = self.reg(insn.rs2);
                let result = if divisor == 0 {
                    u64::MAX
                } else {
                    dividend.wrapping_div(divisor)
                };
                self.write_rd(insn.rd, result);
            }

            Opcode::Rem => {
                let dividend = self.reg(insn.rs1) as i64;
                let divisor = self.reg(insn.rs2) as i64;
                let result = if divisor == 0 {
                    dividend as u64
                } else if dividend == i64::MIN && divisor == -1 {
                    0
                } else {
                    dividend.wrapping_rem(divisor) as u64
                };
                self.write_rd(insn.rd, result);
            }

            Opcode::Remu => {
                let dividend = self.reg(insn.rs1);
                let divisor = self.reg(insn.rs2);
                let result = if divisor == 0 {
                    dividend
                } else {
                    dividend.wrapping_rem(divisor)
                };
                self.write_rd(insn.rd, result);
            }

            Opcode::Divw => {
                let dividend = (self.reg(insn.rs1) & mask(32)) as i32;
                let divisor = (self.reg(insn.rs2) & mask(32)) as i32;
                let result = if divisor == 0 {
                    u64::MAX
                } else if dividend == i32::MIN && divisor == -1 {
                    (dividend as i64) as u64
                } else {
                    (dividend.wrapping_div(divisor) as i64) as u64
                };
                self.write_rd(insn.rd, result);
            }

            Opcode::Divuw => {
                let dividend = (self.reg(insn.rs1) & mask(32)) as u32;
                let divisor = (self.reg(insn.rs2) & mask(32)) as u32;
                let result = if divisor == 0 {
                    u64::MAX
                } else {
                    sext(dividend.wrapping_div(divisor) as u64, 32)
                };
                self.write_rd(insn.rd, result);
            }

            Opcode::Remw => {
                let dividend = (self.reg(insn.rs1) & mask(32)) as i32;
                let divisor = (self.reg(insn.rs2) & mask(32)) as i32;
                let result = if divisor == 0 {
                    (dividend as i64) as u64
                } else if dividend == i32::MIN && divisor == -1 {
                    0
                } else {
                    (dividend.wrapping_rem(divisor) as i64) as u64
                };
                self.write_rd(insn.rd, result);
            }

            Opcode::Remuw => {
                let dividend = (self.reg(insn.rs1) & mask(32)) as u32;
                let divisor = (self.reg(insn.rs2) & mask(32)) as u32;
                let result = if divisor == 0 {
                    sext(dividend as u64, 32)
                } else {
                    sext(dividend.wrapping_rem(divisor) as u64, 32)
                };
                self.write_rd(insn.rd, result);
            }

            // A Extension - Load Reserved / Store Conditional
            Opcode::LrW => {
                let addr = self.reg(insn.rs1);
                let value = self.mem(addr as usize) & mask(32);
                let result = sext(value, 32);
                self.reservation_set = addr;
                self.tracer.record_reservation(addr);
                self.tracer.record_mem_op(MemOp::LoadReservedWord {
                    addr,
                    value: value as u32,
                });
                self.write_rd(insn.rd, result);
            }

            Opcode::LrD => {
                let addr = self.reg(insn.rs1);
                let value = self.mem(addr as usize);
                self.reservation_set = addr;
                self.tracer.record_reservation(addr);
                self.tracer
                    .record_mem_op(MemOp::LoadReservedDouble { addr, value });
                self.write_rd(insn.rd, value);
            }

            Opcode::ScW => {
                let addr = self.reg(insn.rs1);
                let value = self.reg(insn.rs2) & mask(32);
                let success = addr == self.reservation_set;
                if success {
                    for i in 0..4 {
                        *self.mem_mut(addr as usize + i) = ((value >> (8 * i)) & mask(8)) as u8;
                    }
                }
                let result = if success { 0 } else { 1 };
                self.reservation_set = 0;
                self.tracer.record_mem_op(MemOp::StoreConditionalWord {
                    addr,
                    value: value as u32,
                    success,
                });
                self.write_rd(insn.rd, result);
            }

            Opcode::ScD => {
                let addr = self.reg(insn.rs1);
                let value = self.reg(insn.rs2);
                let success = addr == self.reservation_set;
                if success {
                    for i in 0..8 {
                        *self.mem_mut(addr as usize + i) = ((value >> (8 * i)) & mask(8)) as u8;
                    }
                }
                let result = if success { 0 } else { 1 };
                self.reservation_set = 0;
                self.tracer.record_mem_op(MemOp::StoreConditionalDouble {
                    addr,
                    value,
                    success,
                });
                self.write_rd(insn.rd, result);
            }

            // A Extension - Atomic Memory Operations (Word)
            Opcode::AmoswapW => {
                let addr = self.reg(insn.rs1);
                let read_value = self.mem(addr as usize) & mask(32);
                let write_value = self.reg(insn.rs2) & mask(32);
                for i in 0..4 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.write_rd(insn.rd, sext(read_value, 32));
            }

            Opcode::AmoaddW => {
                let addr = self.reg(insn.rs1);
                let read_value = (self.mem(addr as usize) & mask(32)) as i32;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let write_value = (read_value.wrapping_add(rs2_val) as i64) as u64 & mask(32);
                for i in 0..4 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.write_rd(insn.rd, (read_value as i64) as u64);
            }

            Opcode::AmoxorW => {
                let addr = self.reg(insn.rs1);
                let read_value = (self.mem(addr as usize) & mask(32)) as i32;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let write_value = ((read_value ^ rs2_val) as i64) as u64 & mask(32);
                for i in 0..4 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.write_rd(insn.rd, (read_value as i64) as u64);
            }

            Opcode::AmoandW => {
                let addr = self.reg(insn.rs1);
                let read_value = (self.mem(addr as usize) & mask(32)) as i32;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let write_value = ((read_value & rs2_val) as i64) as u64 & mask(32);
                for i in 0..4 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.write_rd(insn.rd, (read_value as i64) as u64);
            }

            Opcode::AmoorW => {
                let addr = self.reg(insn.rs1);
                let read_value = (self.mem(addr as usize) & mask(32)) as i32;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let write_value = ((read_value | rs2_val) as i64) as u64 & mask(32);
                for i in 0..4 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.write_rd(insn.rd, (read_value as i64) as u64);
            }

            Opcode::AmominW => {
                let addr = self.reg(insn.rs1);
                let read_value = (self.mem(addr as usize) & mask(32)) as i32;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let write_value = (read_value.min(rs2_val) as i64) as u64 & mask(32);
                for i in 0..4 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.write_rd(insn.rd, (read_value as i64) as u64);
            }

            Opcode::AmomaxW => {
                let addr = self.reg(insn.rs1);
                let read_value = (self.mem(addr as usize) & mask(32)) as i32;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let write_value = (read_value.max(rs2_val) as i64) as u64 & mask(32);
                for i in 0..4 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.write_rd(insn.rd, (read_value as i64) as u64);
            }

            Opcode::AmominuW => {
                let addr = self.reg(insn.rs1);
                let read_value = self.mem(addr as usize) & mask(32);
                let rs2_val = self.reg(insn.rs2) & mask(32);
                let write_value = read_value.min(rs2_val);
                for i in 0..4 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.write_rd(insn.rd, sext(read_value, 32));
            }

            Opcode::AmomaxuW => {
                let addr = self.reg(insn.rs1);
                let read_value = self.mem(addr as usize) & mask(32);
                let rs2_val = self.reg(insn.rs2) & mask(32);
                let write_value = read_value.max(rs2_val);
                for i in 0..4 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.write_rd(insn.rd, sext(read_value, 32));
            }

            // A Extension - Atomic Memory Operations (Double)
            Opcode::AmoswapD => {
                let addr = self.reg(insn.rs1);
                let read_value = self.mem(addr as usize);
                let write_value = self.reg(insn.rs2);
                for i in 0..8 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.write_rd(insn.rd, read_value);
            }

            Opcode::AmoaddD => {
                let addr = self.reg(insn.rs1);
                let read_value = self.mem(addr as usize);
                let rs2_val = self.reg(insn.rs2);
                let write_value = read_value.wrapping_add(rs2_val);
                for i in 0..8 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.write_rd(insn.rd, read_value);
            }

            Opcode::AmoxorD => {
                let addr = self.reg(insn.rs1);
                let read_value = self.mem(addr as usize);
                let rs2_val = self.reg(insn.rs2);
                let write_value = read_value ^ rs2_val;
                for i in 0..8 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.write_rd(insn.rd, read_value);
            }

            Opcode::AmoandD => {
                let addr = self.reg(insn.rs1);
                let read_value = self.mem(addr as usize);
                let rs2_val = self.reg(insn.rs2);
                let write_value = read_value & rs2_val;
                for i in 0..8 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.write_rd(insn.rd, read_value);
            }

            Opcode::AmoorD => {
                let addr = self.reg(insn.rs1);
                let read_value = self.mem(addr as usize);
                let rs2_val = self.reg(insn.rs2);
                let write_value = read_value | rs2_val;
                for i in 0..8 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.write_rd(insn.rd, read_value);
            }

            Opcode::AmominD => {
                let addr = self.reg(insn.rs1);
                let read_value = self.mem(addr as usize);
                let rs2_val = self.reg(insn.rs2) as i64;
                let write_value = (read_value as i64).min(rs2_val) as u64;
                for i in 0..8 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.write_rd(insn.rd, read_value);
            }

            Opcode::AmomaxD => {
                let addr = self.reg(insn.rs1);
                let read_value = self.mem(addr as usize);
                let rs2_val = self.reg(insn.rs2) as i64;
                let write_value = (read_value as i64).max(rs2_val) as u64;
                for i in 0..8 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.write_rd(insn.rd, read_value);
            }

            Opcode::AmominuD => {
                let addr = self.reg(insn.rs1);
                let read_value = self.mem(addr as usize);
                let rs2_val = self.reg(insn.rs2);
                let write_value = read_value.min(rs2_val);
                for i in 0..8 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.write_rd(insn.rd, read_value);
            }

            Opcode::AmomaxuD => {
                let addr = self.reg(insn.rs1);
                let read_value = self.mem(addr as usize);
                let rs2_val = self.reg(insn.rs2);
                let write_value = read_value.max(rs2_val);
                for i in 0..8 {
                    *self.mem_mut(addr as usize + i) = ((write_value >> (8 * i)) & mask(8)) as u8;
                }
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.write_rd(insn.rd, read_value);
            }

            // System Opcodes
            Opcode::Ecall => {
                let func = self.reg(17);
                match func {
                    93 => {
                        // halt
                        self.halted = true;
                        self.exit_code = self.reg(10);
                    }
                    _ => {
                        panic!("skipping ecall");
                    }
                }
            }

            // TODO remove the eager check once all opcodes have been implemented
            _ => {}
        }

        self.pc += 4;
    }

    /// Write to destination register with tracing.
    /// This helper ensures all register writes are traced.
    #[inline(always)]
    fn write_rd(&mut self, rd: usize, value: u64) {
        *self.reg_mut(rd) = value;
        self.tracer.record_rd(rd as u8, value);
    }
}

#[cfg(test)]
mod test {
    use crate::trace::NoopTracer;
    use crate::{Instruction, Opcode, VM};

    #[test]
    fn test_add_instruction() {
        let mut vm = VM::<NoopTracer>::init();
        *vm.reg_mut(3) = 12;
        *vm.reg_mut(5) = 32;
        // r8 = r3 + r5
        vm.execute_instruction(Instruction::new(Opcode::Add).rs1(3).rs2(5).rd(8));
        assert_eq!(vm.reg(8), 12 + 32);
    }

    #[test]
    fn test_store_byte() {
        let mut vm = VM::<NoopTracer>::init();
        *vm.reg_mut(3) = 12;
        *vm.reg_mut(2) = 5;
        let insn = Instruction::new(Opcode::Sb).rs1(2).imm(2).rs2(3);
        vm.execute_instruction(insn);
        assert_eq!(vm.mem(7), 12);
    }

    #[test]
    fn test_store_half_word() {
        let mut vm = VM::<NoopTracer>::init();
        *vm.reg_mut(3) = 64008;
        *vm.reg_mut(2) = 5;
        let insn = Instruction::new(Opcode::Sh).rs1(2).imm(2).rs2(3);
        vm.execute_instruction(insn);
        assert_eq!(vm.mem(7), 64008);
        assert_eq!(vm.mem(8), 250);
    }

    #[test]
    fn test_store_word() {
        let mut vm = VM::<NoopTracer>::init();
        *vm.reg_mut(3) = 2299561908;
        *vm.reg_mut(2) = 5;
        let insn = Instruction::new(Opcode::Sw).rs1(2).imm(2).rs2(3);
        vm.execute_instruction(insn);
        assert_eq!(vm.mem(7), 2299561908);
        assert_eq!(vm.mem(8), 8982663);
        assert_eq!(vm.mem(9), 35088);
    }

    #[test]
    fn test_store_double_word() {
        let mut vm = VM::<NoopTracer>::init();
        *vm.reg_mut(3) = 1234567898765432123;
        *vm.reg_mut(2) = 5;
        let insn = Instruction::new(Opcode::Sd).rs1(2).imm(2).rs2(3);
        vm.execute_instruction(insn);
        assert_eq!(vm.mem(7), 1234567898765432123);
        assert_eq!(vm.mem(8), 4822530854552469);
        assert_eq!(vm.mem(9), 18838011150595);
        assert_eq!(vm.mem(10), 73585981057);
        assert_eq!(vm.mem(11), 287445238);
        assert_eq!(vm.mem(12), 1122832);
    }

    #[test]
    fn test_jal_opcode() {
        let mut vm = VM::<NoopTracer>::init();
        vm.pc = 8;
        let insn = Instruction::new(Opcode::Jal).imm(12).rd(3);
        vm.execute_instruction(insn);
        assert_eq!(vm.reg(3), 12);
        assert_eq!(vm.pc, 20);
    }

    #[test]
    fn test_jalr_opcode() {
        let mut vm = VM::<NoopTracer>::init();
        vm.pc = 8;
        *vm.reg_mut(5) = 6;
        let insn = Instruction::new(Opcode::Jalr).rs1(5).imm(9).rd(3);
        vm.execute_instruction(insn);
        assert_eq!(vm.reg(3), 12);
        assert_eq!(vm.pc, 15);
    }
}
