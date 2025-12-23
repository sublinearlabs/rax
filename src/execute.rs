use std::i64;

use crate::decode_old::{Instruction, Opcode};
use crate::trace::{MemOp, Tracer};
use crate::{
    VM, is_snan_f32, is_snan_f64,
    util::{mask, mask32, sext},
};

// TODO consider cleaning up sext logic
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

            // F instructions
            Opcode::FmaddS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let c = self.read_f32(insn.rs3);
                let mut res = a.mul_add(b, c);

                // Canonicalize NaN
                if res.is_nan() && !a.is_nan() && !b.is_nan() && !c.is_nan() {
                    res = f32::from_bits(0x7FC00000);
                }

                self.write_f32(insn.rd, res);
                self.raise_fflags_fma_f32(a, b, c, res);
            }

            Opcode::FmsubS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let c = self.read_f32(insn.rs3);
                let res = a.mul_add(b, -c);
                self.write_f32(insn.rd, res);
                self.raise_fflags_fma_f32(a, b, -c, res);
            }

            Opcode::FnmsubS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let c = self.read_f32(insn.rs3);
                let res = (-a).mul_add(b, c);
                self.write_f32(insn.rd, res);
                self.raise_fflags_fma_f32(-a, b, c, res);
            }

            Opcode::FnmaddS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let c = self.read_f32(insn.rs3);
                let res = (-a).mul_add(b, -c);
                self.write_f32(insn.rd, res);
                self.raise_fflags_fma_f32(-a, b, -c, res);
            }

            Opcode::FaddS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let mut res = a + b;

                // Canonicalize NaN
                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f32::from_bits(0x7FC00000);
                }

                self.write_f32(insn.rd, res);
                self.raise_fflags_f32(a, b, res, '+');
            }

            Opcode::FsubS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let mut res = a - b;

                // Canonicalize NaN: RISC-V requires positive quiet NaN
                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f32::from_bits(0x7FC00000); // Canonical positive qNaN
                }

                self.write_f32(insn.rd, res);
                self.raise_fflags_f32(a, b, res, '-');
            }

            Opcode::FmulS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let mut res = a * b;

                // Canonicalize NaN
                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f32::from_bits(0x7FC00000);
                }

                self.write_f32(insn.rd, res);
                self.raise_fflags_f32(a, b, res, '*');
            }

            Opcode::FdivS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let mut res = a / b;

                // Canonicalize NaN
                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f32::from_bits(0x7FC00000);
                }

                self.write_f32(insn.rd, res);
                self.raise_fflags_f32(a, b, res, '/');
            }

            Opcode::FsqrtS => {
                let a = self.read_f32(insn.rs1);

                if is_snan_f32(a) || (a < 0.0 && !a.is_nan()) {
                    self.fcsr_reg |= 0b10000;
                }

                let mut res = a.sqrt();

                // Canonicalize NaN for sqrt of negative
                if res.is_nan() && !a.is_nan() {
                    res = f32::from_bits(0x7FC00000);
                }

                if !res.is_nan() && a >= 0.0 {
                    let exact = (a as f64).sqrt();
                    if exact != (res as f64) {
                        self.fcsr_reg |= 0b00001;
                    }
                }

                self.write_f32(insn.rd, res);
            }

            Opcode::FsgnjS => {
                let rs1_bits = (self.f_reg[insn.rs1] & 0xFFFFFFFF) as u32;
                let rs2_bits = (self.f_reg[insn.rs2] & 0xFFFFFFFF) as u32;
                let sign = rs2_bits & (1 << 31);
                let val = rs1_bits & mask32(31);
                let result = sign | val;
                self.f_reg[insn.rd] = 0xFFFF_FFFF_0000_0000 | (result as u64);
            }

            Opcode::FsgnjnS => {
                let rs1_bits = (self.f_reg[insn.rs1] & 0xFFFFFFFF) as u32;
                let rs2_bits = (self.f_reg[insn.rs2] & 0xFFFFFFFF) as u32;
                let sign = (rs2_bits ^ (1 << 31)) & (1 << 31);
                let val = rs1_bits & mask32(31);
                let result = sign | val;
                self.f_reg[insn.rd] = 0xFFFF_FFFF_0000_0000 | (result as u64);
            }

            Opcode::FsgnjxS => {
                let rs1_bits = (self.f_reg[insn.rs1] & 0xFFFFFFFF) as u32;
                let rs2_bits = (self.f_reg[insn.rs2] & 0xFFFFFFFF) as u32;
                let sign = (rs1_bits & (1 << 31)) ^ (rs2_bits & (1 << 31));
                let val = rs1_bits & mask32(31);
                let result = sign | val;
                self.f_reg[insn.rd] = 0xFFFF_FFFF_0000_0000 | (result as u64);
            }

            Opcode::FminS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // Set NV flag for signaling NaN
                if is_snan_f32(a) || is_snan_f32(b) {
                    self.fcsr_reg |= 0b10000;
                }

                let res = if a.is_nan() && b.is_nan() {
                    f32::from_bits(0x7FC00000) // Canonical NaN
                } else if a.is_nan() {
                    b
                } else if b.is_nan() {
                    a
                } else if a == 0.0 && b == 0.0 {
                    // -0.0 is less than +0.0
                    if a.to_bits() & 0x80000000 != 0 { a } else { b }
                } else {
                    a.min(b)
                };
                self.write_f32(insn.rd, res);
            }

            Opcode::FmaxS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // Set NV flag for signaling NaN
                if is_snan_f32(a) || is_snan_f32(b) {
                    self.fcsr_reg |= 0b10000;
                }

                let res = if a.is_nan() && b.is_nan() {
                    f32::from_bits(0x7FC00000) // Canonical NaN
                } else if a.is_nan() {
                    b
                } else if b.is_nan() {
                    a
                } else if a == 0.0 && b == 0.0 {
                    // +0.0 is greater than -0.0
                    if a.to_bits() & 0x80000000 == 0 { a } else { b }
                } else {
                    a.max(b)
                };
                self.write_f32(insn.rd, res);
            }

            Opcode::FcvtWS => {
                let val = self.read_f32(insn.rs1);

                let (result, flags): (i32, u32) = if val.is_nan() {
                    (i32::MAX, 0b10000)
                } else if val >= 2147483648.0_f32 {
                    (i32::MAX, 0b10000)
                } else if val < -2147483648.0_f32 {
                    (i32::MIN, 0b10000)
                } else {
                    let int_val = val.trunc() as i32;
                    let inexact = if val != val.trunc() { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = (result as i64) as u64;
            }

            Opcode::FcvtWuS => {
                let val = self.read_f32(insn.rs1);

                let (result, flags): (u32, u32) = if val.is_nan() {
                    (u32::MAX, 0b10000) // NV
                } else if val <= -1.0 {
                    // -1.0 or less cannot be represented as unsigned - invalid
                    (0_u32, 0b10000) // NV
                } else if val < 0.0 {
                    // Between -1.0 (exclusive) and 0.0 - truncates to 0, inexact
                    (0_u32, 0b00001) // NX only
                } else if val >= 4294967296.0_f32 {
                    (u32::MAX, 0b10000) // NV
                } else {
                    let truncated = val.trunc();
                    let int_val = truncated as u32;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = (result as i32) as i64 as u64;
            }

            Opcode::FmvXW => {
                let raw_bits = (self.f_reg[insn.rs1] & 0xFFFFFFFF) as u32;
                let result = sext(raw_bits as u64, 32);

                *self.reg_mut(insn.rd) = result;
            }

            Opcode::FeqS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // FeqS only sets NV for signaling NaN
                if is_snan_f32(a) || is_snan_f32(b) {
                    self.fcsr_reg |= 0b10000;
                }

                *self.reg_mut(insn.rd) = if a.is_nan() || b.is_nan() {
                    0
                } else {
                    (a == b) as u64
                };
            }

            Opcode::FltS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // FltS sets NV for ANY NaN (not just signaling)
                if a.is_nan() || b.is_nan() {
                    self.fcsr_reg |= 0b10000;
                    *self.reg_mut(insn.rd) = 0;
                } else {
                    *self.reg_mut(insn.rd) = (a < b) as u64;
                }
            }

            Opcode::FleS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // FleS sets NV for ANY NaN (not just signaling)
                if a.is_nan() || b.is_nan() {
                    self.fcsr_reg |= 0b10000;
                    *self.reg_mut(insn.rd) = 0;
                } else {
                    *self.reg_mut(insn.rd) = (a <= b) as u64;
                }
            }

            Opcode::FclassS => {
                let val = classify32(self.read_f32(insn.rs1).to_bits());
                *self.reg_mut(insn.rd) = val;
            }

            Opcode::FcvtSW => {
                let a = (self.reg(insn.rs1) as i32) as f32;
                self.write_f32(insn.rd, a);
            }

            Opcode::FcvtSWu => {
                let a = (self.reg(insn.rs1) as u32) as f32;
                self.write_f32(insn.rd, a);
            }

            Opcode::FmvWX => {
                let a = f32::from_bits(self.reg(insn.rs1) as u32);
                self.write_f32(insn.rd, a);
            }

            Opcode::FmaddD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let c = self.read_f64(insn.rs3);
                let res = a.mul_add(b, c);
                self.write_f64(insn.rd, res);
                self.raise_fflags_fma_f64(a, b, c, res);
            }

            Opcode::FmsubD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let c = self.read_f64(insn.rs3);
                let res = a.mul_add(b, -c);
                self.write_f64(insn.rd, res);
                self.raise_fflags_fma_f64(a, b, -c, res);
            }

            Opcode::FnmsubD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let c = self.read_f64(insn.rs3);
                let res = (-a).mul_add(b, c);
                self.write_f64(insn.rd, res);
                self.raise_fflags_fma_f64(-a, b, c, res);
            }

            Opcode::FnmaddD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let c = self.read_f64(insn.rs3);
                let res = (-a).mul_add(b, -c);
                self.write_f64(insn.rd, res);
                self.raise_fflags_fma_f64(-a, b, -c, res);
            }

            Opcode::FaddD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let mut res = a + b;

                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000); // Canonical positive qNaN
                }

                self.write_f64(insn.rd, res);
                self.raise_fflags_f64(a, b, res, '+');
            }

            Opcode::FsubD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let mut res = a - b;

                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000);
                }

                self.write_f64(insn.rd, res);
                self.raise_fflags_f64(a, b, res, '-');
            }

            Opcode::FmulD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let mut res = a * b;

                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000);
                }

                self.write_f64(insn.rd, res);
                self.raise_fflags_f64(a, b, res, '*');
            }

            Opcode::FdivD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let mut res = a / b;

                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000);
                }

                self.write_f64(insn.rd, res);
                self.raise_fflags_f64(a, b, res, '/');
            }

            Opcode::FsqrtD => {
                let a = self.read_f64(insn.rs1);

                if is_snan_f64(a) || (a < 0.0 && !a.is_nan()) {
                    self.fcsr_reg |= 0b10000;
                }

                let mut res = a.sqrt();

                if res.is_nan() && !a.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000);
                }

                self.write_f64(insn.rd, res);
            }

            Opcode::FsgnjD => {
                let sign = self.read_f64(insn.rs2).to_bits() & (1 << 63);
                let val = self.read_f64(insn.rs1).to_bits() & mask(63);
                let res = f64::from_bits(sign | val);
                self.write_f64(insn.rd, res);
            }

            Opcode::FsgnjnD => {
                let sign = (self.read_f64(insn.rs2).to_bits() ^ (1 << 63)) & (1 << 63);
                let val = self.read_f64(insn.rs1).to_bits() & mask(63);
                let res = f64::from_bits(sign | val);
                self.write_f64(insn.rd, res);
            }

            Opcode::FsgnjxD => {
                let sign = (self.read_f64(insn.rs1).to_bits() & (1 << 63))
                    ^ (self.read_f64(insn.rs2).to_bits() & (1 << 63));
                let val = self.read_f64(insn.rs1).to_bits() & mask(63);
                let res = f64::from_bits(sign | val);
                self.write_f64(insn.rd, res);
            }

            Opcode::FminD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if is_snan_f64(a) || is_snan_f64(b) {
                    self.fcsr_reg |= 0b10000;
                }

                let res = if a.is_nan() && b.is_nan() {
                    f64::from_bits(0x7FF8000000000000) // Canonical NaN
                } else if a.is_nan() {
                    b
                } else if b.is_nan() {
                    a
                } else if a == 0.0 && b == 0.0 {
                    if a.to_bits() & 0x8000000000000000 != 0 {
                        a
                    } else {
                        b
                    }
                } else {
                    a.min(b)
                };
                self.write_f64(insn.rd, res);
            }

            Opcode::FmaxD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if is_snan_f64(a) || is_snan_f64(b) {
                    self.fcsr_reg |= 0b10000;
                }

                let res = if a.is_nan() && b.is_nan() {
                    f64::from_bits(0x7FF8000000000000)
                } else if a.is_nan() {
                    b
                } else if b.is_nan() {
                    a
                } else if a == 0.0 && b == 0.0 {
                    if a.to_bits() & 0x8000000000000000 == 0 {
                        a
                    } else {
                        b
                    }
                } else {
                    a.max(b)
                };
                self.write_f64(insn.rd, res);
            }

            Opcode::FcvtSD => {
                let a = self.read_f64(insn.rs1);
                let res = a as f32;

                // Set NX if precision was lost
                if !a.is_nan() && !a.is_infinite() && (res as f64) != a {
                    self.fcsr_reg |= 0b00001;
                }

                // Set NV for sNaN
                if is_snan_f64(a) {
                    self.fcsr_reg |= 0b10000;
                }

                self.write_f32(insn.rd, res);
            }

            Opcode::FcvtDS => {
                let a = self.read_f32(insn.rs1);

                // Set NV for sNaN
                if is_snan_f32(a) {
                    self.fcsr_reg |= 0b10000;
                }

                let res = a as f64;
                self.write_f64(insn.rd, res);
            }

            Opcode::FeqD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if is_snan_f64(a) || is_snan_f64(b) {
                    self.fcsr_reg |= 0b10000;
                }

                *self.reg_mut(insn.rd) = if a.is_nan() || b.is_nan() {
                    0
                } else {
                    (a == b) as u64
                };
            }

            Opcode::FltD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if a.is_nan() || b.is_nan() {
                    self.fcsr_reg |= 0b10000;
                    *self.reg_mut(insn.rd) = 0;
                } else {
                    *self.reg_mut(insn.rd) = (a < b) as u64;
                }
            }

            Opcode::FleD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if a.is_nan() || b.is_nan() {
                    self.fcsr_reg |= 0b10000;
                    *self.reg_mut(insn.rd) = 0;
                } else {
                    *self.reg_mut(insn.rd) = (a <= b) as u64;
                }
            }

            Opcode::FclassD => {
                let val = classify64(self.read_f64(insn.rs1).to_bits());
                *self.reg_mut(insn.rd) = val;
            }

            Opcode::FcvtWD => {
                let val = self.read_f64(insn.rs1);

                let (result, flags): (i32, u32) = if val.is_nan() {
                    (i32::MAX, 0b10000)
                } else if val >= (i32::MAX as f64) + 1.0 {
                    (i32::MAX, 0b10000)
                } else if val < (i32::MIN as f64) {
                    (i32::MIN, 0b10000)
                } else {
                    let truncated = val.trunc();
                    let int_val = val as i32;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = (result as i64) as u64;
            }

            Opcode::FcvtWuD => {
                let val = self.read_f64(insn.rs1);

                let (result, flags): (u32, u32) = if val.is_nan() {
                    (u32::MAX, 0b10000)
                } else if val <= -1.0 {
                    (0_u32, 0b10000) // NV - changed from < to <=
                } else if val < 0.0 {
                    (0_u32, 0b00001) // NX only
                } else if val >= (u32::MAX as f64) + 1.0 {
                    (u32::MAX, 0b10000)
                } else {
                    let truncated = val.trunc();
                    let int_val = truncated as u32;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = (result as i32) as i64 as u64;
            }

            Opcode::FcvtDW => {
                let a = (self.reg(insn.rs1) as i32) as f64;
                self.write_f64(insn.rd, a);
            }

            Opcode::FcvtDWu => {
                let a = (self.reg(insn.rs1) as u32) as f64;
                self.write_f64(insn.rd, a);
            }

            Opcode::Flw => {
                let addr = (self.reg(insn.rs1).wrapping_add(insn.imm)) as usize;
                let data = f32::from_bits(self.mem(addr) as u32);
                self.write_f32(insn.rd, data);
            }

            Opcode::Fsw => {
                let addr = (self.reg(insn.rs1).wrapping_add(insn.imm)) as usize;
                let data = self.read_f32(insn.rs2).to_bits().to_le_bytes();
                self.write_bytes(addr, &data);
            }

            Opcode::Fld => {
                let addr = (self.reg(insn.rs1).wrapping_add(insn.imm)) as usize;
                let val = f64::from_bits(self.mem(addr));
                self.write_f64(insn.rd, val);
            }

            Opcode::Fsd => {
                let data = self.read_f64(insn.rs2).to_le_bytes();
                let addr = (self.reg(insn.rs1).wrapping_add(insn.imm)) as usize;
                self.write_bytes(addr, &data);
            }

            Opcode::FcvtLS => {
                let val = self.read_f32(insn.rs1);

                let (result, flags): (i64, u32) = if val.is_nan() {
                    (i64::MAX, 0b10000)
                } else if val >= (i64::MAX as f32) {
                    (i64::MAX, 0b10000)
                } else if val < (i64::MIN as f32) {
                    (i64::MIN, 0b10000)
                } else {
                    let truncated = val.trunc();
                    let int_val = val as i64;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = result as u64;
            }

            Opcode::FcvtLuS => {
                let val = self.read_f32(insn.rs1);

                let (result, flags): (u64, u32) = if val.is_nan() {
                    (u64::MAX, 0b10000)
                } else if val <= -1.0 {
                    (0_u64, 0b10000) // NV - changed from < to <=
                } else if val < 0.0 {
                    (0_u64, 0b00001) // NX only
                } else if val >= (u64::MAX as f32) {
                    (u64::MAX, 0b10000)
                } else {
                    let truncated = val.trunc();
                    let int_val = truncated as u64;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = result;
            }

            Opcode::FcvtSL => {
                let val = (self.reg(insn.rs1) as i64) as f32;
                self.write_f32(insn.rd, val);
            }

            Opcode::FcvtSLu => {
                let val = self.reg(insn.rs1) as f32;
                self.write_f32(insn.rd, val);
            }

            Opcode::FcvtLD => {
                let val = self.read_f64(insn.rs1);

                let (result, flags): (i64, u32) = if val.is_nan() {
                    (i64::MAX, 0b10000)
                } else if val >= (i64::MAX as f64) {
                    (i64::MAX, 0b10000)
                } else if val < (i64::MIN as f64) {
                    (i64::MIN, 0b10000)
                } else {
                    let truncated = val.trunc();
                    let int_val = val as i64;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = result as u64;
            }

            Opcode::FcvtLuD => {
                let val = self.read_f64(insn.rs1);

                let (result, flags): (u64, u32) = if val.is_nan() {
                    (u64::MAX, 0b10000)
                } else if val <= -1.0 {
                    (0_u64, 0b10000) // NV - changed from < to <=
                } else if val < 0.0 {
                    (0_u64, 0b00001) // NX only
                } else if val >= (u64::MAX as f64) {
                    (u64::MAX, 0b10000)
                } else {
                    let truncated = val.trunc();
                    let int_val = truncated as u64;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = result;
            }

            Opcode::FmvXD => {
                let val = self.read_f64(insn.rs1);
                *self.reg_mut(insn.rd) = val.to_bits();
            }

            Opcode::FcvtDL => {
                let val = (self.reg(insn.rs1) as i64) as f64;
                self.write_f64(insn.rd, val);
            }

            Opcode::FcvtDLu => {
                let val = self.reg(insn.rs1) as f64;
                self.write_f64(insn.rd, val);
            }

            Opcode::FmvDX => {
                let val = f64::from_bits(self.reg(insn.rs1));
                self.write_f64(insn.rd, val);
            }

            // CSR instructions
            Opcode::Csrrw => {
                let csr_addr = (insn.imm as u32) & 0xFFF; // Mask to 12 bits
                let old = self.read_csr(csr_addr) as u64;
                let val = self.reg(insn.rs1) as u32;

                self.set_csr(csr_addr, val);
                if insn.rd != 0 {
                    *self.reg_mut(insn.rd) = old;
                }
            }

            Opcode::Csrrs => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                if insn.rs1 != 0 {
                    let val = self.reg(insn.rs1) as u32;
                    let new_val = old as u32 | val;
                    self.set_csr(csr_addr, new_val);
                }
                if insn.rd != 0 {
                    *self.reg_mut(insn.rd) = old;
                }
            }

            Opcode::Csrrc => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                if insn.rs1 != 0 {
                    let val = self.reg(insn.rs1) as u32;
                    let new_val = old as u32 & !val;
                    self.set_csr(csr_addr, new_val);
                }
                if insn.rd != 0 {
                    *self.reg_mut(insn.rd) = old;
                }
            }

            Opcode::Csrrwi => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                let val = (insn.rs1 as u32) & 0x1F;
                self.set_csr(csr_addr, val);
                if insn.rd != 0 {
                    *self.reg_mut(insn.rd) = old;
                }
            }

            Opcode::Csrrsi => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                let val = (insn.rs1 as u32) & 0x1F;
                if val != 0 {
                    let new_val = old as u32 | val;
                    self.set_csr(csr_addr, new_val);
                }
                if insn.rd != 0 {
                    *self.reg_mut(insn.rd) = old;
                }
            }

            Opcode::Csrrci => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                let val = (insn.rs1 as u32) & 0x1F;
                if val != 0 {
                    let new_val = old as u32 & !val;
                    self.set_csr(csr_addr, new_val);
                }
                if insn.rd != 0 {
                    *self.reg_mut(insn.rd) = old;
                }
            }

            // F instructions
            Opcode::FmaddS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let c = self.read_f32(insn.rs3);
                let mut res = a.mul_add(b, c);

                // Canonicalize NaN
                if res.is_nan() && !a.is_nan() && !b.is_nan() && !c.is_nan() {
                    res = f32::from_bits(0x7FC00000);
                }

                self.write_f32(insn.rd, res);
                self.raise_fflags_fma_f32(a, b, c, res);
            }

            Opcode::FmsubS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let c = self.read_f32(insn.rs3);
                let res = a.mul_add(b, -c);
                self.write_f32(insn.rd, res);
                self.raise_fflags_fma_f32(a, b, -c, res);
            }

            Opcode::FnmsubS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let c = self.read_f32(insn.rs3);
                let res = (-a).mul_add(b, c);
                self.write_f32(insn.rd, res);
                self.raise_fflags_fma_f32(-a, b, c, res);
            }

            Opcode::FnmaddS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let c = self.read_f32(insn.rs3);
                let res = (-a).mul_add(b, -c);
                self.write_f32(insn.rd, res);
                self.raise_fflags_fma_f32(-a, b, -c, res);
            }

            Opcode::FaddS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let mut res = a + b;

                // Canonicalize NaN
                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f32::from_bits(0x7FC00000);
                }

                self.write_f32(insn.rd, res);
                self.raise_fflags_f32(a, b, res, '+');
            }

            Opcode::FsubS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let mut res = a - b;

                // Canonicalize NaN: RISC-V requires positive quiet NaN
                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f32::from_bits(0x7FC00000); // Canonical positive qNaN
                }

                self.write_f32(insn.rd, res);
                self.raise_fflags_f32(a, b, res, '-');
            }

            Opcode::FmulS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let mut res = a * b;

                // Canonicalize NaN
                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f32::from_bits(0x7FC00000);
                }

                self.write_f32(insn.rd, res);
                self.raise_fflags_f32(a, b, res, '*');
            }

            Opcode::FdivS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let mut res = a / b;

                // Canonicalize NaN
                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f32::from_bits(0x7FC00000);
                }

                self.write_f32(insn.rd, res);
                self.raise_fflags_f32(a, b, res, '/');
            }

            Opcode::FsqrtS => {
                let a = self.read_f32(insn.rs1);

                if is_snan_f32(a) || (a < 0.0 && !a.is_nan()) {
                    self.fcsr_reg |= 0b10000;
                }

                let mut res = a.sqrt();

                // Canonicalize NaN for sqrt of negative
                if res.is_nan() && !a.is_nan() {
                    res = f32::from_bits(0x7FC00000);
                }

                if !res.is_nan() && a >= 0.0 {
                    let exact = (a as f64).sqrt();
                    if exact != (res as f64) {
                        self.fcsr_reg |= 0b00001;
                    }
                }

                self.write_f32(insn.rd, res);
            }

            Opcode::FsgnjS => {
                let rs1_bits = (self.f_reg[insn.rs1] & 0xFFFFFFFF) as u32;
                let rs2_bits = (self.f_reg[insn.rs2] & 0xFFFFFFFF) as u32;
                let sign = rs2_bits & (1 << 31);
                let val = rs1_bits & mask32(31);
                let result = sign | val;
                self.f_reg[insn.rd] = 0xFFFF_FFFF_0000_0000 | (result as u64);
            }

            Opcode::FsgnjnS => {
                let rs1_bits = (self.f_reg[insn.rs1] & 0xFFFFFFFF) as u32;
                let rs2_bits = (self.f_reg[insn.rs2] & 0xFFFFFFFF) as u32;
                let sign = (rs2_bits ^ (1 << 31)) & (1 << 31);
                let val = rs1_bits & mask32(31);
                let result = sign | val;
                self.f_reg[insn.rd] = 0xFFFF_FFFF_0000_0000 | (result as u64);
            }

            Opcode::FsgnjxS => {
                let rs1_bits = (self.f_reg[insn.rs1] & 0xFFFFFFFF) as u32;
                let rs2_bits = (self.f_reg[insn.rs2] & 0xFFFFFFFF) as u32;
                let sign = (rs1_bits & (1 << 31)) ^ (rs2_bits & (1 << 31));
                let val = rs1_bits & mask32(31);
                let result = sign | val;
                self.f_reg[insn.rd] = 0xFFFF_FFFF_0000_0000 | (result as u64);
            }

            Opcode::FminS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // Set NV flag for signaling NaN
                if is_snan_f32(a) || is_snan_f32(b) {
                    self.fcsr_reg |= 0b10000;
                }

                let res = if a.is_nan() && b.is_nan() {
                    f32::from_bits(0x7FC00000) // Canonical NaN
                } else if a.is_nan() {
                    b
                } else if b.is_nan() {
                    a
                } else if a == 0.0 && b == 0.0 {
                    // -0.0 is less than +0.0
                    if a.to_bits() & 0x80000000 != 0 { a } else { b }
                } else {
                    a.min(b)
                };
                self.write_f32(insn.rd, res);
            }

            Opcode::FmaxS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // Set NV flag for signaling NaN
                if is_snan_f32(a) || is_snan_f32(b) {
                    self.fcsr_reg |= 0b10000;
                }

                let res = if a.is_nan() && b.is_nan() {
                    f32::from_bits(0x7FC00000) // Canonical NaN
                } else if a.is_nan() {
                    b
                } else if b.is_nan() {
                    a
                } else if a == 0.0 && b == 0.0 {
                    // +0.0 is greater than -0.0
                    if a.to_bits() & 0x80000000 == 0 { a } else { b }
                } else {
                    a.max(b)
                };
                self.write_f32(insn.rd, res);
            }

            Opcode::FcvtWS => {
                let val = self.read_f32(insn.rs1);

                let (result, flags): (i32, u32) = if val.is_nan() {
                    (i32::MAX, 0b10000)
                } else if val >= 2147483648.0_f32 {
                    (i32::MAX, 0b10000)
                } else if val < -2147483648.0_f32 {
                    (i32::MIN, 0b10000)
                } else {
                    let int_val = val.trunc() as i32;
                    let inexact = if val != val.trunc() { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = (result as i64) as u64;
            }

            Opcode::FcvtWuS => {
                let val = self.read_f32(insn.rs1);

                let (result, flags): (u32, u32) = if val.is_nan() {
                    (u32::MAX, 0b10000) // NV
                } else if val <= -1.0 {
                    // -1.0 or less cannot be represented as unsigned - invalid
                    (0_u32, 0b10000) // NV
                } else if val < 0.0 {
                    // Between -1.0 (exclusive) and 0.0 - truncates to 0, inexact
                    (0_u32, 0b00001) // NX only
                } else if val >= 4294967296.0_f32 {
                    (u32::MAX, 0b10000) // NV
                } else {
                    let truncated = val.trunc();
                    let int_val = truncated as u32;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = (result as i32) as i64 as u64;
            }

            Opcode::FmvXW => {
                let raw_bits = (self.f_reg[insn.rs1] & 0xFFFFFFFF) as u32;
                let result = sext(raw_bits as u64, 32);

                *self.reg_mut(insn.rd) = result;
            }

            Opcode::FeqS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // FeqS only sets NV for signaling NaN
                if is_snan_f32(a) || is_snan_f32(b) {
                    self.fcsr_reg |= 0b10000;
                }

                *self.reg_mut(insn.rd) = if a.is_nan() || b.is_nan() {
                    0
                } else {
                    (a == b) as u64
                };
            }

            Opcode::FltS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // FltS sets NV for ANY NaN (not just signaling)
                if a.is_nan() || b.is_nan() {
                    self.fcsr_reg |= 0b10000;
                    *self.reg_mut(insn.rd) = 0;
                } else {
                    *self.reg_mut(insn.rd) = (a < b) as u64;
                }
            }

            Opcode::FleS => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // FleS sets NV for ANY NaN (not just signaling)
                if a.is_nan() || b.is_nan() {
                    self.fcsr_reg |= 0b10000;
                    *self.reg_mut(insn.rd) = 0;
                } else {
                    *self.reg_mut(insn.rd) = (a <= b) as u64;
                }
            }

            Opcode::FclassS => {
                let val = classify32(self.read_f32(insn.rs1).to_bits());
                *self.reg_mut(insn.rd) = val;
            }

            Opcode::FcvtSW => {
                let a = (self.reg(insn.rs1) as i32) as f32;
                self.write_f32(insn.rd, a);
            }

            Opcode::FcvtSWu => {
                let a = (self.reg(insn.rs1) as u32) as f32;
                self.write_f32(insn.rd, a);
            }

            Opcode::FmvWX => {
                let a = f32::from_bits(self.reg(insn.rs1) as u32);
                self.write_f32(insn.rd, a);
            }

            Opcode::FmaddD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let c = self.read_f64(insn.rs3);
                let res = a.mul_add(b, c);
                self.write_f64(insn.rd, res);
                self.raise_fflags_fma_f64(a, b, c, res);
            }

            Opcode::FmsubD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let c = self.read_f64(insn.rs3);
                let res = a.mul_add(b, -c);
                self.write_f64(insn.rd, res);
                self.raise_fflags_fma_f64(a, b, -c, res);
            }

            Opcode::FnmsubD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let c = self.read_f64(insn.rs3);
                let res = (-a).mul_add(b, c);
                self.write_f64(insn.rd, res);
                self.raise_fflags_fma_f64(-a, b, c, res);
            }

            Opcode::FnmaddD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let c = self.read_f64(insn.rs3);
                let res = (-a).mul_add(b, -c);
                self.write_f64(insn.rd, res);
                self.raise_fflags_fma_f64(-a, b, -c, res);
            }

            Opcode::FaddD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let mut res = a + b;

                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000); // Canonical positive qNaN
                }

                self.write_f64(insn.rd, res);
                self.raise_fflags_f64(a, b, res, '+');
            }

            Opcode::FsubD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let mut res = a - b;

                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000);
                }

                self.write_f64(insn.rd, res);
                self.raise_fflags_f64(a, b, res, '-');
            }

            Opcode::FmulD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let mut res = a * b;

                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000);
                }

                self.write_f64(insn.rd, res);
                self.raise_fflags_f64(a, b, res, '*');
            }

            Opcode::FdivD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let mut res = a / b;

                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000);
                }

                self.write_f64(insn.rd, res);
                self.raise_fflags_f64(a, b, res, '/');
            }

            Opcode::FsqrtD => {
                let a = self.read_f64(insn.rs1);

                if is_snan_f64(a) || (a < 0.0 && !a.is_nan()) {
                    self.fcsr_reg |= 0b10000;
                }

                let mut res = a.sqrt();

                if res.is_nan() && !a.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000);
                }

                self.write_f64(insn.rd, res);
            }

            Opcode::FsgnjD => {
                let sign = self.read_f64(insn.rs2).to_bits() & (1 << 63);
                let val = self.read_f64(insn.rs1).to_bits() & mask(63);
                let res = f64::from_bits(sign | val);
                self.write_f64(insn.rd, res);
            }

            Opcode::FsgnjnD => {
                let sign = (self.read_f64(insn.rs2).to_bits() ^ (1 << 63)) & (1 << 63);
                let val = self.read_f64(insn.rs1).to_bits() & mask(63);
                let res = f64::from_bits(sign | val);
                self.write_f64(insn.rd, res);
            }

            Opcode::FsgnjxD => {
                let sign = (self.read_f64(insn.rs1).to_bits() & (1 << 63))
                    ^ (self.read_f64(insn.rs2).to_bits() & (1 << 63));
                let val = self.read_f64(insn.rs1).to_bits() & mask(63);
                let res = f64::from_bits(sign | val);
                self.write_f64(insn.rd, res);
            }

            Opcode::FminD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if is_snan_f64(a) || is_snan_f64(b) {
                    self.fcsr_reg |= 0b10000;
                }

                let res = if a.is_nan() && b.is_nan() {
                    f64::from_bits(0x7FF8000000000000) // Canonical NaN
                } else if a.is_nan() {
                    b
                } else if b.is_nan() {
                    a
                } else if a == 0.0 && b == 0.0 {
                    if a.to_bits() & 0x8000000000000000 != 0 {
                        a
                    } else {
                        b
                    }
                } else {
                    a.min(b)
                };
                self.write_f64(insn.rd, res);
            }

            Opcode::FmaxD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if is_snan_f64(a) || is_snan_f64(b) {
                    self.fcsr_reg |= 0b10000;
                }

                let res = if a.is_nan() && b.is_nan() {
                    f64::from_bits(0x7FF8000000000000)
                } else if a.is_nan() {
                    b
                } else if b.is_nan() {
                    a
                } else if a == 0.0 && b == 0.0 {
                    if a.to_bits() & 0x8000000000000000 == 0 {
                        a
                    } else {
                        b
                    }
                } else {
                    a.max(b)
                };
                self.write_f64(insn.rd, res);
            }

            Opcode::FcvtSD => {
                let a = self.read_f64(insn.rs1);
                let res = a as f32;

                // Set NX if precision was lost
                if !a.is_nan() && !a.is_infinite() && (res as f64) != a {
                    self.fcsr_reg |= 0b00001;
                }

                // Set NV for sNaN
                if is_snan_f64(a) {
                    self.fcsr_reg |= 0b10000;
                }

                self.write_f32(insn.rd, res);
            }

            Opcode::FcvtDS => {
                let a = self.read_f32(insn.rs1);

                // Set NV for sNaN
                if is_snan_f32(a) {
                    self.fcsr_reg |= 0b10000;
                }

                let res = a as f64;
                self.write_f64(insn.rd, res);
            }

            Opcode::FeqD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if is_snan_f64(a) || is_snan_f64(b) {
                    self.fcsr_reg |= 0b10000;
                }

                *self.reg_mut(insn.rd) = if a.is_nan() || b.is_nan() {
                    0
                } else {
                    (a == b) as u64
                };
            }

            Opcode::FltD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if a.is_nan() || b.is_nan() {
                    self.fcsr_reg |= 0b10000;
                    *self.reg_mut(insn.rd) = 0;
                } else {
                    *self.reg_mut(insn.rd) = (a < b) as u64;
                }
            }

            Opcode::FleD => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if a.is_nan() || b.is_nan() {
                    self.fcsr_reg |= 0b10000;
                    *self.reg_mut(insn.rd) = 0;
                } else {
                    *self.reg_mut(insn.rd) = (a <= b) as u64;
                }
            }

            Opcode::FclassD => {
                let val = classify64(self.read_f64(insn.rs1).to_bits());
                *self.reg_mut(insn.rd) = val;
            }

            Opcode::FcvtWD => {
                let val = self.read_f64(insn.rs1);

                let (result, flags): (i32, u32) = if val.is_nan() {
                    (i32::MAX, 0b10000)
                } else if val >= (i32::MAX as f64) + 1.0 {
                    (i32::MAX, 0b10000)
                } else if val < (i32::MIN as f64) {
                    (i32::MIN, 0b10000)
                } else {
                    let truncated = val.trunc();
                    let int_val = val as i32;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = (result as i64) as u64;
            }

            Opcode::FcvtWuD => {
                let val = self.read_f64(insn.rs1);

                let (result, flags): (u32, u32) = if val.is_nan() {
                    (u32::MAX, 0b10000)
                } else if val <= -1.0 {
                    (0_u32, 0b10000) // NV - changed from < to <=
                } else if val < 0.0 {
                    (0_u32, 0b00001) // NX only
                } else if val >= (u32::MAX as f64) + 1.0 {
                    (u32::MAX, 0b10000)
                } else {
                    let truncated = val.trunc();
                    let int_val = truncated as u32;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = (result as i32) as i64 as u64;
            }

            Opcode::FcvtDW => {
                let a = (self.reg(insn.rs1) as i32) as f64;
                self.write_f64(insn.rd, a);
            }

            Opcode::FcvtDWu => {
                let a = (self.reg(insn.rs1) as u32) as f64;
                self.write_f64(insn.rd, a);
            }

            Opcode::Flw => {
                let addr = (self.reg(insn.rs1).wrapping_add(insn.imm)) as usize;
                let data = f32::from_bits(self.mem(addr) as u32);
                self.write_f32(insn.rd, data);
            }

            Opcode::Fsw => {
                let addr = (self.reg(insn.rs1).wrapping_add(insn.imm)) as usize;
                let data = self.read_f32(insn.rs2).to_bits().to_le_bytes();
                self.write_bytes(addr, &data);
            }

            Opcode::Fld => {
                let addr = (self.reg(insn.rs1).wrapping_add(insn.imm)) as usize;
                let val = f64::from_bits(self.mem(addr));
                self.write_f64(insn.rd, val);
            }

            Opcode::Fsd => {
                let data = self.read_f64(insn.rs2).to_le_bytes();
                let addr = (self.reg(insn.rs1).wrapping_add(insn.imm)) as usize;
                self.write_bytes(addr, &data);
            }

            Opcode::FcvtLS => {
                let val = self.read_f32(insn.rs1);

                let (result, flags): (i64, u32) = if val.is_nan() {
                    (i64::MAX, 0b10000)
                } else if val >= (i64::MAX as f32) {
                    (i64::MAX, 0b10000)
                } else if val < (i64::MIN as f32) {
                    (i64::MIN, 0b10000)
                } else {
                    let truncated = val.trunc();
                    let int_val = val as i64;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = result as u64;
            }

            Opcode::FcvtLuS => {
                let val = self.read_f32(insn.rs1);

                let (result, flags): (u64, u32) = if val.is_nan() {
                    (u64::MAX, 0b10000)
                } else if val <= -1.0 {
                    (0_u64, 0b10000) // NV - changed from < to <=
                } else if val < 0.0 {
                    (0_u64, 0b00001) // NX only
                } else if val >= (u64::MAX as f32) {
                    (u64::MAX, 0b10000)
                } else {
                    let truncated = val.trunc();
                    let int_val = truncated as u64;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = result;
            }

            Opcode::FcvtSL => {
                let val = (self.reg(insn.rs1) as i64) as f32;
                self.write_f32(insn.rd, val);
            }

            Opcode::FcvtSLu => {
                let val = self.reg(insn.rs1) as f32;
                self.write_f32(insn.rd, val);
            }

            Opcode::FcvtLD => {
                let val = self.read_f64(insn.rs1);

                let (result, flags): (i64, u32) = if val.is_nan() {
                    (i64::MAX, 0b10000)
                } else if val >= (i64::MAX as f64) {
                    (i64::MAX, 0b10000)
                } else if val < (i64::MIN as f64) {
                    (i64::MIN, 0b10000)
                } else {
                    let truncated = val.trunc();
                    let int_val = val as i64;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = result as u64;
            }

            Opcode::FcvtLuD => {
                let val = self.read_f64(insn.rs1);

                let (result, flags): (u64, u32) = if val.is_nan() {
                    (u64::MAX, 0b10000)
                } else if val <= -1.0 {
                    (0_u64, 0b10000) // NV - changed from < to <=
                } else if val < 0.0 {
                    (0_u64, 0b00001) // NX only
                } else if val >= (u64::MAX as f64) {
                    (u64::MAX, 0b10000)
                } else {
                    let truncated = val.trunc();
                    let int_val = truncated as u64;
                    let inexact = if val != truncated { 0b00001 } else { 0 };
                    (int_val, inexact)
                };

                self.fcsr_reg |= flags;
                *self.reg_mut(insn.rd) = result;
            }

            Opcode::FmvXD => {
                let val = self.read_f64(insn.rs1);
                *self.reg_mut(insn.rd) = val.to_bits();
            }

            Opcode::FcvtDL => {
                let val = (self.reg(insn.rs1) as i64) as f64;
                self.write_f64(insn.rd, val);
            }

            Opcode::FcvtDLu => {
                let val = self.reg(insn.rs1) as f64;
                self.write_f64(insn.rd, val);
            }

            Opcode::FmvDX => {
                let val = f64::from_bits(self.reg(insn.rs1));
                self.write_f64(insn.rd, val);
            }

            // CSR instructions
            Opcode::Csrrw => {
                let csr_addr = (insn.imm as u32) & 0xFFF; // Mask to 12 bits
                let old = self.read_csr(csr_addr) as u64;
                let val = self.reg(insn.rs1) as u32;

                self.set_csr(csr_addr, val);
                if insn.rd != 0 {
                    *self.reg_mut(insn.rd) = old;
                }
            }

            Opcode::Csrrs => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                if insn.rs1 != 0 {
                    let val = self.reg(insn.rs1) as u32;
                    let new_val = old as u32 | val;
                    self.set_csr(csr_addr, new_val);
                }
                if insn.rd != 0 {
                    *self.reg_mut(insn.rd) = old;
                }
            }

            Opcode::Csrrc => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                if insn.rs1 != 0 {
                    let val = self.reg(insn.rs1) as u32;
                    let new_val = old as u32 & !val;
                    self.set_csr(csr_addr, new_val);
                }
                if insn.rd != 0 {
                    *self.reg_mut(insn.rd) = old;
                }
            }

            Opcode::Csrrwi => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                let val = (insn.rs1 as u32) & 0x1F;
                self.set_csr(csr_addr, val);
                if insn.rd != 0 {
                    *self.reg_mut(insn.rd) = old;
                }
            }

            Opcode::Csrrsi => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                let val = (insn.rs1 as u32) & 0x1F;
                if val != 0 {
                    let new_val = old as u32 | val;
                    self.set_csr(csr_addr, new_val);
                }
                if insn.rd != 0 {
                    *self.reg_mut(insn.rd) = old;
                }
            }

            Opcode::Csrrci => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                let val = (insn.rs1 as u32) & 0x1F;
                if val != 0 {
                    let new_val = old as u32 & !val;
                    self.set_csr(csr_addr, new_val);
                }
                if insn.rd != 0 {
                    *self.reg_mut(insn.rd) = old;
                }
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

fn classify32(val: u32) -> u64 {
    let sign = val >> 31;
    let exponent = (val >> 23) & mask32(8);
    let frac = val & mask32(23);

    match (sign, exponent, frac) {
        (1, 0xff, 0) => 1,
        (0, 0xff, 0) => 1 << 7,

        (_, 0xff, frac) => {
            let quiet_bit = (frac >> 22) & 1;
            if quiet_bit == 0 { 1 << 8 } else { 1 << 9 }
        }

        (1, 0, 0) => 1 << 3,
        (0, 0, 0) => 1 << 4,

        (1, 0, _) => 1 << 2,
        (0, 0, _) => 1 << 5,

        (1, _, _) => 1 << 1,
        (0, _, _) => 1 << 6,

        (_, _, _) => 0,
    }
}

fn classify64(val: u64) -> u64 {
    let sign = val >> 63;
    let exponent = (val >> 52) & mask(11);
    let frac = val & mask(52);

    match (sign, exponent, frac) {
        (1, 0x7ff, 0) => 1,
        (0, 0x7ff, 0) => 1 << 7,

        (_, 0x7ff, frac) => {
            let quiet_bit = (frac >> 51) & 1;
            if quiet_bit == 0 { 1 << 8 } else { 1 << 9 }
        }

        (1, 0, 0) => 1 << 3,
        (0, 0, 0) => 1 << 4,

        (1, 0, _) => 1 << 2,
        (0, 0, _) => 1 << 5,

        (1, _, _) => 1 << 1,
        (0, _, _) => 1 << 6,

        (_, _, _) => 0,
    }
}

#[cfg(test)]
mod test {
    use crate::VM;
    use crate::decode_old::{Instruction, Opcode};
    use crate::trace::NoopTracer;

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
