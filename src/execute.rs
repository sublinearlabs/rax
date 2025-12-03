use crate::{Instruction, Opcode, VM};

impl VM {
    pub(crate) fn execute_instruction(&mut self, insn: Instruction) {
        match insn.opcode {
            // Register Opcodes
            Opcode::Add => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) + self.reg(insn.rs2);
            }

            Opcode::Sub => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) - self.reg(insn.rs2);
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
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) << self.reg(insn.rs2);
            }

            Opcode::Srl => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) >> self.reg(insn.rs2);
            }

            Opcode::Sra => {
                *self.reg_mut(insn.rd) = self.reg(insn.rs1) >> self.reg(insn.rs2);
            }

            Opcode::Slt => {
                *self.reg_mut(insn.rd) = if self.reg(insn.rs1) < self.reg(insn.rs2) {
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

            // TODO remove the earger check once all opcodes have been implemented
            _ => {}
        }
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
}
