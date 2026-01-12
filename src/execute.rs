use std::i64;

use crate::decode::Instruction;
use crate::ecall::handle_ecall;
use crate::trace::{MemOp, Tracer};
use crate::{
    VM, is_snan_f32, is_snan_f64,
    util::{mask, mask32, sext},
};

// TODO consider cleaning up sext logic
impl<T: Tracer> VM<T> {
    pub(crate) fn execute_instruction(&mut self, insn: Instruction, is_compressed: bool) {
        match insn {
            // Register Opcodes
            Instruction::Add(insn) => {
                let result = self.reg(insn.rs1).wrapping_add(self.reg(insn.rs2));
                self.reg_mut(insn.rd, result);
            }

            Instruction::Sub(insn) => {
                let result = self.reg(insn.rs1).wrapping_sub(self.reg(insn.rs2));
                self.reg_mut(insn.rd, result);
            }

            Instruction::Xor(insn) => {
                let result = self.reg(insn.rs1) ^ self.reg(insn.rs2);
                self.reg_mut(insn.rd, result);
            }

            Instruction::Or(insn) => {
                let result = self.reg(insn.rs1) | self.reg(insn.rs2);
                self.reg_mut(insn.rd, result);
            }

            Instruction::And(insn) => {
                let result = self.reg(insn.rs1) & self.reg(insn.rs2);
                self.reg_mut(insn.rd, result);
            }

            Instruction::Sll(insn) => {
                let result = self.reg(insn.rs1) << (self.reg(insn.rs2) & mask(6));
                self.reg_mut(insn.rd, result);
            }

            Instruction::Srl(insn) => {
                let result = self.reg(insn.rs1) >> (self.reg(insn.rs2) & mask(6));
                self.reg_mut(insn.rd, result);
            }

            Instruction::Sra(insn) => {
                let val = self.reg(insn.rs1) as i64;
                let result = (val >> (self.reg(insn.rs2) & mask(6))) as u64;
                self.reg_mut(insn.rd, result);
            }

            Instruction::Slt(insn) => {
                let result = if (self.reg(insn.rs1) as i64) < (self.reg(insn.rs2) as i64) {
                    1
                } else {
                    0
                };
                self.reg_mut(insn.rd, result);
            }

            Instruction::Sltu(insn) => {
                let result = if self.reg(insn.rs1) < self.reg(insn.rs2) {
                    1
                } else {
                    0
                };
                self.reg_mut(insn.rd, result);
            }

            // Immediate Opcodes
            Instruction::Addi(insn) => {
                let result = self.reg(insn.rs1).wrapping_add(insn.imm as u64);
                self.reg_mut(insn.rd, result);
            }

            Instruction::Xori(insn) => {
                let result = self.reg(insn.rs1) ^ insn.imm as u64;
                self.reg_mut(insn.rd, result);
            }

            Instruction::Ori(insn) => {
                let result = self.reg(insn.rs1) | insn.imm as u64;
                self.reg_mut(insn.rd, result);
            }

            Instruction::Andi(insn) => {
                let result = self.reg(insn.rs1) & insn.imm as u64;
                self.reg_mut(insn.rd, result);
            }

            Instruction::Slli(insn) => {
                let result = self.reg(insn.rs1) << insn.shamt;
                self.reg_mut(insn.rd, result);
            }

            Instruction::Srli(insn) => {
                let result = self.reg(insn.rs1) >> insn.shamt;
                self.reg_mut(insn.rd, result);
            }

            Instruction::Srai(insn) => {
                let shift = insn.shamt;
                let val = self.reg(insn.rs1) as i64;
                let result = (val >> shift) as u64;
                self.reg_mut(insn.rd, result);
            }

            Instruction::Slti(insn) => {
                let result = if (self.reg(insn.rs1) as i64) < (insn.imm as i64) {
                    1
                } else {
                    0
                };
                self.reg_mut(insn.rd, result);
            }

            Instruction::Sltiu(insn) => {
                let result = if self.reg(insn.rs1) < insn.imm as u64 {
                    1
                } else {
                    0
                };
                self.reg_mut(insn.rd, result);
            }

            // Load Opcodes
            Instruction::Lb(insn) => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm as u64);
                let raw_value = self.load_u8(addr as usize) as u64;
                let result = sext(raw_value, 8);
                self.tracer.record_mem_op(MemOp::LoadByte {
                    addr,
                    value: raw_value as u8,
                    signed: true,
                });
                self.reg_mut(insn.rd, result);
            }

            Instruction::Lbu(insn) => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm as u64);
                let result = self.load_u8(addr as usize) as u64;
                self.tracer.record_mem_op(MemOp::LoadByte {
                    addr,
                    value: result as u8,
                    signed: false,
                });
                self.reg_mut(insn.rd, result);
            }

            Instruction::Lh(insn) => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm as u64);
                let raw_value = self.load_u16(addr as usize) as u64;
                let result = sext(raw_value, 16);
                self.tracer.record_mem_op(MemOp::LoadHalf {
                    addr,
                    value: raw_value as u16,
                    signed: true,
                });
                self.reg_mut(insn.rd, result);
            }

            Instruction::Lhu(insn) => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm as u64);
                let result = self.load_u16(addr as usize) as u64;
                self.tracer.record_mem_op(MemOp::LoadHalf {
                    addr,
                    value: result as u16,
                    signed: false,
                });
                self.reg_mut(insn.rd, result);
            }

            Instruction::Lw(insn) => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm as u64);
                let raw_value = self.load_u32(addr as usize) as u64;
                let result = sext(raw_value, 32);
                self.tracer.record_mem_op(MemOp::LoadWord {
                    addr,
                    value: raw_value as u32,
                    signed: true,
                });
                self.reg_mut(insn.rd, result);
            }

            Instruction::Lwu(insn) => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm as u64);
                let result = self.load_u32(addr as usize) as u64;
                self.tracer.record_mem_op(MemOp::LoadWord {
                    addr,
                    value: result as u32,
                    signed: false,
                });
                self.reg_mut(insn.rd, result);
            }

            Instruction::Ld(insn) => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm as u64);
                let result = self.load_u64(addr as usize);
                self.tracer.record_mem_op(MemOp::LoadDouble {
                    addr,
                    value: result,
                });
                self.reg_mut(insn.rd, result);
            }

            // Store Opcodes
            Instruction::Sb(insn) => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm as u64);
                // TODO: do we need the mask(8)
                let value = self.reg(insn.rs2) & mask(8);
                self.store_u8(addr as usize, value as u8);
                self.tracer.record_mem_op(MemOp::StoreByte {
                    addr,
                    value: value as u8,
                });
            }

            Instruction::Sh(insn) => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm as u64);
                let value = self.reg(insn.rs2) & mask(16);
                self.store_u16(addr as usize, value as u16);
                self.tracer.record_mem_op(MemOp::StoreHalf {
                    addr,
                    value: value as u16,
                });
            }

            Instruction::Sw(insn) => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm as u64);
                let value = self.reg(insn.rs2) & mask(32);
                self.store_u32(addr as usize, value as u32);
                self.tracer.record_mem_op(MemOp::StoreWord {
                    addr,
                    value: value as u32,
                });
            }

            Instruction::Sd(insn) => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm as u64);
                let value = self.reg(insn.rs2);
                self.store_u64(addr as usize, value);
                self.tracer
                    .record_mem_op(MemOp::StoreDouble { addr, value });
            }

            // Branch Opcodes
            Instruction::Beq(insn) => {
                if self.reg(insn.rs1) == self.reg(insn.rs2) {
                    self.pc = self.pc.wrapping_add(insn.imm as u64);
                    return;
                }
            }

            Instruction::Bne(insn) => {
                if self.reg(insn.rs1) != self.reg(insn.rs2) {
                    self.pc = self.pc.wrapping_add(insn.imm as u64);
                    return;
                }
            }

            Instruction::Blt(insn) => {
                if (self.reg(insn.rs1) as i64) < (self.reg(insn.rs2) as i64) {
                    self.pc = self.pc.wrapping_add(insn.imm as u64);
                    return;
                }
            }

            Instruction::Bltu(insn) => {
                if self.reg(insn.rs1) < self.reg(insn.rs2) {
                    self.pc = self.pc.wrapping_add(insn.imm as u64);
                    return;
                }
            }

            Instruction::Bge(insn) => {
                if (self.reg(insn.rs1) as i64) >= (self.reg(insn.rs2) as i64) {
                    self.pc = self.pc.wrapping_add(insn.imm as u64);
                    return;
                }
            }

            Instruction::Bgeu(insn) => {
                if self.reg(insn.rs1) >= self.reg(insn.rs2) {
                    self.pc = self.pc.wrapping_add(insn.imm as u64);
                    return;
                };
            }

            // Jump opcodes
            Instruction::Jal(insn) => {
                let result = self.pc.wrapping_add(4);
                self.reg_mut(insn.rd, result);
                self.pc = self.pc.wrapping_add(insn.imm as u64);
                return;
            }

            Instruction::Jalr(insn) => {
                let target = self.reg(insn.rs1).wrapping_add(insn.imm as u64);
                let result = self.pc.wrapping_add(if is_compressed { 2 } else { 4 });
                self.reg_mut(insn.rd, result);
                self.pc = target;
                return;
            }

            // Lui and Auipc
            Instruction::Lui(insn) => {
                self.reg_mut(insn.rd, insn.imm as u64);
            }

            Instruction::Auipc(insn) => {
                let result = self.pc.wrapping_add(insn.imm as u64);
                self.reg_mut(insn.rd, result);
            }

            // RV64I Instructions
            Instruction::Addiw(insn) => {
                let res = self.reg(insn.rs1).wrapping_add(insn.imm as u64) & mask(32);
                let result = sext(res, 32);
                self.reg_mut(insn.rd, result);
            }

            Instruction::Slliw(insn) => {
                let val = self.reg(insn.rs1) << insn.shamt;
                let result = sext(val & mask(32), 32);
                self.reg_mut(insn.rd, result);
            }

            Instruction::Srliw(insn) => {
                let result = sext((self.reg(insn.rs1) & mask(32)) >> insn.shamt, 32);
                self.reg_mut(insn.rd, result);
            }

            Instruction::Sraiw(insn) => {
                let shift = insn.shamt;
                let a = (self.reg(insn.rs1) & mask(32)) as i32;
                let result = (a >> shift) as i64 as u64;
                self.reg_mut(insn.rd, result);
            }

            Instruction::Addw(insn) => {
                let result = sext(
                    self.reg(insn.rs1).wrapping_add(self.reg(insn.rs2)) & mask(32),
                    32,
                );
                self.reg_mut(insn.rd, result);
            }

            Instruction::Subw(insn) => {
                let a = self.reg(insn.rs1) as i32;
                let b = self.reg(insn.rs2) as i32;
                let result = a.wrapping_sub(b) as i64 as u64;
                self.reg_mut(insn.rd, result);
            }

            Instruction::Sllw(insn) => {
                let a = self.reg(insn.rs1);
                let shift = self.reg(insn.rs2) & mask(5);
                let result = sext((a << shift) & mask(32), 32);
                self.reg_mut(insn.rd, result);
            }

            Instruction::Srlw(insn) => {
                let a = self.reg(insn.rs1) & mask(32);
                let shift = self.reg(insn.rs2) & mask(5);
                let result = sext(a >> shift, 32);
                self.reg_mut(insn.rd, result);
            }

            Instruction::Sraw(insn) => {
                let a = (self.reg(insn.rs1) & mask(32)) as i32;
                let shift = self.reg(insn.rs2) & mask(5);
                let result = (a >> shift) as i64 as u64;
                self.reg_mut(insn.rd, result);
            }

            // M Extension - Multiplication
            Instruction::Mul(insn) => {
                let a = self.reg(insn.rs1) as i64;
                let b = self.reg(insn.rs2) as i64;
                let full = (a as i128).wrapping_mul(b as i128);
                let result = a.wrapping_mul(b) as u64;
                self.tracer.record_mul(result, (full >> 64) as u64);
                self.reg_mut(insn.rd, result);
            }

            Instruction::Mulh(insn) => {
                let a = (self.reg(insn.rs1) as i64) as i128;
                let b = (self.reg(insn.rs2) as i64) as i128;
                let full = a.wrapping_mul(b);
                let lo = full as u64;
                let hi = (full >> 64) as u64;
                self.tracer.record_mul(lo, hi);
                self.reg_mut(insn.rd, hi);
            }

            Instruction::Mulhsu(insn) => {
                let a = (self.reg(insn.rs1) as i64) as i128;
                let b = (self.reg(insn.rs2) as u128) as i128;
                let full = a.wrapping_mul(b);
                let lo = full as u64;
                let hi = (full >> 64) as u64;
                self.tracer.record_mul(lo, hi);
                self.reg_mut(insn.rd, hi);
            }

            Instruction::Mulhu(insn) => {
                let a = self.reg(insn.rs1) as u128;
                let b = self.reg(insn.rs2) as u128;
                let full = a.wrapping_mul(b);
                let lo = full as u64;
                let hi = (full >> 64) as u64;
                self.tracer.record_mul(lo, hi);
                self.reg_mut(insn.rd, hi);
            }

            Instruction::Mulw(insn) => {
                let a = self.reg(insn.rs1);
                let b = self.reg(insn.rs2);
                let product = a.wrapping_mul(b);
                let result = (((product & mask(32)) as i32) as i64) as u64;
                self.tracer.record_mul(product & mask(32), 0);
                self.reg_mut(insn.rd, result);
            }

            // M Extension - Division
            Instruction::Div(insn) => {
                let dividend = self.reg(insn.rs1) as i64;
                let divisor = self.reg(insn.rs2) as i64;
                let result = if divisor == 0 {
                    u64::MAX
                } else if dividend == i64::MIN && divisor == -1 {
                    dividend as u64
                } else {
                    dividend.wrapping_div(divisor) as u64
                };
                self.reg_mut(insn.rd, result);
            }

            Instruction::Divu(insn) => {
                let dividend = self.reg(insn.rs1);
                let divisor = self.reg(insn.rs2);
                let result = if divisor == 0 {
                    u64::MAX
                } else {
                    dividend.wrapping_div(divisor)
                };
                self.reg_mut(insn.rd, result);
            }

            Instruction::Rem(insn) => {
                let dividend = self.reg(insn.rs1) as i64;
                let divisor = self.reg(insn.rs2) as i64;
                let result = if divisor == 0 {
                    dividend as u64
                } else if dividend == i64::MIN && divisor == -1 {
                    0
                } else {
                    dividend.wrapping_rem(divisor) as u64
                };
                self.reg_mut(insn.rd, result);
            }

            Instruction::Remu(insn) => {
                let dividend = self.reg(insn.rs1);
                let divisor = self.reg(insn.rs2);
                let result = if divisor == 0 {
                    dividend
                } else {
                    dividend.wrapping_rem(divisor)
                };
                self.reg_mut(insn.rd, result);
            }

            Instruction::Divw(insn) => {
                let dividend = (self.reg(insn.rs1) & mask(32)) as i32;
                let divisor = (self.reg(insn.rs2) & mask(32)) as i32;
                let result = if divisor == 0 {
                    u64::MAX
                } else if dividend == i32::MIN && divisor == -1 {
                    (dividend as i64) as u64
                } else {
                    (dividend.wrapping_div(divisor) as i64) as u64
                };
                self.reg_mut(insn.rd, result);
            }

            Instruction::Divuw(insn) => {
                let dividend = (self.reg(insn.rs1) & mask(32)) as u32;
                let divisor = (self.reg(insn.rs2) & mask(32)) as u32;
                let result = if divisor == 0 {
                    u64::MAX
                } else {
                    sext(dividend.wrapping_div(divisor) as u64, 32)
                };
                self.reg_mut(insn.rd, result);
            }

            Instruction::Remw(insn) => {
                let dividend = (self.reg(insn.rs1) & mask(32)) as i32;
                let divisor = (self.reg(insn.rs2) & mask(32)) as i32;
                let result = if divisor == 0 {
                    (dividend as i64) as u64
                } else if dividend == i32::MIN && divisor == -1 {
                    0
                } else {
                    (dividend.wrapping_rem(divisor) as i64) as u64
                };
                self.reg_mut(insn.rd, result);
            }

            Instruction::Remuw(insn) => {
                let dividend = (self.reg(insn.rs1) & mask(32)) as u32;
                let divisor = (self.reg(insn.rs2) & mask(32)) as u32;
                let result = if divisor == 0 {
                    sext(dividend as u64, 32)
                } else {
                    sext(dividend.wrapping_rem(divisor) as u64, 32)
                };
                self.reg_mut(insn.rd, result);
            }

            // A Extension - Load Reserved / Store Conditional
            Instruction::LrW(insn) => {
                let addr = self.reg(insn.rs1);
                let value = self.load_u32(addr as usize) as u64;
                let result = sext(value, 32);
                self.reservation_set = addr;
                self.tracer.record_reservation(addr);
                self.tracer.record_mem_op(MemOp::LoadReservedWord {
                    addr,
                    value: value as u32,
                });
                self.reg_mut(insn.rd, result);
            }

            Instruction::LrD(insn) => {
                let addr = self.reg(insn.rs1);
                let value = self.load_u64(addr as usize);
                self.reservation_set = addr;
                self.tracer.record_reservation(addr);
                self.tracer
                    .record_mem_op(MemOp::LoadReservedDouble { addr, value });
                self.reg_mut(insn.rd, value);
            }

            Instruction::ScW(insn) => {
                let addr = self.reg(insn.rs1);
                let value = self.reg(insn.rs2) & mask(32);
                let success = addr == self.reservation_set;
                if success {
                    self.store_u32(addr as usize, value as u32);
                }
                let result = if success { 0 } else { 1 };
                self.reservation_set = 0;
                self.tracer.record_mem_op(MemOp::StoreConditionalWord {
                    addr,
                    value: value as u32,
                    success,
                });
                self.reg_mut(insn.rd, result);
            }

            Instruction::ScD(insn) => {
                let addr = self.reg(insn.rs1);
                let value = self.reg(insn.rs2);
                let success = addr == self.reservation_set;
                if success {
                    self.store_u64(addr as usize, value);
                }
                let result = if success { 0 } else { 1 };
                self.reservation_set = 0;
                self.tracer.record_mem_op(MemOp::StoreConditionalDouble {
                    addr,
                    value,
                    success,
                });
                self.reg_mut(insn.rd, result);
            }

            // A Extension - Atomic Memory Operations (Word)
            Instruction::AmoSwapW(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u32(addr as usize) as u64;
                let write_value = self.reg(insn.rs2) & mask(32);
                self.store_u32(addr as usize, write_value as u32);
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.reg_mut(insn.rd, sext(read_value, 32));
            }

            Instruction::AmoAddW(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u32(addr as usize) as i32;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let write_value = (read_value.wrapping_add(rs2_val) as i64) as u64 & mask(32);
                self.store_u32(addr as usize, write_value as u32);
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.reg_mut(insn.rd, (read_value as i64) as u64);
            }

            Instruction::AmoXorW(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u32(addr as usize) as i32;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let write_value = ((read_value ^ rs2_val) as i64) as u64 & mask(32);
                self.store_u32(addr as usize, write_value as u32);
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.reg_mut(insn.rd, (read_value as i64) as u64);
            }

            Instruction::AmoAndW(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u32(addr as usize) as i32;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let write_value = ((read_value & rs2_val) as i64) as u64 & mask(32);
                self.store_u32(addr as usize, write_value as u32);
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.reg_mut(insn.rd, (read_value as i64) as u64);
            }

            Instruction::AmoOrW(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u32(addr as usize) as i32;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let write_value = ((read_value | rs2_val) as i64) as u64 & mask(32);
                self.store_u32(addr as usize, write_value as u32);
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.reg_mut(insn.rd, (read_value as i64) as u64);
            }

            Instruction::AmoMinW(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u32(addr as usize) as i32;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let write_value = (read_value.min(rs2_val) as i64) as u64 & mask(32);
                self.store_u32(addr as usize, write_value as u32);
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.reg_mut(insn.rd, (read_value as i64) as u64);
            }

            Instruction::AmoMaxW(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u32(addr as usize) as i32;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let write_value = (read_value.max(rs2_val) as i64) as u64 & mask(32);
                self.store_u32(addr as usize, write_value as u32);
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.reg_mut(insn.rd, (read_value as i64) as u64);
            }

            Instruction::AmoMinuW(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u32(addr as usize) as u64;
                let rs2_val = self.reg(insn.rs2) & mask(32);
                let write_value = read_value.min(rs2_val);
                self.store_u32(addr as usize, write_value as u32);
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.reg_mut(insn.rd, sext(read_value, 32));
            }

            Instruction::AmoMaxuW(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u32(addr as usize) as u64;
                let rs2_val = self.reg(insn.rs2) & mask(32);
                let write_value = read_value.max(rs2_val);
                self.store_u32(addr as usize, write_value as u32);
                self.tracer.record_mem_op(MemOp::AtomicWord {
                    addr,
                    read_value: read_value as u32,
                    write_value: write_value as u32,
                });
                self.reg_mut(insn.rd, sext(read_value, 32));
            }

            // A Extension - Atomic Memory Operations (Double)
            Instruction::AmoSwapD(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u64(addr as usize);
                let write_value = self.reg(insn.rs2);
                self.store_u64(addr as usize, write_value);
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.reg_mut(insn.rd, read_value);
            }

            Instruction::AmoAddD(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u64(addr as usize);
                let rs2_val = self.reg(insn.rs2);
                let write_value = read_value.wrapping_add(rs2_val);
                self.store_u64(addr as usize, write_value);
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.reg_mut(insn.rd, read_value);
            }

            Instruction::AmoXorD(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u64(addr as usize);
                let rs2_val = self.reg(insn.rs2);
                let write_value = read_value ^ rs2_val;
                self.store_u64(addr as usize, write_value);
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.reg_mut(insn.rd, read_value);
            }

            Instruction::AmoAndD(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u64(addr as usize);
                let rs2_val = self.reg(insn.rs2);
                let write_value = read_value & rs2_val;
                self.store_u64(addr as usize, write_value);
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.reg_mut(insn.rd, read_value);
            }

            Instruction::AmoOrD(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u64(addr as usize);
                let rs2_val = self.reg(insn.rs2);
                let write_value = read_value | rs2_val;
                self.store_u64(addr as usize, write_value);
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.reg_mut(insn.rd, read_value);
            }

            Instruction::AmoMinD(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u64(addr as usize);
                let rs2_val = self.reg(insn.rs2) as i64;
                let write_value = (read_value as i64).min(rs2_val) as u64;
                self.store_u64(addr as usize, write_value);
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.reg_mut(insn.rd, read_value);
            }

            Instruction::AmoMaxD(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u64(addr as usize);
                let rs2_val = self.reg(insn.rs2) as i64;
                let write_value = (read_value as i64).max(rs2_val) as u64;
                self.store_u64(addr as usize, write_value);
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.reg_mut(insn.rd, read_value);
            }

            Instruction::AmoMinuD(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u64(addr as usize);
                let rs2_val = self.reg(insn.rs2);
                let write_value = read_value.min(rs2_val);
                self.store_u64(addr as usize, write_value);
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.reg_mut(insn.rd, read_value);
            }

            Instruction::AmoMaxuD(insn) => {
                let addr = self.reg(insn.rs1);
                let read_value = self.load_u64(addr as usize);
                let rs2_val = self.reg(insn.rs2);
                let write_value = read_value.max(rs2_val);
                self.store_u64(addr as usize, write_value);
                self.tracer.record_mem_op(MemOp::AtomicDouble {
                    addr,
                    read_value,
                    write_value,
                });
                self.reg_mut(insn.rd, read_value);
            }

            // F instructions
            Instruction::FmaddS(insn) => {
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

            Instruction::FmsubS(insn) => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let c = self.read_f32(insn.rs3);
                let res = a.mul_add(b, -c);
                self.write_f32(insn.rd, res);
                self.raise_fflags_fma_f32(a, b, -c, res);
            }

            Instruction::FnmsubS(insn) => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let c = self.read_f32(insn.rs3);
                let res = (-a).mul_add(b, c);
                self.write_f32(insn.rd, res);
                self.raise_fflags_fma_f32(-a, b, c, res);
            }

            Instruction::FnmaddS(insn) => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);
                let c = self.read_f32(insn.rs3);
                let res = (-a).mul_add(b, -c);
                self.write_f32(insn.rd, res);
                self.raise_fflags_fma_f32(-a, b, -c, res);
            }

            Instruction::FaddS(insn) => {
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

            Instruction::FsubS(insn) => {
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

            Instruction::FmulS(insn) => {
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

            Instruction::FdivS(insn) => {
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

            Instruction::FsqrtS(insn) => {
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
                        self.tracer.record_csr_reg(self.fcsr_reg);
                    }
                }

                self.write_f32(insn.rd, res);
            }

            Instruction::FsgnjS(insn) => {
                let rs1_bits = (self.f_reg[insn.rs1 as usize] & 0xFFFFFFFF) as u32;
                let rs2_bits = (self.f_reg[insn.rs2 as usize] & 0xFFFFFFFF) as u32;
                let sign = rs2_bits & (1 << 31);
                let val = rs1_bits & mask32(31);
                let result = sign | val;
                let res = 0xFFFF_FFFF_0000_0000 | (result as u64);
                self.f_reg[insn.rd as usize] = res;
                self.tracer.record_rd(insn.rd, res);
            }

            Instruction::FsgnjnS(insn) => {
                let rs1_bits = (self.f_reg[insn.rs1 as usize] & 0xFFFFFFFF) as u32;
                let rs2_bits = (self.f_reg[insn.rs2 as usize] & 0xFFFFFFFF) as u32;
                let sign = (rs2_bits ^ (1 << 31)) & (1 << 31);
                let val = rs1_bits & mask32(31);
                let result = sign | val;
                let res = 0xFFFF_FFFF_0000_0000 | (result as u64);
                self.f_reg[insn.rd as usize] = res;
                self.tracer.record_rd(insn.rd, res);
            }

            Instruction::FsgnjxS(insn) => {
                let rs1_bits = (self.f_reg[insn.rs1 as usize] & 0xFFFFFFFF) as u32;
                let rs2_bits = (self.f_reg[insn.rs2 as usize] & 0xFFFFFFFF) as u32;
                let sign = (rs1_bits & (1 << 31)) ^ (rs2_bits & (1 << 31));
                let val = rs1_bits & mask32(31);
                let result = sign | val;
                let res = 0xFFFF_FFFF_0000_0000 | (result as u64);
                self.f_reg[insn.rd as usize] = res;
                self.tracer.record_rd(insn.rd, res);
            }

            Instruction::FminS(insn) => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // Set NV flag for signaling NaN
                if is_snan_f32(a) || is_snan_f32(b) {
                    self.fcsr_reg |= 0b10000;
                    self.tracer.record_csr_reg(self.fcsr_reg);
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

            Instruction::FmaxS(insn) => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // Set NV flag for signaling NaN
                if is_snan_f32(a) || is_snan_f32(b) {
                    self.fcsr_reg |= 0b10000;
                    self.tracer.record_csr_reg(self.fcsr_reg);
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

            Instruction::FcvtWS(insn) => {
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
                self.reg_mut(insn.rd, result as i64 as u64);
                self.tracer.record_csr_reg(self.fcsr_reg);
            }

            Instruction::FcvtWuS(insn) => {
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
                self.reg_mut(insn.rd, result as i32 as i64 as u64);
                self.tracer.record_csr_reg(self.fcsr_reg);
            }

            Instruction::FmvXW(insn) => {
                let raw_bits = (self.f_reg[insn.rs1 as usize] & 0xFFFFFFFF) as u32;
                let result = sext(raw_bits as u64, 32);

                self.reg_mut(insn.rd, result);
            }

            Instruction::FeqS(insn) => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // FeqS only sets NV for signaling NaN
                if is_snan_f32(a) || is_snan_f32(b) {
                    self.fcsr_reg |= 0b10000;
                    self.tracer.record_csr_reg(self.fcsr_reg);
                }

                let res = if a.is_nan() || b.is_nan() {
                    0
                } else {
                    (a == b) as u64
                };
                self.reg_mut(insn.rd, res);
            }

            Instruction::FltS(insn) => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // FltS sets NV for ANY NaN (not just signaling)
                if a.is_nan() || b.is_nan() {
                    self.fcsr_reg |= 0b10000;
                    self.tracer.record_csr_reg(self.fcsr_reg);
                    self.reg_mut(insn.rd, 0);
                } else {
                    self.reg_mut(insn.rd, (a < b) as u64);
                }
            }

            Instruction::FleS(insn) => {
                let a = self.read_f32(insn.rs1);
                let b = self.read_f32(insn.rs2);

                // FleS sets NV for ANY NaN (not just signaling)
                if a.is_nan() || b.is_nan() {
                    self.fcsr_reg |= 0b10000;
                    self.tracer.record_csr_reg(self.fcsr_reg);
                    self.reg_mut(insn.rd, 0);
                } else {
                    self.reg_mut(insn.rd, (a <= b) as u64);
                }
            }
            Instruction::FclassS(insn) => {
                let val = classify32(self.read_f32(insn.rs1).to_bits());
                self.reg_mut(insn.rd, val);
            }

            Instruction::FcvtSW(insn) => {
                let a = (self.reg(insn.rs1) as i32) as f32;
                self.write_f32(insn.rd, a);
            }

            Instruction::FcvtSWu(insn) => {
                let a = (self.reg(insn.rs1) as u32) as f32;
                self.write_f32(insn.rd, a);
            }

            Instruction::FmvWX(insn) => {
                let a = f32::from_bits(self.reg(insn.rs1) as u32);
                self.write_f32(insn.rd, a);
            }

            Instruction::FmaddD(insn) => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let c = self.read_f64(insn.rs3);
                let res = a.mul_add(b, c);
                self.write_f64(insn.rd, res);
                self.raise_fflags_fma_f64(a, b, c, res);
            }

            Instruction::FmsubD(insn) => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let c = self.read_f64(insn.rs3);
                let res = a.mul_add(b, -c);
                self.write_f64(insn.rd, res);
                self.raise_fflags_fma_f64(a, b, -c, res);
            }

            Instruction::FnmsubD(insn) => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let c = self.read_f64(insn.rs3);
                let res = (-a).mul_add(b, c);
                self.write_f64(insn.rd, res);
                self.raise_fflags_fma_f64(-a, b, c, res);
            }

            Instruction::FnmaddD(insn) => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let c = self.read_f64(insn.rs3);
                let res = (-a).mul_add(b, -c);
                self.write_f64(insn.rd, res);
                self.raise_fflags_fma_f64(-a, b, -c, res);
            }

            Instruction::FaddD(insn) => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let mut res = a + b;

                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000); // Canonical positive qNaN
                }

                self.write_f64(insn.rd, res);
                self.raise_fflags_f64(a, b, res, '+');
            }

            Instruction::FsubD(insn) => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let mut res = a - b;

                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000);
                }

                self.write_f64(insn.rd, res);
                self.raise_fflags_f64(a, b, res, '-');
            }

            Instruction::FmulD(insn) => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let mut res = a * b;

                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000);
                }

                self.write_f64(insn.rd, res);
                self.raise_fflags_f64(a, b, res, '*');
            }

            Instruction::FdivD(insn) => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);
                let mut res = a / b;

                if res.is_nan() && !a.is_nan() && !b.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000);
                }

                self.write_f64(insn.rd, res);
                self.raise_fflags_f64(a, b, res, '/');
            }

            Instruction::FsqrtD(insn) => {
                let a = self.read_f64(insn.rs1);

                if is_snan_f64(a) || (a < 0.0 && !a.is_nan()) {
                    self.fcsr_reg |= 0b10000;
                    self.tracer.record_csr_reg(self.fcsr_reg);
                }

                let mut res = a.sqrt();

                if res.is_nan() && !a.is_nan() {
                    res = f64::from_bits(0x7FF8000000000000);
                }

                self.write_f64(insn.rd, res);
            }

            Instruction::FsgnjD(insn) => {
                let sign = self.read_f64(insn.rs2).to_bits() & (1 << 63);
                let val = self.read_f64(insn.rs1).to_bits() & mask(63);
                let res = f64::from_bits(sign | val);
                self.write_f64(insn.rd, res);
            }

            Instruction::FsgnjnD(insn) => {
                let sign = (self.read_f64(insn.rs2).to_bits() ^ (1 << 63)) & (1 << 63);
                let val = self.read_f64(insn.rs1).to_bits() & mask(63);
                let res = f64::from_bits(sign | val);
                self.write_f64(insn.rd, res);
            }

            Instruction::FsgnjxD(insn) => {
                let sign = (self.read_f64(insn.rs1).to_bits() & (1 << 63))
                    ^ (self.read_f64(insn.rs2).to_bits() & (1 << 63));
                let val = self.read_f64(insn.rs1).to_bits() & mask(63);
                let res = f64::from_bits(sign | val);
                self.write_f64(insn.rd, res);
            }

            Instruction::FminD(insn) => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if is_snan_f64(a) || is_snan_f64(b) {
                    self.fcsr_reg |= 0b10000;
                    self.tracer.record_csr_reg(self.fcsr_reg);
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

            Instruction::FmaxD(insn) => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if is_snan_f64(a) || is_snan_f64(b) {
                    self.fcsr_reg |= 0b10000;
                    self.tracer.record_csr_reg(self.fcsr_reg);
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

            Instruction::FcvtSD(insn) => {
                let a = self.read_f64(insn.rs1);
                let res = a as f32;

                // Set NX if precision was lost
                if !a.is_nan() && !a.is_infinite() && (res as f64) != a {
                    self.fcsr_reg |= 0b00001;
                    self.tracer.record_csr_reg(self.fcsr_reg);
                }

                // Set NV for sNaN
                if is_snan_f64(a) {
                    self.fcsr_reg |= 0b10000;
                    self.tracer.record_csr_reg(self.fcsr_reg);
                }

                self.write_f32(insn.rd, res);
            }

            Instruction::FcvtDS(insn) => {
                let a = self.read_f32(insn.rs1);

                // Set NV for sNaN
                if is_snan_f32(a) {
                    self.fcsr_reg |= 0b10000;
                    self.tracer.record_csr_reg(self.fcsr_reg);
                }

                let res = a as f64;
                self.write_f64(insn.rd, res);
            }

            Instruction::FeqD(insn) => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if is_snan_f64(a) || is_snan_f64(b) {
                    self.fcsr_reg |= 0b10000;
                    self.tracer.record_csr_reg(self.fcsr_reg);
                }

                let res = if a.is_nan() || b.is_nan() {
                    0
                } else {
                    (a == b) as u64
                };

                self.reg_mut(insn.rd, res);
            }

            Instruction::FltD(insn) => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if a.is_nan() || b.is_nan() {
                    self.fcsr_reg |= 0b10000;
                    self.tracer.record_csr_reg(self.fcsr_reg);
                    self.reg_mut(insn.rd, 0);
                } else {
                    self.reg_mut(insn.rd, (a < b) as u64);
                }
            }

            Instruction::FleD(insn) => {
                let a = self.read_f64(insn.rs1);
                let b = self.read_f64(insn.rs2);

                if a.is_nan() || b.is_nan() {
                    self.fcsr_reg |= 0b10000;
                    self.tracer.record_csr_reg(self.fcsr_reg);
                    self.reg_mut(insn.rd, 0);
                } else {
                    self.reg_mut(insn.rd, (a <= b) as u64);
                }
            }

            Instruction::FclassD(insn) => {
                let val = classify64(self.read_f64(insn.rs1).to_bits());
                self.reg_mut(insn.rd, val);
            }

            Instruction::FcvtWD(insn) => {
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
                self.reg_mut(insn.rd, result as i64 as u64);
                self.tracer.record_csr_reg(self.fcsr_reg);
            }

            Instruction::FcvtWuD(insn) => {
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
                self.reg_mut(insn.rd, result as i32 as i64 as u64);
                self.tracer.record_csr_reg(self.fcsr_reg);
            }

            Instruction::FcvtDW(insn) => {
                let a = (self.reg(insn.rs1) as i32) as f64;
                self.write_f64(insn.rd, a);
            }

            Instruction::FcvtDWu(insn) => {
                let a = (self.reg(insn.rs1) as u32) as f64;
                self.write_f64(insn.rd, a);
            }

            Instruction::Flw(insn) => {
                let addr = (self.reg(insn.rs1).wrapping_add(insn.imm as u64)) as usize;
                let data = f32::from_bits(self.load_u32(addr));
                self.write_f32(insn.rd, data);
            }

            Instruction::Fsw(insn) => {
                let addr = (self.reg(insn.rs1).wrapping_add(insn.imm as u64)) as usize;
                let data = self.read_f32(insn.rs2).to_bits().to_le_bytes();
                self.store_u32(addr, u32::from_le_bytes(data));
                self.tracer.record_mem_op(MemOp::StoreWord {
                    addr: addr as u64,
                    value: u32::from_le_bytes(data),
                });
            }

            Instruction::Fld(insn) => {
                let addr = (self.reg(insn.rs1).wrapping_add(insn.imm as u64)) as usize;
                let val = f64::from_bits(self.load_u64(addr));
                self.write_f64(insn.rd, val);
            }

            Instruction::Fsd(insn) => {
                let data = self.read_f64(insn.rs2).to_le_bytes();
                let addr = (self.reg(insn.rs1).wrapping_add(insn.imm as u64)) as usize;
                self.store_u64(addr, u64::from_le_bytes(data));
                self.tracer.record_mem_op(MemOp::StoreDouble {
                    addr: addr as u64,
                    value: u64::from_le_bytes(data),
                });
            }

            Instruction::FcvtLS(insn) => {
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
                self.reg_mut(insn.rd, result as u64);
                self.tracer.record_csr_reg(self.fcsr_reg);
            }

            Instruction::FcvtLuS(insn) => {
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
                self.reg_mut(insn.rd, result);
                self.tracer.record_csr_reg(self.fcsr_reg);
            }

            Instruction::FcvtSL(insn) => {
                let val = (self.reg(insn.rs1) as i64) as f32;
                self.write_f32(insn.rd, val);
            }

            Instruction::FcvtSLu(insn) => {
                let val = self.reg(insn.rs1) as f32;
                self.write_f32(insn.rd, val);
            }

            Instruction::FcvtLD(insn) => {
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
                self.reg_mut(insn.rd, result as u64);
                self.tracer.record_csr_reg(self.fcsr_reg);
            }

            Instruction::FcvtLuD(insn) => {
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
                self.reg_mut(insn.rd, result);
                self.tracer.record_csr_reg(self.fcsr_reg);
            }

            Instruction::FmvXD(insn) => {
                let val = self.read_f64(insn.rs1).to_bits();
                self.reg_mut(insn.rd, val);
            }

            Instruction::FcvtDL(insn) => {
                let val = (self.reg(insn.rs1) as i64) as f64;
                self.write_f64(insn.rd, val);
            }

            Instruction::FcvtDLu(insn) => {
                let val = self.reg(insn.rs1) as f64;
                self.write_f64(insn.rd, val);
            }

            Instruction::FmvDX(insn) => {
                let val = f64::from_bits(self.reg(insn.rs1));
                self.write_f64(insn.rd, val);
            }

            // CSR instructions
            Instruction::Csrrw(insn) => {
                let csr_addr = (insn.imm as u32) & 0xFFF; // Mask to 12 bits
                let old = self.read_csr(csr_addr) as u64;
                let val = self.reg(insn.rs1) as u32;

                self.set_csr(csr_addr, val);
                if insn.rd != 0 {
                    self.reg_mut(insn.rd, old);
                }
            }

            Instruction::Csrrs(insn) => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                if insn.rs1 != 0 {
                    let val = self.reg(insn.rs1) as u32;
                    let new_val = old as u32 | val;
                    self.set_csr(csr_addr, new_val);
                }
                if insn.rd != 0 {
                    self.reg_mut(insn.rd, old);
                }
            }

            Instruction::Csrrc(insn) => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                if insn.rs1 != 0 {
                    let val = self.reg(insn.rs1) as u32;
                    let new_val = old as u32 & !val;
                    self.set_csr(csr_addr, new_val);
                }
                if insn.rd != 0 {
                    self.reg_mut(insn.rd, old);
                }
            }

            Instruction::Csrrwi(insn) => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                let val = (insn.rs1 as u32) & 0x1F;
                self.set_csr(csr_addr, val);
                if insn.rd != 0 {
                    self.reg_mut(insn.rd, old);
                }
            }

            Instruction::Csrrsi(insn) => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                let val = (insn.rs1 as u32) & 0x1F;
                if val != 0 {
                    let new_val = old as u32 | val;
                    self.set_csr(csr_addr, new_val);
                }
                if insn.rd != 0 {
                    self.reg_mut(insn.rd, old);
                }
            }

            Instruction::Csrrci(insn) => {
                let csr_addr = (insn.imm as u32) & 0xFFF;
                let old = self.read_csr(csr_addr) as u64;
                let val = (insn.rs1 as u32) & 0x1F;
                if val != 0 {
                    let new_val = old as u32 & !val;
                    self.set_csr(csr_addr, new_val);
                }
                if insn.rd != 0 {
                    self.reg_mut(insn.rd, old);
                }
            }

            // System Opcodes
            Instruction::Ecall => {
                handle_ecall(self);
            }

            // TODO remove the eager check once all opcodes have been implemented
            _ => {}
        }

        if is_compressed {
            self.pc += 2;
        } else {
            self.pc += 4;
        }
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
    use crate::ecall::constants;
    use crate::trace::NoopTracer;
    use crate::{VM, decode};

    #[test]
    fn test_add_instruction() {
        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(3, 12);
        vm.reg_mut(5, 32);
        // r8 = r3 + r5
        // 0x518433 = Instruction::Add(R { rd: 8, rs1: 3, rs2: 5 });
        let insn = 0x518433;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.reg(8), 12 + 32);
    }

    #[test]
    fn test_store_byte() {
        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(3, 12);
        vm.reg_mut(2, 5);
        // 0x310123 = Instruction::Sb(S {rs1: 2, rs2: 3, imm: 2});
        let insn = 0x310123;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.load_u64(7), 12);
    }

    #[test]
    fn test_store_half_word() {
        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(3, 64008);
        vm.reg_mut(2, 5);
        // 0x311123 = Instruction::Sh(S {rs1: 2, rs2: 3, imm: 2});
        let insn = 0x311123;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.load_u64(7), 64008);
        assert_eq!(vm.load_u64(8), 250);
    }

    #[test]
    fn test_store_word() {
        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(3, 2299561908);
        vm.reg_mut(2, 5);
        // 0x312123 = Instruction::Sw(S { rs1: 2, rs2: 3, imm: 2 });
        let insn = 0x312123;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.load_u64(7), 2299561908);
        assert_eq!(vm.load_u64(8), 8982663);
        assert_eq!(vm.load_u64(9), 35088);
    }

    #[test]
    fn test_store_double_word() {
        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(3, 1234567898765432123);
        vm.reg_mut(2, 5);
        // 0x313123 = Instruction::Sd(S { rs1: 2, rs2: 3, imm: 2 });
        let insn = 0x313123;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.load_u64(7), 1234567898765432123);
        assert_eq!(vm.load_u64(8), 4822530854552469);
        assert_eq!(vm.load_u64(9), 18838011150595);
        assert_eq!(vm.load_u64(10), 73585981057);
        assert_eq!(vm.load_u64(11), 287445238);
        assert_eq!(vm.load_u64(12), 1122832);
    }

    #[test]
    fn test_jal_opcode() {
        let mut vm = VM::<NoopTracer>::init();
        vm.pc = 8;
        // 0xC001EF = Instruction::Jal(J { rd: 3, imm: 12 });
        let insn = 0xC001EF;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.reg(3), 12);
        assert_eq!(vm.pc, 20);
    }

    #[test]
    fn test_jalr_opcode() {
        let mut vm = VM::<NoopTracer>::init();
        vm.pc = 8;
        vm.reg_mut(5, 6);
        // 0x9281E7 = Instruction::Jalr(I {rs1: 5, rd: 3, imm: 9});
        let insn = 0x9281E7;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.reg(3), 12);
        assert_eq!(vm.pc, 15);
    }

    #[test]
    fn test_ecall_stdin() {
        let mut vm = VM::<NoopTracer>::init();

        // Prepare an input stream "hello"
        vm.input_stream = b"hello".to_vec();
        vm.input_cursor = 0;

        // a0 = fd (stdin), a1 = guest ptr, a2 = len
        vm.reg_mut(10, constants::STDIN_FILENO); // x10 = a0
        vm.reg_mut(11, 0); // x11 = a1 -> memory addr 0
        vm.reg_mut(12, 3); // x12 = a2 -> read 3 bytes

        // place ecall function (ECALL_STD_INPUT) in x17 (a7)
        vm.reg_mut(17, constants::ECALL_STD_INPUT as u64);

        // execute ecall (standard encoding 0x0000_0073)
        let insn = 0x0000_0073;
        vm.execute_instruction(decode(insn), false);

        // check bytes written to guest memory and return value in a0
        assert_eq!(vm.read_bytes(0, 3), b"hel".to_vec());
        assert_eq!(vm.reg(10), 3);
    }

    #[test]
    fn test_ecall_stdout() {
        let mut vm = VM::<NoopTracer>::init();

        // Write "world" into guest memory at address 0
        vm.write_bytes(0, b"world");

        // a0 = fd (stdout), a1 = guest ptr, a2 = len
        vm.reg_mut(10, constants::STDOUT_FILENO); // x10 = a0
        vm.reg_mut(11, 0); // x11 = a1 -> memory addr 0
        vm.reg_mut(12, 5); // x12 = a2 -> length

        // place ecall function (ECALL_STD_OUTPUT) in x17 (a7)
        vm.reg_mut(17, constants::ECALL_STD_OUTPUT as u64);

        // execute ecall
        let insn = 0x0000_0073;
        vm.execute_instruction(decode(insn), false);

        // stdout handler returns length read in a0
        assert_eq!(vm.reg(10), 5);
    }
}
