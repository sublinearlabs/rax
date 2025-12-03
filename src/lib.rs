use crate::memory::Memory;

mod execute;
mod memory;
mod util;

#[derive(Default)]
struct VM {
    registers: [u64; 32],
    memory: Memory,
    x0_sink: u64, // blackhole for writes to x0
    pc: usize,
}

impl VM {
    /// Returns a VM with empty state
    fn init() -> Self {
        Self::default()
    }

    /// Returns the current value at the idx register
    fn reg(&self, idx: usize) -> u64 {
        if idx == 0 { 0 } else { self.registers[idx] }
    }

    /// Returns a mutable reference to the idx register
    fn reg_mut(&mut self, idx: usize) -> &mut u64 {
        if idx == 0 {
            &mut self.x0_sink
        } else {
            &mut self.registers[idx]
        }
    }

    /// Reads 64 bytes from memory at the given addr
    /// assumes value at memory address is the LSB
    fn mem(&self, addr: usize) -> u64 {
        let mut result = 0_u64;
        for i in 0..8 {
            let byte = self.memory.read((addr + i) as u64);
            result |= (byte as u64) << (i * 8);
        }
        result
    }

    /// Returns a mutable reference to a single byte at the given
    /// memory addr
    fn mem_mut(&mut self, addr: usize) -> &mut u8 {
        self.memory.mem_mut(addr as u64)
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
    // Register opcodes
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

    // Immediate opcodes
    Addi,
    Xori,
    Ori,
    Andi,
    Slli,
    Srli,
    Srai,
    Slti,
    Sltiu,

    // Load opcodes
    Lb,
    Lh,
    Lw,
    Lbu,
    Lhu
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

    #[test]
    fn test_memory_loading_le() {
        let mut vm = VM::init();

        let num = 4_u64;
        let num_bytes = num.to_le_bytes();

        // write to memory
        let addr = 0;
        for i in 0..8 {
            *vm.mem_mut(addr + i) = num_bytes[i];
        }

        // read from memory
        assert_eq!(vm.mem(addr), num);
    }
}
