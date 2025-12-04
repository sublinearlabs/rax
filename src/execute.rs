use crate::{
    Instruction, Opcode, VM,
    util::{mask, sext},
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

            Opcode::Slt | Opcode::Sltu => {
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

            Opcode::Slti | Opcode::Sltiu => {
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
                *self.reg_mut(insn.rd) = self.mem(addr as usize);
            }

            // Store Opcodes
            Opcode::Sb => {
                *self.mem_mut(insn.rs1 + (insn.imm as usize)) =
                    (self.reg(insn.rs2) & mask(8)) as u8;
            }

            Opcode::Sh => {
                let addr = insn.rs1 + (insn.imm as usize);
                for i in 0..4 {
                    *self.mem_mut(addr + i) = ((self.reg(insn.rs2) >> (8 * i)) & mask(8)) as u8;
                }
            }

            // Suspicious, need to look into this some more (but doesn't seem pressing for add to
            // work)
            Opcode::Sw => {
                let addr = insn.rs1 + (insn.imm as usize);
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
                if (self.reg(insn.rs1) as i32) < (self.reg(insn.rs2) as i32) {
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
                *self.reg_mut(insn.rd) = self.pc + 4;
                self.pc += insn.imm;
                return;
            }

            Opcode::Jalr => {
                *self.reg_mut(insn.rd) = self.pc + 4;
                self.pc = self.reg(insn.rs1) as u64 + insn.imm;
                return;
            }

            // Lui and Auipc
            Opcode::Lui => {
                *self.reg_mut(insn.rd) = insn.imm;
            }

            Opcode::Auipc => {
                *self.reg_mut(insn.rd) = self.pc.wrapping_add(insn.imm);
            }

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

            // TODO there is still a problem with this
            Opcode::Sraiw => {
                let val = (self.reg(insn.rs1) & mask(32)) as i64;
                *self.reg_mut(insn.rd) = (val >> (insn.imm & mask(5))) as u64;
            }

            Opcode::Addw => {
                *self.reg_mut(insn.rd) = sext(
                    self.reg(insn.rs1).wrapping_add(self.reg(insn.rs2)) & mask(32),
                    32,
                );
            }

            Opcode::Subw => {
                // TODO why do I still get upper bits not empty
                let a = self.reg(insn.rs1) as i32;
                let b = self.reg(insn.rs2) as i32;
                let val = a.wrapping_sub(b);
                *self.reg_mut(insn.rd) = sext(val as u64, 32);
            }

            Opcode::Sllw => {
                *self.reg_mut(insn.rd) = sext(
                    (self.reg(insn.rs1) << (self.reg(insn.rs2) & mask(5))) & mask(32),
                    32,
                );
            }

            Opcode::Srlw => {
                *self.reg_mut(insn.rd) = sext(
                    (self.reg(insn.rs1) & mask(32)) >> (self.reg(insn.rs2) & mask(5)),
                    32,
                );
            }

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
        let insn = Instruction::new(Opcode::Sb).rs1(5).imm(2).rs2(3);
        vm.execute_instruction(insn);
        assert_eq!(vm.mem(7), 12);
    }

    #[test]
    fn test_store_half_word() {
        let mut vm = VM::init();
        *vm.reg_mut(3) = 4194867295;
        let insn = Instruction::new(Opcode::Sh).rs1(5).imm(2).rs2(3);
        vm.execute_instruction(insn);
        assert_eq!(vm.mem(7), 4194867295);
        assert_eq!(vm.mem(8), 16386200);
        assert_eq!(vm.mem(9), 64008);
    }

    #[test]
    fn test_store_word() {
        let mut vm = VM::init();
        *vm.reg_mut(3) = 1234567898765432123;
        let insn = Instruction::new(Opcode::Sw).rs1(5).imm(2).rs2(3);
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
