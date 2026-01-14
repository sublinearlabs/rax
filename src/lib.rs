use std::fs;

use crate::decode::compressed::decode_compressed;
use crate::elf::decode_elf;
use crate::memory::Memory;
use crate::trace::{DefaultTracer, Tracer};
use crate::util::{mask, mask16};
use decode::decode;

mod decode;
mod ecall;
mod elf;
mod execute;
mod memory;
pub mod trace;
mod util;

/// RISC-V Virtual Machine with configurable tracing.
///
/// The VM is generic over a `Tracer` type, enabling zero-cost abstraction:
/// - `NoopTracer`: All tracing calls are optimized away (zero overhead)
/// - `FullTracer`: Complete execution trace is captured
pub struct VM<T: Tracer = DefaultTracer> {
    registers: [u64; 32],
    f_reg: [u64; 32],
    memory: Memory,
    fcsr_reg: u32,
    reservation_set: u64,
    pc: u64,
    pub halted: bool,
    pub exit_code: u64,
    pub cycles: u64,
    pub elapsed: std::time::Duration,
    tracer: T,

    // std in
    pub input_stream: Vec<u8>,
    pub input_cursor: usize,
}

impl<T: Tracer> Default for VM<T> {
    fn default() -> Self {
        Self {
            registers: [0u64; 32],
            memory: Memory::default(),
            reservation_set: 0,
            pc: 0,
            halted: false,
            exit_code: 0,
            cycles: 0,
            elapsed: std::time::Duration::default(),
            tracer: T::default(),
            f_reg: [0u64; 32],
            fcsr_reg: 0,
            input_stream: Vec::new(),
            input_cursor: 0,
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

    /// Set input stream
    pub fn set_input_stream(&mut self, input: Vec<u8>) {
        self.input_stream = input;
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
        let start = std::time::Instant::now();
        while !self.halted {
            self.step();
        }
        self.elapsed = start.elapsed();
    }

    /// Execute with timing information
    pub fn run_with_timing(&mut self) {
        self.run();
        println!("run took: {:?}ms", self.elapsed.as_micros());
        println!("run took: {:?}s", self.elapsed.as_secs_f64());

        println!("cycles: {}", self.cycles);
        // cycles / microseconds = Mhz
        println!(
            "{:.2} Mhz",
            self.cycles as f64 / self.elapsed.as_micros() as f64
        )
    }

    /// Perform one cycle with tracing
    pub fn step(&mut self) {
        let insn = self.load_u16(self.pc as usize);
        let is_compressed = insn & mask16(2) != 0b11;

        let (insn, insn_bytes) = if is_compressed {
            (decode_compressed(insn), insn as u32)
        } else {
            let insn_upper = self.load_u16((self.pc + 2) as usize);
            let insn = (insn_upper as u32) << 16 | insn as u32;
            (decode(insn), insn)
        };

        // Begin tracing this instruction
        self.tracer.begin_instruction(
            self.cycles,
            self.pc,
            &self.registers,
            &self.f_reg,
            insn_bytes,
            &insn,
        );

        // Execute the instruction (this will update PC)
        self.execute_instruction(insn, is_compressed);

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
        self.tracer.finalize(self.registers, self.f_reg, self.pc)
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
    pub(crate) fn reg(&self, idx: u8) -> u64 {
        if idx == 0 {
            0
        } else {
            self.registers[idx as usize]
        }
    }

    /// Returns a mutable reference to the idx register
    pub(crate) fn reg_mut(&mut self, idx: u8, value: u64) {
        if idx == 0 {
            self.registers[idx as usize] = 0;
        } else {
            self.registers[idx as usize] = value;
        }
        self.tracer.record_rd(idx, value);
    }

    /// Returns the current value at the idx floating point register
    fn read_f64(&self, idx: u8) -> f64 {
        f64::from_bits(self.f_reg[idx as usize])
    }

    /// Updates idx floating point register to value
    fn write_f64(&mut self, idx: u8, value: f64) {
        let res = value.to_bits();
        self.f_reg[idx as usize] = res;
        self.tracer.record_rd(idx, res);
    }

    // Read f32
    fn read_f32(&self, idx: u8) -> f32 {
        let val = self.f_reg[idx as usize];
        if val >> 32 != 0xffff_ffff {
            // signal quiet
            return f32::from_bits(0x7FC0_0000);
        }
        f32::from_bits(val as u32)
    }

    // Write f32
    fn write_f32(&mut self, idx: u8, val: f32) {
        let res = 0xffff_ffff_0000_0000 | (val.to_bits() as u64);
        self.f_reg[idx as usize] = res;
        self.tracer.record_rd(idx, res);
    }

    /// Load 8 bytes from memory at the given addr
    /// assumes value at memory address is the LSB
    pub(crate) fn load_u64(&self, addr: usize) -> u64 {
        self.memory.read_u64(addr as u64)
    }

    /// Load 4 bytes from memory at the given addr
    /// assumes value at memory address is the LSB
    pub(crate) fn load_u32(&self, addr: usize) -> u32 {
        self.memory.read_u32(addr as u64)
    }

    /// Load 2 bytes from memory at the given addr
    /// assumes value at memory address is the LSB
    pub(crate) fn load_u16(&self, addr: usize) -> u16 {
        self.memory.read_u16(addr as u64)
    }

    /// Load 1 byte from memory at the given addr
    pub(crate) fn load_u8(&self, addr: usize) -> u8 {
        self.memory.read_u8(addr as u64)
    }

    /// Write 8 butes to memory at the given addr
    pub(crate) fn store_u64(&mut self, addr: usize, value: u64) {
        self.memory.write_u64(addr as u64, value);
    }

    /// Write 4 bytes to memory at the given addr
    pub(crate) fn store_u32(&mut self, addr: usize, value: u32) {
        self.memory.write_u32(addr as u64, value);
    }

    /// Write 2 bytes to memory at the given addr
    pub(crate) fn store_u16(&mut self, addr: usize, value: u16) {
        self.memory.write_u16(addr as u64, value);
    }

    /// Write 1 byte to memory at the given addr
    pub(crate) fn store_u8(&mut self, addr: usize, value: u8) {
        self.memory.write_u8(addr as u64, value);
    }

    /// Write multiple bytes from a given address
    pub fn write_bytes(&mut self, addr: usize, data: &[u8]) {
        self.memory.write_n_bytes(addr as u64, data);
    }

    /// Read multiple bytes from a given address
    pub(crate) fn read_bytes(&mut self, addr: usize, len: usize) -> Vec<u8> {
        self.memory.read_n_bytes(addr as u64, len)
    }

    fn read_csr(&self, csr: u32) -> u32 {
        match csr {
            // Read fflags
            0x1 => self.fcsr_reg & 0x1f,
            // Read frm
            0x2 => (self.fcsr_reg >> 5) & 0x7,
            // Read csr
            0x3 => self.fcsr_reg,
            _ => 0,
        }
    }

    fn set_csr(&mut self, csr: u32, val: u32) {
        match csr {
            // Set fflags
            0x1 => {
                self.fcsr_reg &= !0x1f;
                self.fcsr_reg |= val & 0x1f;
            }
            // Set Frm
            0x2 => {
                self.fcsr_reg &= !(0x7 << 5);
                self.fcsr_reg |= (val & 0x7) << 5;
            }
            // Set Csr
            0x3 => {
                self.fcsr_reg &= !0xff;
                self.fcsr_reg |= val & 0xff;
            }
            _ => {}
        }
        self.tracer.record_csr_reg(self.fcsr_reg);
    }

    fn raise_fflags_f32(&mut self, a: f32, b: f32, res: f32, op: char) {
        let mut flags = 0u32;

        // NV: Invalid operation
        // 1. Result is NaN but neither input was NaN
        if res.is_nan() && !a.is_nan() && !b.is_nan() {
            flags |= 0b10000;
        }

        // 2. Check for signaling NaN inputs (always invalid)
        if is_snan_f32(a) || is_snan_f32(b) {
            flags |= 0b10000;
        }

        // 3. Invalid subtraction: inf - inf or (-inf) - (-inf)
        if op == '-' && a.is_infinite() && b.is_infinite() && a.signum() == b.signum() {
            flags |= 0b10000;
        }

        // 4. Invalid addition: inf + (-inf) or (-inf) + inf
        if op == '+' && a.is_infinite() && b.is_infinite() && a.signum() != b.signum() {
            flags |= 0b10000;
        }

        // 5. Invalid multiplication: 0 * inf or inf * 0
        if op == '*' && ((a == 0.0 && b.is_infinite()) || (a.is_infinite() && b == 0.0)) {
            flags |= 0b10000;
        }

        // 6. Invalid division: 0/0 or inf/inf
        if op == '/' && ((a == 0.0 && b == 0.0) || (a.is_infinite() && b.is_infinite())) {
            flags |= 0b10000;
        }

        // DZ: Divide by zero (finite / 0)
        if op == '/' && b == 0.0 && !a.is_nan() && !a.is_infinite() && a != 0.0 {
            flags |= 0b01000;
        }

        // OF: Overflow (result is infinite but inputs weren't)
        if res.is_infinite() && !a.is_infinite() && !b.is_infinite() {
            flags |= 0b00100;
            flags |= 0b00001; // Overflow also sets inexact
        }

        // UF: Underflow (result is subnormal)
        if is_subnormal_f32(res) {
            flags |= 0b00010;
            flags |= 0b00001; // Underflow also sets inexact (usually)
        }

        // NX: Inexact
        if !res.is_nan() && !res.is_infinite() && !a.is_nan() && !b.is_nan() {
            let a64 = a as f64;
            let b64 = b as f64;
            let exact = match op {
                '+' => a64 + b64,
                '-' => a64 - b64,
                '*' => a64 * b64,
                '/' => {
                    if b64 != 0.0 {
                        a64 / b64
                    } else {
                        res as f64
                    }
                }
                _ => res as f64,
            };
            if exact != (res as f64) {
                flags |= 0b00001;
            }
        }

        self.fcsr_reg |= flags;
        self.tracer.record_csr_reg(self.fcsr_reg);
    }

    fn raise_fflags_f64(&mut self, a: f64, b: f64, res: f64, op: char) {
        let mut flags = 0u32;

        // NV: Invalid operation - result is NaN but neither input was NaN
        if res.is_nan() && !a.is_nan() && !b.is_nan() {
            flags |= 0b10000;
        }

        // Check for signaling NaN inputs
        if is_snan_f64(a) || is_snan_f64(b) {
            flags |= 0b10000;
        }

        // DZ: Divide by zero
        if op == '/' && b == 0.0 && !a.is_nan() && !b.is_nan() && !a.is_infinite() {
            flags |= 0b01000;
        }

        // OF: Overflow (result is infinite but inputs weren't)
        if res.is_infinite() && !a.is_infinite() && !b.is_infinite() {
            flags |= 0b00100;
            flags |= 0b00001; // Overflow also sets inexact
        }

        // UF: Underflow (result is subnormal)
        if is_subnormal_f64(res) {
            flags |= 0b00010;
        }

        // NX: Inexact - for f64, we can't easily use higher precision
        // Use a heuristic: check if result has full precision bits used
        // This is imperfect but catches many cases
        if !res.is_nan() && !res.is_infinite() {
            // For operations that are commonly inexact
            if op == '/' {
                // Division is often inexact unless result is exact
                let check = res * b;
                if check != a {
                    flags |= 0b00001;
                }
            }
            // For sqrt, add, sub, mul - harder to detect without f128
        }

        self.fcsr_reg |= flags;
        self.tracer.record_csr_reg(self.fcsr_reg);
    }

    fn raise_fflags_fma_f32(&mut self, a: f32, b: f32, c: f32, res: f32) {
        let mut flags = 0u32;

        // NV: Invalid operation
        // sNaN inputs
        if is_snan_f32(a) || is_snan_f32(b) || is_snan_f32(c) {
            flags |= 0b10000;
        }

        // 0 * inf or inf * 0
        if (a == 0.0 && b.is_infinite()) || (a.is_infinite() && b == 0.0) {
            flags |= 0b10000;
        }

        // inf + (-inf) in the addition part
        let mul_res = a * b;
        if mul_res.is_infinite() && c.is_infinite() && mul_res.signum() != c.signum() {
            flags |= 0b10000;
        }

        // Result is NaN but no input was NaN
        if res.is_nan() && !a.is_nan() && !b.is_nan() && !c.is_nan() {
            flags |= 0b10000;
        }

        // OF: Overflow
        if res.is_infinite() && !a.is_infinite() && !b.is_infinite() && !c.is_infinite() {
            flags |= 0b00100;
            flags |= 0b00001;
        }

        // UF: Underflow
        if is_subnormal_f32(res) {
            flags |= 0b00010;
            flags |= 0b00001;
        }

        // NX: Inexact - use f64 to check
        if !res.is_nan() && !res.is_infinite() && !a.is_nan() && !b.is_nan() && !c.is_nan() {
            let a64 = a as f64;
            let b64 = b as f64;
            let c64 = c as f64;
            let exact = a64.mul_add(b64, c64);
            if exact != (res as f64) {
                flags |= 0b00001;
            }
        }

        self.fcsr_reg |= flags;
        self.tracer.record_csr_reg(self.fcsr_reg);
    }

    fn raise_fflags_fma_f64(&mut self, a: f64, b: f64, c: f64, res: f64) {
        let mut flags = 0u32;

        // NV: Invalid operation
        if is_snan_f64(a) || is_snan_f64(b) || is_snan_f64(c) {
            flags |= 0b10000;
        }

        if (a == 0.0 && b.is_infinite()) || (a.is_infinite() && b == 0.0) {
            flags |= 0b10000;
        }

        let mul_res = a * b;
        if mul_res.is_infinite() && c.is_infinite() && mul_res.signum() != c.signum() {
            flags |= 0b10000;
        }

        if res.is_nan() && !a.is_nan() && !b.is_nan() && !c.is_nan() {
            flags |= 0b10000;
        }

        // OF: Overflow
        if res.is_infinite() && !a.is_infinite() && !b.is_infinite() && !c.is_infinite() {
            flags |= 0b00100;
            flags |= 0b00001;
        }

        // UF: Underflow
        if is_subnormal_f64(res) {
            flags |= 0b00010;
        }

        self.fcsr_reg |= flags;
        self.tracer.record_csr_reg(self.fcsr_reg);
    }
}

fn is_snan_f32(val: f32) -> bool {
    let bits = val.to_bits();
    let exp = (bits >> 23) & 0xFF;
    let frac = bits & 0x7FFFFF;
    exp == 0xFF && frac != 0 && (frac & 0x400000) == 0
}

fn is_subnormal_f32(val: f32) -> bool {
    let bits = val.to_bits();
    let exp = (bits >> 23) & 0xFF;
    let frac = bits & 0x7FFFFF;
    exp == 0 && frac != 0
}

fn is_snan_f64(val: f64) -> bool {
    let bits = val.to_bits();
    let exp = (bits >> 52) & 0x7FF;
    let frac = bits & 0xFFFFFFFFFFFFF;
    // Signaling NaN: exponent all 1s, fraction non-zero, quiet bit (bit 51) is 0
    exp == 0x7FF && frac != 0 && (frac & 0x8000000000000) == 0
}

fn is_subnormal_f64(val: f64) -> bool {
    let bits = val.to_bits();
    let exp = (bits >> 52) & 0x7FF;
    let frac = bits & 0xFFFFFFFFFFFFF;
    exp == 0 && frac != 0
}

#[cfg(test)]
mod tests {
    use crate::trace::{FullTracer, NoopTracer};

    use super::*;

    /// VM with no tracing (zero overhead)
    pub type FastVM = VM<NoopTracer>;

    /// VM with full execution tracing
    pub type TracingVM = VM<FullTracer>;

    #[test]
    fn test_register_read_write() {
        let mut vm = VM::<NoopTracer>::init();

        // read
        assert_eq!(vm.reg(5), 0);
        // write
        vm.reg_mut(5, 10);
        assert_eq!(vm.reg(5), 10);
        // write
        vm.reg_mut(5, 20);
        assert_eq!(vm.reg(5), 20);
    }

    #[test]
    fn test_register_0_always_0() {
        let mut vm = VM::<NoopTracer>::init();
        // read register 0
        assert_eq!(vm.reg(0), 0);
        // write to register 0
        vm.reg_mut(0, 20);
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
        assert_eq!(vm.load_u64(0), 4);
        assert_eq!(vm.load_u64(8), 10);
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
        vm.reg_mut(1, 1);
        vm.reg_mut(2, 1);

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
        vm.reg_mut(1, 1);
        vm.reg_mut(2, 1);

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

    #[test]
    fn test_round_std_io() {
        // Path to the echo guest program built for the test environment.
        // If the binary is not present, skip the test to avoid failing CI for missing artifacts.
        let echo_bin = "rust-bin/echo/target/riscv64ima-unknown-none-elf/release/echo".to_string();
        if fs::metadata(&echo_bin).is_err() {
            eprintln!("Skipping test_round_std_io: {} not found", echo_bin);
            return;
        }

        // Initialize the VM from the echo ELF and provide some stdin.
        let mut vm = VM::<NoopTracer>::init_from_elf(echo_bin);
        vm.input_stream = "Hola Riscv, buenos días".as_bytes().to_vec();
        vm.input_cursor = 0;

        // Run the guest program; it should echo the input and then exit via ecall.
        vm.run();

        // Verify the VM halted and exited successfully.
        assert!(vm.halted);
        assert_eq!(vm.exit_code, 0);
    }
}
