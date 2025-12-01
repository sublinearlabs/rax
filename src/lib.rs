#[derive(Default)]
struct VM {
    registers: [u64; 32],
}

impl VM {
    /// Returns a VM with empty state
    fn init() -> Self {
        Self::default()
    }

    fn reg(&self, idx: usize) -> u64 {
        if idx == 0 { 0 } else { self.registers[idx] }
    }

    fn set_reg(&mut self, idx: usize, val: u64) {
        if idx != 0 {
            self.registers[idx] = val;
        }
    }

    fn execute_instruction(&mut self, insn: Instruction) {
        match insn.opcode {
            Opcode::Add => {
                self.set_reg(insn.rd, self.reg(insn.rs1) + self.reg(insn.rs2));
            }

            // TODO remove the earger check once all opcodes have been implemented
            _ => {}
        }
    }
}

// RISCV insturction
struct Instruction {
    opcode: Opcode,
    rd: usize,
    rs1: usize,
    rs2: usize,
}

impl Instruction {
    /// Build a new instruction
    fn new(opcode: Opcode, rd: usize, rs1: usize, rs2: usize) -> Self {
        Self {
            opcode,
            rd,
            rs1,
            rs2,
        }
    }
}

// RISCV Opcodes
enum Opcode {
    Add,
    Sub,
    Xor,
    Or,
    And,
    Sll,
    Srl,
    Sra,
    Slt,
    Sltu,
}

#[cfg(test)]
mod tests {
    use crate::Opcode;

    use super::*;

    #[test]
    fn test_register_read_write() {
        let mut vm = VM::init();

        // read
        assert_eq!(vm.reg(5), 0);
        // write
        vm.set_reg(5, 10);
        assert_eq!(vm.reg(5), 10);
        // write
        vm.set_reg(5, 20);
        assert_eq!(vm.reg(5), 20);
    }

    #[test]
    fn test_register_0_always_0() {
        let mut vm = VM::init();
        // read register 0
        assert_eq!(vm.reg(0), 0);
        // write to register 0
        vm.set_reg(0, 20);
        assert_eq!(vm.reg(0), 0);
    }

    #[test]
    fn test_add_instruction() {
        let mut vm = VM::init();
        vm.set_reg(3, 12);
        vm.set_reg(5, 32);
        // r8 = r3 + r5
        vm.execute_instruction(Instruction::new(Opcode::Add, 8, 3, 5));
        assert_eq!(vm.reg(8), 12 + 32);
    }
}
