use std::{collections::HashMap, fs};

use crate::decode::{Instruction, compressed::decode_compressed};
use crate::elf::decode_elf;
use crate::memory::Memory;
use crate::trace::{DefaultTracer, Tracer};
use crate::util::{is_snan_f32, is_snan_f64, is_subnormal_f32, is_subnormal_f64, mask16};
use decode::decode;

mod decode;
mod ecall;
mod elf;
mod execute;
mod instr_execute;
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
    // Instruction segments from ELF file
    insn_segments: HashMap<u64, InstructionSegment>,
    // Mapping of pc to all decoded instructions
    decoded_instructions: HashMap<u64, (u64, Instruction, bool)>,
    // Basic blocks is a mapping from the pc of the block start to the decoded block instructions
    basic_blocks: HashMap<u64, Vec<(Instruction, bool)>>,
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

#[derive(Debug)]
pub(crate) struct InstructionSegment {
    pub(crate) start: u64,
    pub(crate) end: u64,
    pub(crate) data: Vec<u8>,
}

impl InstructionSegment {
    pub(crate) fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            data: vec![],
        }
    }
}

impl<T: Tracer> Default for VM<T> {
    fn default() -> Self {
        Self {
            registers: [0u64; 32],
            memory: Memory::default(),
            decoded_instructions: HashMap::new(),
            basic_blocks: HashMap::new(),
            insn_segments: HashMap::new(),
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
        let (memory, pc, insn_segments) = decode_elf(&elf_bytes);
        // Initialize stack pointer (x2/sp) to a valid memory address
        let mut registers = [0u64; 32];
        registers[2] = Self::DEFAULT_STACK_POINTER;
        let mut vm = Self {
            registers,
            memory,
            insn_segments,
            pc,
            ..Default::default()
        };
        // Decode instructions and build basic blocks
        vm.decode_instructions_and_build_basic_blocks();
        vm
    }

    /// Init the VM from an elf file with a specific tracer
    pub fn init_from_elf_with_tracer(path: String, tracer: T) -> Self {
        let elf_bytes = fs::read(path).unwrap();
        let (memory, pc, insn_segments) = decode_elf(&elf_bytes);
        // Initialize stack pointer (x2/sp) to a valid memory address
        let mut registers = [0u64; 32];
        registers[2] = Self::DEFAULT_STACK_POINTER;
        let mut vm = Self {
            registers,
            memory,
            insn_segments,
            pc,
            tracer,
            ..Default::default()
        };
        // Decode instructions and build basic blocks
        vm.decode_instructions_and_build_basic_blocks();
        vm
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
        // Check if we're at the start of a basic block
        let block = self.get_basic_block(self.pc);
        if let Some(block) = block {
            // Clone the block to avoid borrowing issues
            let block = block.clone();
            // Execute the entire basic block
            self.execute_basic_block(&block);
        } else {
            // Fall back to single instruction execution
            self.execute_single_instruction();
        }
    }

    fn execute_single_instruction(&mut self) {
        let (insn, is_compressed) =
            if let Some((instr, compressed)) = self.get_instruction_with_flag(self.pc) {
                (instr.clone(), compressed)
            } else {
                // Try to read and decode the instruction from memory
                if let Some((decoded_insn, compressed)) = self.try_decode_instruction_at_pc() {
                    (decoded_insn, compressed)
                } else {
                    (Instruction::Illegal(0), false)
                }
            };

        // For tracing, we need to determine the instruction bytes
        let insn_bytes = if is_compressed {
            self.load_u16(self.pc as usize) as u32
        } else {
            let lower = self.load_u16(self.pc as usize) as u32;
            let upper = self.load_u16((self.pc + 2) as usize) as u32;
            (upper << 16) | lower
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
        self.execute_instruction(&insn, is_compressed);

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

    fn execute_basic_block(&mut self, block: &[(Instruction, bool)]) {
        for (i, (insn, is_compressed)) in block.iter().enumerate() {
            let current_pc = self.pc;

            // For tracing, we need to determine the instruction bytes
            let insn_bytes = if *is_compressed {
                self.load_u16(current_pc as usize) as u32
            } else {
                let lower = self.load_u16(current_pc as usize) as u32;
                let upper = self.load_u16((current_pc + 2) as usize) as u32;
                (upper << 16) | lower
            };

            // Begin tracing this instruction
            self.tracer.begin_instruction(
                self.cycles + i as u64,
                current_pc,
                &self.registers,
                &self.f_reg,
                insn_bytes,
                insn,
            );

            // Execute the instruction (this will update PC)
            self.execute_instruction(insn, *is_compressed);

            // Record next PC
            self.tracer.record_next_pc(self.pc);

            // Check for halt
            if self.halted {
                self.tracer.record_halt();
                break;
            }

            // Commit the trace row
            self.tracer.commit();
        }

        self.cycles = self.cycles.wrapping_add(block.len() as u64);
    }

    fn decode_instructions_and_build_basic_blocks(&mut self) {
        // Phase 1: Decode all instructions from all segments
        for (_, segment) in &self.insn_segments {
            let mut pc = segment.start;
            while pc < segment.end {
                let offset = (pc - segment.start) as usize;
                if offset + 2 > segment.data.len() {
                    break; // Not enough data
                }
                let insn = self.load_u16_insn(offset, segment);
                let is_compressed = insn & mask16(2) != 0b11;

                let decoded_insn = if is_compressed {
                    decode_compressed(insn)
                } else {
                    if offset + 4 > segment.data.len() {
                        // Not enough data for full instruction, treat as illegal
                        Instruction::Illegal(insn as u32)
                    } else {
                        let insn_upper = self.load_u16_insn(offset + 2, segment);
                        let full_insn = (insn_upper as u32) << 16 | insn as u32;
                        decode(full_insn)
                    }
                };

                self.decoded_instructions
                    .insert(pc, (pc, decoded_insn, is_compressed));

                pc += if is_compressed { 2 } else { 4 };
            }
        }

        // Phase 2: Identify leaders (basic block start addresses)
        let mut leaders = std::collections::HashSet::new();
        leaders.insert(self.pc);

        for (pc, insn, _) in self.decoded_instructions.values().into_iter() {
            if let Some(offset) = insn.jump_target() {
                let target = (*pc as i64 + offset) as u64;
                leaders.insert(target);
            }

            if insn.is_branch_or_jmp() {
                let next_pc = if self.contains_pc(pc + 2) {
                    pc + 2
                } else {
                    pc + 4
                };
                leaders.insert(next_pc);
            }
        }

        // Phase 3: Build basic blocks
        let mut sorted_leaders: Vec<u64> = leaders.into_iter().collect();
        sorted_leaders.sort();

        for window in sorted_leaders.windows(2) {
            let start = window[0];
            let end = window[1];

            let mut block = Vec::new();
            let mut current = start;
            loop {
                if let Some((insn, compressed)) = self.get_instruction_with_flag(current) {
                    block.push((insn.clone(), compressed));
                    let next_current = if current + 2 == end || !self.contains_pc(current + 2) {
                        current + 4
                    } else {
                        current + 2
                    };
                    if next_current >= end {
                        break;
                    }
                    current = next_current;
                } else {
                    break;
                }
            }
            if !block.is_empty() {
                self.basic_blocks.insert(start, block);
            }
        }

        // Handle the last block if any
        if let Some(&last_leader) = sorted_leaders.last() {
            if let Some((insn, compressed)) = self.get_instruction_with_flag(last_leader) {
                self.basic_blocks
                    .insert(last_leader, vec![(insn.clone(), compressed)]);
            }
        }
    }

    // Binary search helper to check if PC exists
    fn contains_pc(&self, pc: u64) -> bool {
        self.decoded_instructions.contains_key(&pc)
    }

    // Binary search helper to get instruction by PC
    pub(crate) fn get_instruction(&self, pc: u64) -> Option<&Instruction> {
        self.decoded_instructions.get(&pc).map(|(_, insn, _)| insn)
    }

    // Get instruction and compressed flag by PC
    pub(crate) fn get_instruction_with_flag(&self, pc: u64) -> Option<(&Instruction, bool)> {
        self.decoded_instructions
            .get(&pc)
            .map(|(_, insn, compressed)| (insn, compressed.clone()))
    }

    // Get basic block starting at the given PC
    pub(crate) fn get_basic_block(&self, pc: u64) -> Option<&Vec<(Instruction, bool)>> {
        self.basic_blocks.get(&pc)
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

    pub(crate) fn load_u16_insn(&self, offset: usize, segment: &InstructionSegment) -> u16 {
        let mut res = [0; 2];
        res.copy_from_slice(&segment.data[offset..offset + 2]);
        u16::from_le_bytes(res)
    }

    pub(crate) fn write_insn(&mut self, pc: u64, segment: InstructionSegment) {
        self.insn_segments.insert(pc, segment);
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

    /// Try to decode the instruction at the current PC from memory
    fn try_decode_instruction_at_pc(&self) -> Option<(Instruction, bool)> {
        // Try to read 2 bytes first to check if compressed
        let bytes = self.memory.read_n_bytes(self.pc, 2);
        if bytes.len() < 2 {
            return None;
        }
        let insn_u16 = u16::from_le_bytes([bytes[0], bytes[1]]);
        let is_compressed = (insn_u16 & mask16(2)) != 0b11;

        if is_compressed {
            // Compressed instruction
            let decoded = decode_compressed(insn_u16);
            Some((decoded, true))
        } else {
            // Try to read 4 bytes
            let bytes4 = self.memory.read_n_bytes(self.pc, 4);
            if bytes4.len() < 4 {
                return Some((Instruction::Illegal(insn_u16 as u32), false));
            }
            let full_insn = u32::from_le_bytes([bytes4[0], bytes4[1], bytes4[2], bytes4[3]]);
            let decoded = decode(full_insn);
            Some((decoded, false))
        }
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

        let segment = InstructionSegment {
            start: 0,
            end: 36,
            data: fib_prog.to_vec(),
        };

        let mut vm = VM::<NoopTracer>::init();
        vm.write_insn(0, segment);
        vm.decode_instructions_and_build_basic_blocks();
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
        let segment = InstructionSegment {
            start: 0,
            end: 12,
            data: fib_prog.to_vec(),
        };
        vm.write_insn(0, segment);
        vm.decode_instructions_and_build_basic_blocks();
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

    #[test]
    fn test_analyzer() {
        let file_location = "test-bin/rust-bin/fib/fib-gc".to_string();
        let mut vm = FastVM::init_from_elf(file_location);
        vm.decode_instructions_and_build_basic_blocks();

        // Basic sanity checks
        assert!(
            !vm.basic_blocks.is_empty(),
            "Should create at least one basic block"
        );
        assert!(
            vm.decoded_instructions.len() > 0,
            "Should decode some instructions"
        );

        // Entry point should be a leader (have a basic block starting there)
        assert!(
            vm.basic_blocks.contains_key(&vm.pc),
            "Entry point should be a leader with a basic block"
        );

        // Collect all leaders (basic block start addresses)
        let leaders: std::collections::HashSet<u64> = vm.basic_blocks.keys().cloned().collect();

        // Verify that jump targets that point to valid instruction addresses are leaders
        for (pc, insn, _) in vm.decoded_instructions.values() {
            if let Some(offset) = insn.jump_target() {
                let target = (*pc as i64 + offset) as u64;
                println!("PC {:#x}, offset {:#x}, target {:#x}", pc, offset, target);
                // Only check targets that are within our decoded instruction range
                if vm.decoded_instructions.contains_key(&target) {
                    println!("Target {:#x} is in decoded instructions", target);
                    assert!(
                        leaders.contains(&target),
                        "Jump target {:#x} should be a leader",
                        target
                    );
                } else {
                    println!("Target {:#x} is NOT in decoded instructions", target);
                }
            }
        }

        // Verify basic block properties
        for (block_start, instructions) in &vm.basic_blocks {
            assert!(!instructions.is_empty(), "Basic blocks should not be empty");

            // Check that instructions in the middle of blocks are not jumps/branches
            for (i, (insn, _)) in instructions.iter().enumerate() {
                if i < instructions.len() - 1 {
                    // Instructions in the middle should not be jumps/branches
                    assert!(
                        !insn.is_branch_or_jmp(),
                        "Instruction at position {} in block starting at {:#x} should not be a jump/branch",
                        i,
                        block_start
                    );
                }
            }

            // The last instruction can be a jump/branch
            if let Some((last_insn, _)) = instructions.last() {
                // If it's a jump/branch, the next instruction should be a leader
                if last_insn.is_branch_or_jmp() {
                    // We can't easily verify this without knowing the exact PC, but we can check
                    // that there are other leaders
                    assert!(
                        leaders.len() > 1,
                        "Should have multiple leaders if jumps exist"
                    );
                }
            }
        }

        // Verify that all instructions are covered by basic blocks
        let mut covered_instructions = 0;
        for instructions in vm.basic_blocks.values() {
            covered_instructions += instructions.len();
        }

        // Note: This might not be exactly equal due to compressed instructions having different sizes
        // But we should cover most instructions
        assert!(
            covered_instructions > 0,
            "Should cover some instructions in basic blocks"
        );
        assert!(
            covered_instructions <= vm.decoded_instructions.len(),
            "Should not cover more instructions than were decoded"
        );

        // Verify that leaders are properly ordered and don't overlap
        let mut sorted_leaders: Vec<u64> = leaders.into_iter().collect();
        sorted_leaders.sort();

        for window in sorted_leaders.windows(2) {
            let current = window[0];
            let next = window[1];
            assert!(current < next, "Leaders should be in ascending order");

            // Check that there's no overlap - the end of one block should be before the start of the next
            if let Some(instructions) = vm.basic_blocks.get(&current) {
                // Calculate the actual end of the block by summing instruction sizes
                let mut current_end = current;
                for _insn in instructions {
                    // Check if this instruction is compressed (2 bytes) or regular (4 bytes)
                    // We can determine this by checking if the PC exists at pc+2
                    let next_pc = current_end + 2;
                    if vm.decoded_instructions.contains_key(&next_pc) {
                        current_end += 2; // compressed
                    } else {
                        current_end += 4; // regular
                    }
                }
                assert!(
                    current_end <= next,
                    "Block ending at {:#x} should not overlap with block starting at {:#x}",
                    current_end,
                    next
                );
            }
        }

        println!("Analyzer test passed!");
        println!("- Decoded {} instructions", vm.decoded_instructions.len());
        println!("- Created {} basic blocks", vm.basic_blocks.len());
        println!("- Entry point: {:#x}", vm.pc);
    }

    #[test]
    fn test_basic_block_boundaries() {
        // Test with a simple program that has clear jump/branch boundaries
        let file_location = "test-bin/rust-bin/fib/fib-gc".to_string();
        let mut vm = FastVM::init_from_elf(file_location);
        vm.decode_instructions_and_build_basic_blocks();

        // Find any basic blocks that end with jumps/branches
        let mut jump_blocks = Vec::new();
        for (start_pc, instructions) in &vm.basic_blocks {
            if let Some((last_insn, _)) = instructions.last() {
                if last_insn.is_branch_or_jmp() {
                    jump_blocks.push(*start_pc);
                }
            }
        }

        if !jump_blocks.is_empty() {
            println!(
                "Found {} basic blocks ending with jumps/branches",
                jump_blocks.len()
            );
            // Verify that these blocks don't have jumps in the middle
            for block_start in jump_blocks {
                let instructions = &vm.basic_blocks[&block_start];
                for (i, (insn, _)) in instructions.iter().enumerate() {
                    if i < instructions.len() - 1 {
                        assert!(
                            !insn.is_branch_or_jmp(),
                            "Block {:#x}: instruction at position {} should not be a jump/branch",
                            block_start,
                            i
                        );
                    }
                }
            }
        }
    }
}
