use crate::{Instruction, Opcode, VM};

impl VM {
    pub(crate) fn execute_instruction(&mut self, insn: Instruction) {
        match insn.opcode {
            Opcode::Add => {
                self.set_reg(insn.rd, self.reg(insn.rs1) + self.reg(insn.rs2));
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
        vm.set_reg(3, 12);
        vm.set_reg(5, 32);
        // r8 = r3 + r5
        vm.execute_instruction(Instruction::new(Opcode::Add).rs1(3).rs2(5).rd(8));
        assert_eq!(vm.reg(8), 12 + 32);
    }
}
