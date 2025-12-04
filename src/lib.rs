use crate::decode::Instruction;
use crate::decode::Opcode;
use crate::memory::Memory;

mod decode;
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

    /// Reads 32 bytes from memory at the given addr
    /// assumes value at memory address is the LSB
    fn mem32(&self, addr: usize) -> u32 {
        let mut result = 0_u32;
        for i in 0..4 {
            let byte = self.memory.read((addr + i) as u64);
            result |= (byte as u32) << (i * 8);
        }
        result
    }

    /// Returns a mutable reference to a single byte at the given
    /// memory addr
    fn mem_mut(&mut self, addr: usize) -> &mut u8 {
        self.memory.mem_mut(addr as u64)
    }

    /// Write multiple bytes from a given address
    fn write_bytes(&mut self, addr: usize, data: &[u8]) {
        self.memory.write_bytes(addr as u64, data);
    }
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

        let bytes = [
            0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        // write to memory
        vm.write_bytes(0, &bytes);

        // read from memory
        assert_eq!(vm.mem(0), 4);
        assert_eq!(vm.mem(8), 10);
    }
}
