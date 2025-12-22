use std::fs;

use crate::elf::decode_elf;
use crate::memory::Memory;
use crate::trace::{DefaultTracer, FullTracer, NoopTracer, Tracer};
use decode_old::{Instruction, Opcode, decode_insn};

mod decode_old;
mod elf;
mod execute;
mod memory;
pub mod trace;
mod util;

/// RISC-V Virtual Machine with configurable tracing.
///
/// The VM is generic over a `Tracer` type, enabling zero-cost abstraction:
/// - `NoopTracer`: All tracing calls are optimized away ()
/// - `FullTracer`: Complete execution trace is captured
pub struct VM<T: Tracer = DefaultTracer> {
    registers: [u64; 32],
    memory: Memory,
    x0_sink: u64, // blackhole for writes to x0
    reservation_set: u64,
    pc: u64,
    halted: bool,
    exit_code: u64,
    cycles: u64,
    tracer: T,
}

impl<T: Tracer> Default for VM<T> {
    fn default() -> Self {
        Self {
            registers: [0u64; 32],
            memory: Memory::default(),
            x0_sink: 0,
            reservation_set: 0,
            pc: 0,
            halted: false,
            exit_code: 0,
            cycles: 0,
            tracer: T::default(),
        }
    }
}

impl<T: Tracer> VM<T> {
    /// Returns a VM with empty state
    pub fn init() -> Self {
        Self::default()
    }

    /// Default stack pointer address (128 MB)
    const DEFAULT_STACK_POINTER: u64 = 0x0800_0000;

    /// Init the VM from an elf file
    pub fn init_from_elf(path: String) -> Self {
        let elf_bytes = fs::read(path).unwrap();
        let (memory, pc) = decode_elf(&elf_bytes);
        // Initialize stack pointer (x2/sp) to a valid memory address
        let mut registers = [0u64; 32];
        registers[2] = Self::DEFAULT_STACK_POINTER;
        Self {
            registers,
            memory,
            pc,
            ..Default::default()
        }
    }

    /// Init the VM from an elf file with a specific tracer
    pub fn init_from_elf_with_tracer(path: String, tracer: T) -> Self {
        let elf_bytes = fs::read(path).unwrap();
        let (memory, pc) = decode_elf(&elf_bytes);
        // Initialize stack pointer (x2/sp) to a valid memory address
        let mut registers = [0u64; 32];
        registers[2] = Self::DEFAULT_STACK_POINTER;
        Self {
            registers,
            memory,
            pc,
            tracer,
            ..Default::default()
        }
    }

    /// Set a custom tracer
    pub fn with_tracer(mut self, tracer: T) -> Self {
        self.tracer = tracer;
        self
    }

    /// Get a reference to the tracer
    pub fn tracer(&self) -> &T {
        &self.tracer
    }

    /// Get a mutable reference to the tracer
    pub fn tracer_mut(&mut self) -> &mut T {
        &mut self.tracer
    }

    /// Execute the VM until halted
    pub fn run(&mut self) {
        while !self.halted {
            self.step();
        }
    }

    /// Execute with timing information
    pub fn run_with_timing(&mut self) {
        let start = std::time::Instant::now();
        self.run();
        let end = start.elapsed();
        println!("run took: {:?}ms", end.as_micros());
        println!("cycles: {}", self.cycles);
        // cycles / microseconds = Mhz
        println!("{:.2} Mhz", self.cycles as f64 / end.as_micros() as f64)
    }

    /// Perform one cycle with tracing
    pub fn step(&mut self) {
        let raw_insn = self.mem32(self.pc as usize);
        let insn = decode_insn(raw_insn);

        // Begin tracing this instruction
        self.tracer
            .begin_instruction(self.cycles, self.pc, &self.registers, raw_insn, &insn);

        // Execute the instruction (this will update PC)
        self.execute_instruction(insn);

        // Record next PC (set during execute_instruction or default to pc+4)
        self.tracer.record_next_pc(self.pc);

        // Check for halt
        if self.halted {
            self.tracer.record_halt();
        }

        // Commit the trace row
        self.tracer.commit();

        self.cycles = self.cycles.wrapping_add(1);
    }

    /// Finalize tracing and return the execution trace
    ///
    /// Returns `Some(ExecutionTrace)` if tracing was enabled, `None` otherwise.
    pub fn take_trace(self) -> Option<crate::trace::ExecutionTrace> {
        self.tracer.finalize(self.registers, self.pc)
    }

    /// Check if tracing is active
    pub fn is_tracing(&self) -> bool {
        self.tracer.is_active()
    }

    /// Get the current cycle count
    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    /// Get the current PC
    pub fn pc(&self) -> u64 {
        self.pc
    }

    /// Check if the VM has halted
    pub fn halted(&self) -> bool {
        self.halted
    }

    /// Get the exit code
    pub fn exit_code(&self) -> u64 {
        self.exit_code
    }

    /// Returns the current value at the idx register
    pub(crate) fn reg(&self, idx: usize) -> u64 {
        if idx == 0 { 0 } else { self.registers[idx] }
    }

    /// Returns a mutable reference to the idx register
    pub(crate) fn reg_mut(&mut self, idx: usize) -> &mut u64 {
        if idx == 0 {
            &mut self.x0_sink
        } else {
            &mut self.registers[idx]
        }
    }

    /// Reads 64 bytes from memory at the given addr
    /// assumes value at memory address is the LSB
    pub(crate) fn mem(&self, addr: usize) -> u64 {
        let mut result = 0_u64;
        for i in 0..8 {
            let byte = self.memory.read((addr + i) as u64);
            result |= (byte as u64) << (i * 8);
        }
        result
    }

    /// Reads 32 bytes from memory at the given addr
    /// assumes value at memory address is the LSB
    pub(crate) fn mem32(&self, addr: usize) -> u32 {
        let mut result = 0_u32;
        for i in 0..4 {
            let byte = self.memory.read((addr + i) as u64);
            result |= (byte as u32) << (i * 8);
        }
        result
    }

    /// Returns a mutable reference to a single byte at the given
    /// memory addr
    pub(crate) fn mem_mut(&mut self, addr: usize) -> &mut u8 {
        self.memory.mem_mut(addr as u64)
    }

    /// Write multiple bytes from a given address
    #[cfg(test)]
    pub(crate) fn write_bytes(&mut self, addr: usize, data: &[u8]) {
        self.memory.write_bytes(addr as u64, data);
    }
}

/// VM with no tracing (zero overhead)
pub type FastVM = VM<NoopTracer>;

/// VM with full execution tracing
pub type TracingVM = VM<FullTracer>;

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

    #[test]
    fn test_rv64ua() {
        let _ = fs::read_dir("test-bin/rv64ua")
            .expect("Failed to read directory")
            .filter_map(|entry| entry.ok())
            .map(|entry| run_test_elf(entry.path().to_str().unwrap().to_string()))
            .collect::<Vec<_>>();
    }

    #[test]
    fn test_rust_fib() {
        run_test_elf("rust-bin/fib/target/riscv64ima-unknown-none-elf/release/fib".to_string());
    }

    fn run_test_elf(path: String) {
        println!("running test: {path}");

        let mut vm = VM::<NoopTracer>::init_from_elf(path);
        vm.run();

        println!("exit_code {}", vm.exit_code);
        assert!(vm.halted);
        assert_eq!(vm.exit_code, 0);
    }

    #[test]
    fn test_register_read_write() {
        let mut vm = VM::<NoopTracer>::init();

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
        let mut vm = VM::<NoopTracer>::init();
        // read register 0
        assert_eq!(vm.reg(0), 0);
        // write to register 0
        *vm.reg_mut(0) = 20;
        assert_eq!(vm.reg(0), 0);
    }

    #[test]
    fn test_memory_loading_le() {
        let mut vm = VM::<NoopTracer>::init();

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

        let mut vm = VM::<NoopTracer>::init();
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

    #[test]
    fn test_tracing_vm() {
        let fib_prog = [
            0xb3, 0x81, 0x20, 0x00, // add x3, x1, x2
            0xb3, 0x00, 0x01, 0x00, // add x1, x2, x0
            0x33, 0x81, 0x01, 0x00, // add x2, x3, x0
        ];

        let mut vm = TracingVM::init();
        vm.write_bytes(0, &fib_prog);
        *vm.reg_mut(1) = 1;
        *vm.reg_mut(2) = 1;

        assert!(vm.is_tracing());

        vm.step();
        vm.step();
        vm.step();

        let trace = vm.take_trace().expect("Should have trace");

        assert_eq!(trace.rows.len(), 3);
        assert_eq!(trace.total_cycles, 3);
    }

    #[test]
    fn test_fast_vm_no_trace() {
        let vm = FastVM::init();
        assert!(!vm.is_tracing());
        assert!(vm.take_trace().is_none());
    }
}
