use crate::{
    Instruction, Opcode, VM,
    util::{mask, mask32, sext},
};

// TODO consider cleaning up sext logic

impl VM {
    pub(crate) fn execute_instruction(&mut self, insn: Instruction) {
        match insn.opcode {
            // Register Opcodes
            Opcode::Add => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1).wrapping_add(self.reg(insn.rs2));
            }

            Opcode::Sub => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1).wrapping_sub(self.reg(insn.rs2));
            }

            Opcode::Xor => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) ^ self.reg(insn.rs2);
            }

            Opcode::Or => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) | self.reg(insn.rs2);
            }

            Opcode::And => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) & self.reg(insn.rs2);
            }

            Opcode::Sll => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) << (self.reg(insn.rs2) & mask(6));
            }

            Opcode::Srl => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) >> (self.reg(insn.rs2) & mask(6));
            }

            Opcode::Sra => {
                let val = self.reg(insn.rs1) as i64;
                *self.reg_mut(insn.rd) = (val >> (self.reg(insn.rs2) & mask(6))) as u64;
            }

            Opcode::Slt => {
                *self.reg_mut(insn.rd) =
                    if (self.reg(insn.rs1) as i64) < (self.reg(insn.rs2) as i64) {
                        1
                    } else {
                        0
                    };
            }

            Opcode::Sltu => {
                *self.reg_mut(insn.rd) = if self.reg(insn.rs1) < self.reg(insn.rs2) {
                    1
                } else {
                    0
                };
            }

            // Immediate Opcodes
            Opcode::Addi => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1).wrapping_add(insn.imm);
            }

            Opcode::Xori => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) ^ insn.imm;
            }

            Opcode::Ori => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) | insn.imm;
            }

            Opcode::Andi => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) & insn.imm;
            }

            Opcode::Slli => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) << insn.imm;
            }

            Opcode::Srli => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) >> (insn.imm & mask(6));
            }

            Opcode::Srai => {
                let shift = insn.imm & mask(6);
                let val = self.reg(insn.rs1) as i64;
                *self.reg_mut(insn.rd) = (val >> shift) as u64;
            }

            Opcode::Slti => {
                *self.reg_mut(insn.rd) = if (self.reg(insn.rs1) as i64) < (insn.imm as i64) {
                    1
                } else {
                    0
                };
            }

            Opcode::Sltiu => {
                *self.reg_mut(insn.rd) = if self.reg(insn.rs1) < insn.imm { 1 } else { 0 };
            }

            Opcode::Lb => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                *self.reg_mut(insn.rd) = sext(self.mem(addr as usize) & mask(8), 8);
            }

            Opcode::Lbu => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                *self.reg_mut(insn.rd) = self.mem(addr as usize) & mask(8);
            }

            Opcode::Lh => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                *self.reg_mut(insn.rd) = sext(self.mem(addr as usize) & mask(16), 16);
            }

            Opcode::Lhu => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                *self.reg_mut(insn.rd) = self.mem(addr as usize) & mask(16);
            }

            Opcode::Lw => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                *self.reg_mut(insn.rd) = sext(self.mem(addr as usize) & mask(32), 32);
            }

            Opcode::Ld => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm);
                *self.reg_mut(insn.rd) = self.mem(addr as usize);
            }

            // Store Opcodes
            Opcode::Sb => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm) as usize;
                *self.mem_mut(addr) = (self.reg(insn.rs2) & mask(8)) as u8;
            }

            Opcode::Sh => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm) as usize;
                for i in 0..2 {
                    *self.mem_mut(addr + i) = ((self.reg(insn.rs2) >> (8 * i)) & mask(8)) as u8;
                }
            }

            // Suspicious, need to look into this some more (but doesn't seem pressing for add to
            // work)
            Opcode::Sw => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm) as usize;
                for i in 0..4 {
                    *self.mem_mut(addr + i) = ((self.reg(insn.rs2) >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::Sd => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm) as usize;
                for i in 0..8 {
                    *self.mem_mut(addr + i) = ((self.reg(insn.rs2) >> (8 * i)) & mask(8)) as u8;
                }
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
                *self.reg_mut(insn.rd) = self.pc.wrapping_add(4);
                self.pc = self.pc.wrapping_add(insn.imm);
                return;
            }

            Opcode::Jalr => {
                let old_rs1 = self.reg(insn.rs1);
                *self.reg_mut(insn.rd) = self.pc.wrapping_add(4);
                self.pc = old_rs1.wrapping_add(insn.imm);
                return;
            }

            // Lui and Auipc
            Opcode::Lui => {
                *self.reg_mut(insn.rd) = insn.imm;
            }

            Opcode::Auipc => {
                *self.reg_mut(insn.rd) = self.pc.wrapping_add(insn.imm);
            }

            // I Instructions
            Opcode::Addiw => {
                let res = self.reg(insn.rs1).wrapping_add(insn.imm) & mask(32);
                *self.reg_mut(insn.rd) = sext(res, 32);
            }

            Opcode::Slliw => {
                let val = self.reg(insn.rs1) << (insn.imm & mask(5));
                *self.reg_mut(insn.rd) = sext(val & mask(32), 32);
            }

            Opcode::Srliw => {
                *self.reg_mut(insn.rd) = sext((self.reg(insn.rs1) & mask(32)) >> insn.imm, 32);
            }

            Opcode::Sraiw => {
                let shift = (insn.imm & mask(5)) as i32;
                let a = (self.reg(insn.rs1) & mask(32)) as i32;
                *self.reg_mut(insn.rd) = (a >> shift) as i64 as u64;
            }

            Opcode::Addw => {
                *self.reg_mut(insn.rd) = sext(
                    self.reg(insn.rs1).wrapping_add(self.reg(insn.rs2)) & mask(32),
                    32,
                );
            }

            Opcode::Subw => {
                let a = self.reg(insn.rs1) as i32;
                let b = self.reg(insn.rs2) as i32;
                let val = a.wrapping_sub(b) as i64;
                *self.reg_mut(insn.rd) = val as u64;
            }

            Opcode::Sllw => {
                let a = self.reg(insn.rs1);
                let shift = self.reg(insn.rs2) & mask(5);
                *self.reg_mut(insn.rd) = sext((a << shift) & mask(32), 32);
            }

            Opcode::Srlw => {
                let a = self.reg(insn.rs1) & mask(32);
                let shift = self.reg(insn.rs2) & mask(5);
                *self.reg_mut(insn.rd) = sext(a >> shift, 32);
            }

            Opcode::Sraw => {
                let a = (self.reg(insn.rs1) & mask(32)) as i32;
                let shift = self.reg(insn.rs2) & mask(5);
                *self.reg_mut(insn.rd) = (a >> shift) as i64 as u64;
            }

            Opcode::Lwu => {
                let addr = self.reg(insn.rs1).wrapping_add(insn.imm) as usize;
                *self.reg_mut(insn.rd) = self.mem(addr) & mask(32);
            }

            // M Instructions
            Opcode::Mul => {
                let a = self.reg(insn.rs1) as i64;
                let b = self.reg(insn.rs2) as i64;
                *self.reg_mut(insn.rd) = a.wrapping_mul(b) as u64;
            }

            Opcode::Mulh => {
                let a = (self.reg(insn.rs1) as i64) as i128;
                let b = ((self.reg(insn.rs2)) as i64) as i128;
                *self.reg_mut(insn.rd) = (a.wrapping_mul(b) >> 64) as u64;
            }

            Opcode::Mulhsu => {
                let a = (self.reg(insn.rs1) as i64) as i128;
                let b = (self.reg(insn.rs2) as u128) as i128;
                *self.reg_mut(insn.rd) = (a.wrapping_mul(b) >> 64) as u64;
            }

            Opcode::Mulhu => {
                let a = self.reg(insn.rs1) as u128;
                let b = self.reg(insn.rs2) as u128;
                *self.reg_mut(insn.rd) = (a.wrapping_mul(b) >> 64) as u64;
            }

            Opcode::Div => {
                let dividend = self.reg(insn.rs1) as i64;
                let divisor = self.reg(insn.rs2) as i64;

                *self.reg_mut(insn.rd) = if divisor == 0 {
                    u64::MAX
                } else if dividend == i64::MIN && divisor == -1 {
                    dividend as u64
                } else {
                    dividend.wrapping_div(divisor) as u64
                }
            }

            Opcode::Divu => {
                let dividend = self.reg(insn.rs1);
                let divisor = self.reg(insn.rs2);

                *self.reg_mut(insn.rd) = if divisor == 0 {
                    u64::MAX
                } else {
                    dividend.wrapping_div(divisor)
                }
            }

            Opcode::Rem => {
                let dividend = self.reg(insn.rs1) as i64;
                let divisor = self.reg(insn.rs2) as i64;

                *self.reg_mut(insn.rd) = if divisor == 0 {
                    dividend as u64
                } else if dividend == i64::MIN && divisor == -1 {
                    0
                } else {
                    dividend.wrapping_rem(divisor) as u64
                }
            }

            Opcode::Remu => {
                let dividend = self.reg(insn.rs1);
                let divisor = self.reg(insn.rs2);

                *self.reg_mut(insn.rd) = if divisor == 0 {
                    dividend
                } else {
                    dividend.wrapping_rem(divisor)
                }
            }

            Opcode::Mulw => {
                let a = self.reg(insn.rs1);
                let b = self.reg(insn.rs2);
                *self.reg_mut(insn.rd) = (((a.wrapping_mul(b) & mask(32)) as i32) as i64) as u64
            }

            Opcode::Divw => {
                let dividend = (self.reg(insn.rs1) & mask(32)) as i32;
                let divisor = (self.reg(insn.rs2) & mask(32)) as i32;

                *self.reg_mut(insn.rd) = if divisor == 0 {
                    u64::MAX
                } else if dividend == i32::MIN && divisor == -1 {
                    (dividend as i64) as u64
                } else {
                    (dividend.wrapping_div(divisor) as i64) as u64
                }
            }

            Opcode::Divuw => {
                let dividend = (self.reg(insn.rs1) & mask(32)) as u32;
                let divisor = (self.reg(insn.rs2) & mask(32)) as u32;

                *self.reg_mut(insn.rd) = if divisor == 0 {
                    u64::MAX
                } else {
                    sext(dividend.wrapping_div(divisor) as u64, 32)
                }
            }

            Opcode::Remw => {
                let dividend = (self.reg(insn.rs1) & mask(32)) as i32;
                let divisor = (self.reg(insn.rs2) & mask(32)) as i32;

                *self.reg_mut(insn.rd) = if divisor == 0 {
                    (dividend as i64) as u64
                } else if dividend == i32::MIN && divisor == -1 {
                    0
                } else {
                    (dividend.wrapping_rem(divisor) as i64) as u64
                }
            }

            Opcode::Remuw => {
                let dividend = (self.reg(insn.rs1) & mask(32)) as u32;
                let divisor = (self.reg(insn.rs2) & mask(32)) as u32;

                *self.reg_mut(insn.rd) = if divisor == 0 {
                    sext(dividend as u64, 32)
                } else {
                    sext(dividend.wrapping_rem(divisor) as u64, 32)
                }
            }

            // Zalrsc A instructions
            Opcode::LrW => {
                *self.reg_mut(insn.rd) = sext(self.mem(self.reg(insn.rs1) as usize) & mask(32), 32);
                self.reservation_set = self.reg(insn.rs1);
            }

            Opcode::LrD => {
                let addr = self.reg(insn.rs1);
                *self.reg_mut(insn.rd) = self.mem(addr as usize);
                self.reservation_set = addr;
            }

            Opcode::ScW => {
                let addr = self.reg(insn.rs1) as usize;
                if self.reg(insn.rs1) == self.reservation_set {
                    for i in 0..4 {
                        *self.mem_mut(addr + i) = ((self.reg(insn.rs2) >> (8 * i)) & mask(8)) as u8;
                    }
                    *self.reg_mut(insn.rd) = 0;
                } else {
                    *self.reg_mut(insn.rd) = 1;
                };
                self.reservation_set = 0;
            }

            Opcode::ScD => {
                let addr = self.reg(insn.rs1) as usize;
                if self.reg(insn.rs1) == self.reservation_set {
                    for i in 0..8 {
                        *self.mem_mut(addr + i) = ((self.reg(insn.rs2) >> (8 * i)) & mask(8)) as u8;
                    }
                    *self.reg_mut(insn.rd) = 0;
                } else {
                    *self.reg_mut(insn.rd) = 1;
                };
                self.reservation_set = 0;
            }

            // Zaamo A instructions
            Opcode::AmoswapW => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = self.mem(addr) & mask(32);
                *self.reg_mut(insn.rd) = sext(temp, 32);
                for i in 0..4 {
                    *self.mem_mut(addr + i) = ((self.reg(insn.rs2) >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmoaddW => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = (self.mem(addr) & mask(32)) as i32;
                *self.reg_mut(insn.rd) = (temp as i64) as u64;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let res = (temp.wrapping_add(rs2_val) as i64) as u64;
                for i in 0..4 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmoxorW => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = (self.mem(addr) & mask(32)) as i32;
                *self.reg_mut(insn.rd) = (temp as i64) as u64;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let res = ((temp ^ rs2_val) as i64) as u64;
                for i in 0..4 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmoandW => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = (self.mem(addr) & mask(32)) as i32;
                *self.reg_mut(insn.rd) = (temp as i64) as u64;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let res = ((temp & rs2_val) as i64) as u64;
                for i in 0..4 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmoorW => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = (self.mem(addr) & mask(32)) as i32;
                *self.reg_mut(insn.rd) = (temp as i64) as u64;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let res = ((temp | rs2_val) as i64) as u64;
                for i in 0..4 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmominW => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = (self.mem(addr) & mask(32)) as i32;
                *self.reg_mut(insn.rd) = (temp as i64) as u64;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let res = (temp.min(rs2_val) as i64) as u64;
                for i in 0..4 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmomaxW => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = (self.mem(addr) & mask(32)) as i32;
                *self.reg_mut(insn.rd) = (temp as i64) as u64;
                let rs2_val = (self.reg(insn.rs2) & mask(32)) as i32;
                let res = (temp.max(rs2_val) as i64) as u64;
                for i in 0..4 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmominuW => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = self.mem(addr) & mask(32);
                *self.reg_mut(insn.rd) = sext(temp, 32);
                let rs2_val = self.reg(insn.rs2) & mask(32);
                let res = sext(temp.min(rs2_val), 32);
                for i in 0..4 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmomaxuW => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = self.mem(addr) & mask(32);
                *self.reg_mut(insn.rd) = sext(temp, 32);
                let rs2_val = self.reg(insn.rs2) & mask(32);
                let res = sext(temp.max(rs2_val), 32);
                for i in 0..4 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmoswapD => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = self.mem(addr);
                *self.reg_mut(insn.rd) = temp;
                for i in 0..8 {
                    *self.mem_mut(addr + i) = ((self.reg(insn.rs2) >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmoaddD => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = self.mem(addr);
                *self.reg_mut(insn.rd) = temp;
                let rs2_val = self.reg(insn.rs2);
                let res = temp.wrapping_add(rs2_val);
                for i in 0..8 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmoxorD => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = self.mem(addr);
                *self.reg_mut(insn.rd) = temp;
                let rs2_val = self.reg(insn.rs2);
                let res = temp ^ rs2_val;
                for i in 0..8 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmoandD => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = self.mem(addr);
                *self.reg_mut(insn.rd) = temp;
                let rs2_val = self.reg(insn.rs2);
                let res = temp & rs2_val;
                for i in 0..8 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmoorD => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = self.mem(addr);
                *self.reg_mut(insn.rd) = temp;
                let rs2_val = self.reg(insn.rs2);
                let res = temp | rs2_val;
                for i in 0..8 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmominD => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = self.mem(addr);
                *self.reg_mut(insn.rd) = temp;
                let rs2_val = self.reg(insn.rs2) as i64;
                let res = (temp as i64).min(rs2_val) as u64;
                for i in 0..8 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmomaxD => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = self.mem(addr);
                *self.reg_mut(insn.rd) = temp;
                let rs2_val = self.reg(insn.rs2) as i64;
                let res = (temp as i64).max(rs2_val) as u64;
                for i in 0..8 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmominuD => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = self.mem(addr);
                *self.reg_mut(insn.rd) = temp;
                let rs2_val = self.reg(insn.rs2);
                let res = temp.min(rs2_val);
                for i in 0..8 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            Opcode::AmomaxuD => {
                let addr = self.reg(insn.rs1) as usize;
                let temp = self.mem(addr);
                *self.reg_mut(insn.rd) = temp;
                let rs2_val = self.reg(insn.rs2);
                let res = temp.max(rs2_val);
                for i in 0..8 {
                    *self.mem_mut(addr + i) = ((res >> (8 * i)) & mask(8)) as u8;
                }
            }

            // F instructions
            Opcode::FmaddS => {
                let rs3 = insn.imm >> 2;
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                let b = f32::from_bits(self.f_reg(insn.rs2) as u32);
                let c = f32::from_bits(self.f_reg(rs3 as usize) as u32);
                *self.f_reg_mut(insn.rd) = a.mul_add(b, c).to_bits() as u64;
            }

            Opcode::FmsubS => {
                let rs3 = insn.imm >> 2;
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                let b = f32::from_bits(self.f_reg(insn.rs2) as u32);
                let c = f32::from_bits(self.f_reg(rs3 as usize) as u32);
                *self.f_reg_mut(insn.rd) = a.mul_add(b, -c).to_bits() as u64;
            }

            Opcode::FnmsubS => {
                let rs3 = insn.imm >> 2;
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                let b = f32::from_bits(self.f_reg(insn.rs2) as u32);
                let c = f32::from_bits(self.f_reg(rs3 as usize) as u32);
                *self.f_reg_mut(insn.rd) = (-a).mul_add(b, c).to_bits() as u64;
            }

            Opcode::FnmaddS => {
                let rs3 = insn.imm >> 2;
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                let b = f32::from_bits(self.f_reg(insn.rs2) as u32);
                let c = f32::from_bits(self.f_reg(rs3 as usize) as u32);
                *self.f_reg_mut(insn.rd) = (-a).mul_add(b, -c).to_bits() as u64;
            }

            Opcode::FaddS => {
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                let b = f32::from_bits(self.f_reg(insn.rs2) as u32);
                *self.f_reg_mut(insn.rd) = (a + b).to_bits() as u64;
            }

            Opcode::FsubS => {
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                let b = f32::from_bits(self.f_reg(insn.rs2) as u32);
                *self.f_reg_mut(insn.rd) = (a - b).to_bits() as u64;
            }

            Opcode::FmulS => {
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                let b = f32::from_bits(self.f_reg(insn.rs2) as u32);
                *self.f_reg_mut(insn.rd) = (a * b).to_bits() as u64;
            }

            Opcode::FdivS => {
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                let b = f32::from_bits(self.f_reg(insn.rs2) as u32);
                *self.f_reg_mut(insn.rd) = (a / b).to_bits() as u64;
            }

            Opcode::FsqrtS => {
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                *self.f_reg_mut(insn.rd) = a.sqrt().to_bits() as u64;
            }

            Opcode::FsgnjS => {
                let sign = self.f_reg(insn.rs2) & (1 << 31);
                let val = self.f_reg(insn.rs1) & mask(31);
                let data = sign | val;
                *self.f_reg_mut(insn.rd) = data;
            }

            Opcode::FsgnjnS => {
                let sign = (self.f_reg(insn.rs2) ^ (1 << 31)) & (1 << 31);
                let val = self.f_reg(insn.rs1) & mask(31);
                let data = sign | val;
                *self.f_reg_mut(insn.rd) = data;
            }

            Opcode::FsgnjxS => {
                let sign = (self.f_reg(insn.rs2) & (1 << 31)) ^ (self.f_reg(insn.rs1) & (1 << 31));
                let val = self.f_reg(insn.rs1) & mask(31);
                let data = sign | val;
                *self.f_reg_mut(insn.rd) = data;
            }

            Opcode::FminS => {
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                let b = f32::from_bits(self.f_reg(insn.rs2) as u32);
                *self.f_reg_mut(insn.rd) = a.min(b).to_bits() as u64;
            }

            Opcode::FmaxS => {
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                let b = f32::from_bits(self.f_reg(insn.rs2) as u32);
                *self.f_reg_mut(insn.rd) = a.max(b).to_bits() as u64;
            }

            Opcode::FcvtWS => {
                let a = (f32::from_bits(self.f_reg(insn.rs1) as u32) as i32) as i64;
                *self.reg_mut(insn.rd) = a as u64;
            }

            Opcode::FcvtWuS => {
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32) as u32;
                *self.reg_mut(insn.rd) = a as u64;
            }

            Opcode::FmvXW => {
                *self.reg_mut(insn.rd) = sext(self.f_reg(insn.rs1) & mask(32), 32);
            }

            Opcode::FeqS => {
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                let b = f32::from_bits(self.f_reg(insn.rs2) as u32);
                *self.reg_mut(insn.rd) = if a.is_nan() || b.is_nan() {
                    0
                } else {
                    a.eq(&b) as u64
                };
            }

            Opcode::FltS => {
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                let b = f32::from_bits(self.f_reg(insn.rs2) as u32);
                *self.reg_mut(insn.rd) = if a.is_nan() || b.is_nan() {
                    0
                } else {
                    a.lt(&b) as u64
                };
            }

            Opcode::FleS => {
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32);
                let b = f32::from_bits(self.f_reg(insn.rs2) as u32);
                *self.reg_mut(insn.rd) = if a.is_nan() || b.is_nan() {
                    0
                } else {
                    a.le(&b) as u64
                };
            }

            Opcode::FclassS => {
                let val = classify32(self.f_reg(insn.rs1) as u32);
                *self.reg_mut(insn.rd) = val;
            }

            Opcode::FcvtSW => {
                let a = (self.reg(insn.rs1) as i32) as f32;
                *self.f_reg_mut(insn.rd) = a.to_bits() as u64;
            }

            Opcode::FcvtSWu => {
                let a = (self.reg(insn.rs1) as u32) as f32;
                *self.f_reg_mut(insn.rd) = a.to_bits() as u64;
            }

            Opcode::FmvWX => {
                let a = self.reg(insn.rs1) as u32;
                *self.f_reg_mut(insn.rd) = a as u64;
            }

            Opcode::FmaddD => {
                let rs3 = insn.imm >> 2;
                let a = f64::from_bits(self.f_reg(insn.rs1));
                let b = f64::from_bits(self.f_reg(insn.rs2));
                let c = f64::from_bits(self.f_reg(rs3 as usize));
                *self.f_reg_mut(insn.rd) = a.mul_add(b, c).to_bits();
            }

            Opcode::FmsubD => {
                let rs3 = insn.imm >> 2;
                let a = f64::from_bits(self.f_reg(insn.rs1));
                let b = f64::from_bits(self.f_reg(insn.rs2));
                let c = f64::from_bits(self.f_reg(rs3 as usize));
                *self.f_reg_mut(insn.rd) = a.mul_add(b, -c).to_bits();
            }

            Opcode::FnmsubD => {
                let rs3 = insn.imm >> 2;
                let a = f64::from_bits(self.f_reg(insn.rs1));
                let b = f64::from_bits(self.f_reg(insn.rs2));
                let c = f64::from_bits(self.f_reg(rs3 as usize));
                *self.f_reg_mut(insn.rd) = (-a).mul_add(b, c).to_bits();
            }

            Opcode::FnmaddD => {
                let rs3 = insn.imm >> 2;
                let a = f64::from_bits(self.f_reg(insn.rs1));
                let b = f64::from_bits(self.f_reg(insn.rs2));
                let c = f64::from_bits(self.f_reg(rs3 as usize));
                *self.f_reg_mut(insn.rd) = (-a).mul_add(b, -c).to_bits();
            }

            Opcode::FaddD => {
                let a = f64::from_bits(self.f_reg(insn.rs1));
                let b = f64::from_bits(self.f_reg(insn.rs2));
                *self.f_reg_mut(insn.rd) = (a + b).to_bits();
            }

            Opcode::FsubD => {
                let a = f64::from_bits(self.f_reg(insn.rs1));
                let b = f64::from_bits(self.f_reg(insn.rs2));
                *self.f_reg_mut(insn.rd) = (a - b).to_bits();
            }

            Opcode::FmulD => {
                let a = f64::from_bits(self.f_reg(insn.rs1));
                let b = f64::from_bits(self.f_reg(insn.rs2));
                *self.f_reg_mut(insn.rd) = (a * b).to_bits();
            }

            Opcode::FdivD => {
                let a = f64::from_bits(self.f_reg(insn.rs1));
                let b = f64::from_bits(self.f_reg(insn.rs2));
                *self.f_reg_mut(insn.rd) = (a / b).to_bits();
            }

            Opcode::FsqrtD => {
                let a = f64::from_bits(self.f_reg(insn.rs1));
                *self.f_reg_mut(insn.rd) = a.sqrt().to_bits();
            }

            Opcode::FsgnjD => {
                let sign = self.f_reg(insn.rs2) & (1 << 63);
                let val = self.f_reg(insn.rs1) & mask(63);
                let res = sign | val;
                *self.f_reg_mut(insn.rd) = res;
            }

            Opcode::FsgnjnD => {
                let sign = (self.f_reg(insn.rs2) ^ (1 << 63)) & (1 << 63);
                let val = self.f_reg(insn.rs1) & mask(63);
                let res = sign | val;
                *self.f_reg_mut(insn.rd) = res;
            }

            Opcode::FsgnjxD => {
                let sign = (self.f_reg(insn.rs1) & (1 << 63)) ^ (self.f_reg(insn.rs2) & (1 << 63));
                let val = self.f_reg(insn.rs1) & mask(63);
                let res = sign | val;
                *self.f_reg_mut(insn.rd) = res;
            }

            Opcode::FminD => {
                let a = f64::from_bits(self.f_reg(insn.rs1));
                let b = f64::from_bits(self.f_reg(insn.rs2));
                *self.f_reg_mut(insn.rd) = a.min(b).to_bits();
            }

            Opcode::FmaxD => {
                let a = self.f_reg(insn.rs1) as f64;
                let b = self.f_reg(insn.rs2) as f64;
                *self.f_reg_mut(insn.rd) = a.max(b).to_bits();
            }

            Opcode::FcvtSD => {
                let a = f64::from_bits(self.f_reg(insn.rs1)) as f32;
                *self.f_reg_mut(insn.rd) = a.to_bits() as u64;
            }

            Opcode::FcvtDS => {
                let a = f32::from_bits(self.f_reg(insn.rs1) as u32) as f64;
                *self.f_reg_mut(insn.rd) = a.to_bits();
            }

            Opcode::FeqD => {
                let a = f64::from_bits(self.f_reg(insn.rs1));
                let b = f64::from_bits(self.f_reg(insn.rs2));
                *self.reg_mut(insn.rd) = if a.is_nan() || b.is_nan() {
                    0
                } else {
                    a.eq(&b) as u64
                };
            }

            Opcode::FltD => {
                let a = f64::from_bits(self.f_reg(insn.rs1));
                let b = f64::from_bits(self.f_reg(insn.rs2));
                *self.reg_mut(insn.rd) = if a.is_nan() || b.is_nan() {
                    0
                } else {
                    a.lt(&b) as u64
                };
            }

            Opcode::FleD => {
                let a = f64::from_bits(self.f_reg(insn.rs1));
                let b = f64::from_bits(self.f_reg(insn.rs2));
                *self.reg_mut(insn.rd) = if a.is_nan() || b.is_nan() {
                    0
                } else {
                    a.le(&b) as u64
                };
            }

            Opcode::FclassD => {
                let val = classify64(self.f_reg(insn.rs1));
                *self.reg_mut(insn.rd) = val;
            }

            Opcode::FcvtWD => {
                let a = (f64::from_bits(self.f_reg(insn.rs1)) as i32) as i64;
                *self.reg_mut(insn.rd) = a as u64;
            }

            Opcode::FcvtWuD => {
                let a = f64::from_bits(self.f_reg(insn.rs1)) as u32;
                *self.reg_mut(insn.rd) = a as u64;
            }

            Opcode::FcvtDW => {
                let a = (self.reg(insn.rs1) as i32) as f64;
                *self.f_reg_mut(insn.rd) = a.to_bits();
            }

            Opcode::FcvtDWu => {
                let a = (self.reg(insn.rs1) as u32) as f64;
                *self.f_reg_mut(insn.rd) = a.to_bits();
            }

            Opcode::Flw => {
                let addr = (self.reg(insn.rs1) + insn.imm) as usize;
                let data = self.mem(addr) & mask(32);
                *self.f_reg_mut(insn.rd) = data;
            }

            Opcode::Fsw => {
                let addr = (self.reg(insn.rs1) + insn.imm) as usize;
                let data = (self.f_reg(insn.rs2) as u32).to_le_bytes();
                self.write_bytes(addr, &data);
            }

            Opcode::Fld => {
                let addr = (self.reg(insn.rs1) + insn.imm) as usize;
                *self.f_reg_mut(insn.rd) = self.mem(addr);
            }

            Opcode::Fsd => {
                let data = self.f_reg(insn.rs2).to_le_bytes();
                let addr = (self.reg(insn.rs1) + insn.imm) as usize;
                self.write_bytes(addr, &data);
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

            // TODO remove the earger check once all opcodes have been implemented
            _ => {}
        }

        self.pc += 4;
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
    use crate::{Instruction, Opcode, VM};

    #[test]
    fn test_add_instruction() {
        let mut vm = VM::init();
        *vm.reg_mut(3) = 12;
        *vm.reg_mut(5) = 32;
        // r8 = r3 + r5
        vm.execute_instruction(Instruction::new(Opcode::Add).rs1(3).rs2(5).rd(8));
        assert_eq!(vm.reg(8), 12 + 32);
    }

    #[test]
    fn test_store_byte() {
        let mut vm = VM::init();
        *vm.reg_mut(3) = 12;
        *vm.reg_mut(2) = 5;
        let insn = Instruction::new(Opcode::Sb).rs1(2).imm(2).rs2(3);
        vm.execute_instruction(insn);
        assert_eq!(vm.mem(7), 12);
    }

    #[test]
    fn test_store_half_word() {
        let mut vm = VM::init();
        *vm.reg_mut(3) = 64008;
        *vm.reg_mut(2) = 5;
        let insn = Instruction::new(Opcode::Sh).rs1(2).imm(2).rs2(3);
        vm.execute_instruction(insn);
        assert_eq!(vm.mem(7), 64008);
        assert_eq!(vm.mem(8), 250);
    }

    #[test]
    fn test_store_word() {
        let mut vm = VM::init();
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
        let mut vm = VM::init();
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
        let mut vm = VM::init();
        vm.pc = 8;
        let insn = Instruction::new(Opcode::Jal).imm(12).rd(3);
        vm.execute_instruction(insn);
        assert_eq!(vm.reg(3), 12);
        assert_eq!(vm.pc, 20);
    }

    #[test]
    fn test_jalr_opcode() {
        let mut vm = VM::init();
        vm.pc = 8;
        *vm.reg_mut(5) = 6;
        let insn = Instruction::new(Opcode::Jalr).rs1(5).imm(9).rd(3);
        vm.execute_instruction(insn);
        assert_eq!(vm.reg(3), 12);
        assert_eq!(vm.pc, 15);
    }
}
