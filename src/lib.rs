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
    f_reg: [u64; 32],
    memory: Memory,
    fcsr_reg: u32,
    x0_sink: u64, // blackhole for writes to x0
    reservation_set: u64,
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
        // print!(" {:?}, addr: {:0x}\n", insn.opcode, self.pc);
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

    /// Returns the current value at the idx floating point register
    fn read_f64(&self, idx: usize) -> f64 {
        f64::from_bits(self.f_reg[idx])
    }

    /// Returns a mutable reference to the idx floating point register
    fn write_f64(&mut self, idx: usize, value: f64) {
        self.f_reg[idx] = value.to_bits();
    }

    // Read f32
    fn read_f32(&self, idx: usize) -> f32 {
        let val = self.f_reg[idx];
        if val >> 32 != 0xffff_ffff {
            // signal quiet
            return f32::from_bits(0x7FC0_0000);
        }
        f32::from_bits(val as u32)
    }

    // Write f32
    fn write_f32(&mut self, idx: usize, val: f32) {
        self.f_reg[idx] = 0xffff_ffff_0000_0000 | (val.to_bits() as u64);
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

        // run_test_elf("test-bin/rv64ua/rv64ua-p-amomax_d".to_string());
    }

    #[test]
    fn test_rv64uf() {
        let _ = fs::read_dir("test-bin/rv64uf")
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
        if vm.exit_code != 0 {
            println!("failing test {}", vm.exit_code >> 1);
        }
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
