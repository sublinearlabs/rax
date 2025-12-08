use std::fs;

use crate::decode::Instruction;
use crate::decode::Opcode;
use crate::decode::decode_insn;
use crate::elf::decode_elf;
use crate::memory::Memory;

mod decode;
mod elf;
mod execute;
mod memory;
mod util;

#[derive(Default)]
struct VM {
    registers: [u64; 32],
    memory: Memory,
    x0_sink: u64, // blackhole for writes to x0
    pc: u64,
    halted: bool,
    exit_code: u64,
    cycles: u64,
}

impl VM {
    /// Returns a VM with empty state
    fn init() -> Self {
        Self::default()
    }

    /// Init the VM from an elf file
    fn init_from_elf(path: String) -> Self {
        let elf_bytes = fs::read(path).unwrap();
        let (memory, pc) = decode_elf(&elf_bytes);
        Self {
            memory,
            pc,
            ..Default::default()
        }
    }

    /// execute the vm
    fn run(&mut self) {
        while !self.halted {
            self.step();
        }
    }

    fn run_with_timing(&mut self) {
        let start = std::time::Instant::now();
        self.run();
        let end = start.elapsed();
        println!("run took: {:?}ms", end.as_micros());
        println!("cycles: {}", self.cycles);
        // cycles / microseconds = Mhz
        println!("{:.2} Mhz", self.cycles as f64 / end.as_micros() as f64)
    }

    /// perform one cycle
    fn step(&mut self) {
        // print!("{:x}: ", self.pc);
        let raw_insn = self.mem32(self.pc as usize);
        let insn = decode_insn(raw_insn);
        print!(" {:?}\n", insn.opcode);
        self.execute_instruction(insn);
        self.cycles = self.cycles.wrapping_add(1);
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
            // print!("{:x} ", byte);
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
    fn test_rv64ui() {
        let _ = fs::read_dir("test-bin/rv64ui")
            .expect("Failed to read directory")
            .filter_map(|entry| entry.ok())
            .map(|entry| run_test_elf(entry.path().to_str().unwrap().to_string()))
            .collect::<Vec<_>>();
    }

    #[test]
    fn test_rv64um() {
        let _ = fs::read_dir("test-bin/rv64um")
            .expect("Failed to read directory")
            .filter_map(|entry| entry.ok())
            .map(|entry| run_test_elf(entry.path().to_str().unwrap().to_string()))
            .collect::<Vec<_>>();
    }

    fn run_test_elf(path: String) {
        println!("running test: {path}");

        let mut vm = VM::init_from_elf(path);
        vm.run();

        println!("exit_code {}", vm.exit_code);
        assert!(vm.halted);
        assert_eq!(vm.exit_code, 0);
    }

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

    #[test]
    fn test_instruction_loading() {
        let fib_prog = [
            // Fib Step 0
            0xb3, 0x81, 0x20, 0x00, // add x3, x1, x2
            0xb3, 0x00, 0x01, 0x00, // add x1, x2, x0
            0x33, 0x81, 0x01, 0x00, // add x2, x3, x0
            // Fib Step 1
            0xb3, 0x81, 0x20, 0x00, // add x3, x1, x2
            0xb3, 0x00, 0x01, 0x00, // add x1, x2, x0
            0x33, 0x81, 0x01, 0x00, // add x2, x3, x0
            // Fib Step 2
            0xb3, 0x81, 0x20, 0x00, // add x3, x1, x2
            0xb3, 0x00, 0x01, 0x00, // add x1, x2, x0
            0x33, 0x81, 0x01, 0x00, // add x2, x3, x0
        ];

        let mut vm = VM::init();
        vm.write_bytes(0, &fib_prog);
        *vm.reg_mut(1) = 1;
        *vm.reg_mut(2) = 1;

        vm.step();
        vm.step();
        vm.step();

        assert_eq!(vm.reg(1), 1);
        assert_eq!(vm.reg(2), 2);

        vm.step();
        vm.step();
        vm.step();

        assert_eq!(vm.reg(1), 2);
        assert_eq!(vm.reg(2), 3);

        vm.step();
        vm.step();
        vm.step();

        assert_eq!(vm.reg(1), 3);
        assert_eq!(vm.reg(2), 5);

        assert_eq!(vm.cycles, 9);
    }
}
