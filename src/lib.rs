mod execute;
mod memory;

#[derive(Default)]
struct VM {
    registers: [u64; 32],
    x0_sink: u64, // blackhole for writes to x0
}

impl VM {
    /// Returns a VM with empty state
    fn init() -> Self {
        Self::default()
    }

    fn reg(&self, idx: usize) -> u64 {
        if idx == 0 { 0 } else { self.registers[idx] }
    }

    fn reg_mut(&mut self, idx: usize) -> &mut u64 {
        if idx == 0 {
            &mut self.x0_sink
        } else {
            &mut self.registers[idx]
        }
    }
}

// RISCV insturction
struct Instruction {
    opcode: Opcode,
    rd: usize,
    rs1: usize,
    rs2: usize,
    imm: u64,
}

impl Instruction {
    fn new(opcode: Opcode) -> Self {
        Self {
            opcode,
            rd: 0,
            rs1: 0,
            rs2: 0,
            imm: 0,
        }
    }

    fn rd(self, val: usize) -> Self {
        Self { rd: val, ..self }
    }

    fn rs1(self, val: usize) -> Self {
        Self { rs1: val, ..self }
    }

    fn rs2(self, val: usize) -> Self {
        Self { rs2: val, ..self }
    }

    fn imm(self, val: u64) -> Self {
        Self { imm: val, ..self }
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
    use super::*;

    #[test]
    fn test_register_read_write() {
        let mut vm = VM::init();

        // read
        assert_eq!(vm.reg(5), 0);
        // write
        *vm.reg_mut(5) = 10;
        assert_eq!(vm.reg(5), 10);
        // write
        *vm.reg_mut(5) = 20;
        assert_eq!(vm.reg(5), 20);
    }

    #[test]
    fn test_register_0_always_0() {
        let mut vm = VM::init();
        // read register 0
        assert_eq!(vm.reg(0), 0);
        // write to register 0
        *vm.reg_mut(0) = 20;
        assert_eq!(vm.reg(0), 0);
    }
}
