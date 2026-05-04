//! x86-64 ELF segment representation
//!
//! This module provides structures for representing x86-64 code segments

use crate::{
    aot::register_mapping::RegisterMapping, elf::Elf, translate::translator::RiscvToX86Translator,
};

/// x86-64 code segment
#[derive(Debug, Clone)]
pub struct X86Segment {
    /// Raw x86-64 bytecode
    pub data: Vec<u8>,

    /// Virtual address where this segment is loaded
    pub vaddr: u64,

    /// File offset where this segment starts
    pub offset: usize,

    /// Size in the file
    pub file_size: usize,

    /// Size in memory (may be larger if there's uninitialized data)
    pub mem_size: usize,

    /// Whether this segment is executable
    pub is_executable: bool,

    /// Whether this segment is writable
    pub is_writable: bool,

    /// Whether this segment is readable
    pub is_readable: bool,
}

impl X86Segment {
    /// Create a new x86-64 segment
    pub fn new(
        data: Vec<u8>,
        vaddr: u64,
        offset: usize,
        file_size: usize,
        mem_size: usize,
        is_executable: bool,
        is_writable: bool,
        is_readable: bool,
    ) -> Self {
        Self {
            data,
            vaddr,
            offset,
            file_size,
            mem_size,
            is_executable,
            is_writable,
            is_readable,
        }
    }

    /// Create an executable code segment (.text)
    pub fn text(bytecode: Vec<u8>, vaddr: u64, offset: usize) -> Self {
        let file_size = bytecode.len();
        Self::new(
            bytecode, vaddr, offset, file_size, file_size, true,  // executable
            false, // not writable
            true,  // readable
        )
    }

    /// Create a data segment (.rodata)
    pub fn data(data: Vec<u8>, vaddr: u64, offset: usize) -> Self {
        let file_size = data.len();
        Self::new(
            data, vaddr, offset, file_size, file_size, false, // not executable
            false, // not writable
            true,  // readable
        )
    }

    /// Create a BSS segment (.bss) - uninitialized data
    pub fn bss(mem_size: usize, vaddr: u64, offset: usize) -> Self {
        Self::new(
            Vec::new(), // Empty data, BSS is uninitialized
            vaddr,
            offset,
            0, // No file size for BSS
            mem_size,
            false, // not executable
            true,  // writable
            true,  // readable
        )
    }
}

/// x86-64 ELF representation with multiple segments
#[derive(Debug)]
pub struct X86Elf {
    /// All segments in this ELF
    pub segments: Vec<X86Segment>,

    /// Entry point address
    pub entry_point: u64,
}

impl X86Elf {
    /// Create a new x86-64 ELF
    pub fn new(entry_point: u64) -> Self {
        Self {
            segments: Vec::new(),
            entry_point,
        }
    }

    /// Add a segment
    pub fn add_segment(&mut self, segment: X86Segment) {
        self.segments.push(segment);
    }

    /// Add executable code segment
    pub fn add_text(&mut self, bytecode: Vec<u8>, vaddr: u64, offset: usize) {
        self.add_segment(X86Segment::text(bytecode, vaddr, offset));
    }

    /// Add data segment
    pub fn add_data(&mut self, data: Vec<u8>, vaddr: u64, offset: usize) {
        self.add_segment(X86Segment::data(data, vaddr, offset));
    }

    /// Add BSS segment
    pub fn add_bss(&mut self, mem_size: usize, vaddr: u64, offset: usize) {
        self.add_segment(X86Segment::bss(mem_size, vaddr, offset));
    }

    /// Get all executable segments
    pub fn executable_segments(&self) -> Vec<&X86Segment> {
        self.segments
            .iter()
            .filter(|seg| seg.is_executable)
            .collect()
    }

    /// Get all writable segments
    pub fn writable_segments(&self) -> Vec<&X86Segment> {
        self.segments.iter().filter(|seg| seg.is_writable).collect()
    }

    /// Get total size needed for all segments in memory
    pub fn total_memory_size(&self) -> u64 {
        if self.segments.is_empty() {
            return 0;
        }

        let max_end = self
            .segments
            .iter()
            .map(|seg| seg.vaddr + seg.mem_size as u64)
            .max()
            .unwrap_or(0);

        let min_start = self.segments.iter().map(|seg| seg.vaddr).min().unwrap_or(0);

        max_end - min_start
    }
}

impl From<Elf> for X86Elf {
    fn from(value: Elf) -> Self {
        // Standard x86-64 memory layout
        let code_base = 0x400000u64; // .text at 0x400000
        let data_vaddr = 0x500000u64; // .data at 0x500000 (between .text and .bss)
        let bss_base = 0x601000u64; // .bss at 0x601000

        // Determine entry point - use text_base for x86-64 (not RISC-V vaddr)
        let entry_point = code_base;

        let mut x86_elf = X86Elf::new(entry_point);
        let mut segments_to_add: Vec<(u64, X86Segment)> = Vec::new();

        for segment in value.segments {
            if segment.is_readable && segment.is_executable {
                let mut translator = RiscvToX86Translator::<RegisterMapping>::new(
                    segment.entry,
                    code_base,
                    bss_base,
                );

                for insn in segment.insns {
                    translator.process_instruction(&insn).expect(
                        format!("Error encountered translating instruction: {:?}", insn).as_str(),
                    );
                }

                // Extract PC mapping before finalize
                let pc_mapping = translator.get_pc_mapping().clone();

                // Finalize the emitter to apply all relocations
                let text_data = translator
                    .emitter
                    .finalize()
                    .expect("Failed to finalize emitter");

                // Create text segment with x86-64 vaddr
                let text_seg = X86Segment::text(text_data, code_base, 0);
                segments_to_add.push((code_base, text_seg));

                // Create PC mapping table segment
                // The table maps RISC-V PC indices to x86-64 bytecode offsets
                // Format: array of u64 values where index = (riscv_pc - entry_point) / 4
                let pc_map_vaddr = data_vaddr + 0x2000u64; // Place PC map at offset 0x2000 in data
                let mut pc_map_data =
                    Vec::with_capacity(pc_mapping.offsets.len() * 8);

                // Copy existing offsets
                for &offset in &pc_mapping.offsets {
                    pc_map_data.extend_from_slice(&offset.to_le_bytes());
                }

                let pc_map_seg = X86Segment::data(pc_map_data, pc_map_vaddr, 0);
                segments_to_add.push((pc_map_vaddr, pc_map_seg));
            } else if segment.is_readable && segment.is_writable {
                // Create BSS segment with x86-64 vaddr
                let bss_seg = X86Segment::bss(segment.mem_size, bss_base, 0);
                segments_to_add.push((bss_base, bss_seg));
            } else if segment.is_readable {
                // Create .rodata segment
                let data_seg = X86Segment::data(segment.data, data_vaddr, 0);
                segments_to_add.push((data_vaddr, data_seg));
            }
        }

        // Create a data segment with syscall mapping table
        // RISC-V syscall numbers -> x86-64 syscall numbers
        let mut syscall_map = vec![0u8; 256];
        syscall_map[63] = 0; // read
        syscall_map[64] = 1; // write
        syscall_map[93] = 60; // exit

        let syscall_data_vaddr = data_vaddr + 0x1000u64; // Place syscall table at offset 0x1000 in data
        let syscall_seg = X86Segment::data(syscall_map, syscall_data_vaddr, 0);
        segments_to_add.push((syscall_data_vaddr, syscall_seg));

        // Add an explicit BSS segment for register spilling
        // Allocate space for spilled RISC-V registers (up to 32 registers * 8 bytes = 256 bytes)
        let spill_bss_size = 256usize;
        let spill_bss_seg = X86Segment::bss(spill_bss_size, bss_base, 0);
        segments_to_add.push((bss_base, spill_bss_seg));

        // Sort segments by virtual address to maintain proper order
        segments_to_add.sort_by_key(|(vaddr, _)| *vaddr);

        for (_, seg) in segments_to_add {
            x86_elf.add_segment(seg);
        }

        x86_elf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x86_segment_creation() {
        let seg = X86Segment::text(vec![0x90, 0xc3], 0x400000, 0);
        assert_eq!(seg.vaddr, 0x400000);
        assert_eq!(seg.file_size, 2);
        assert!(seg.is_executable);
        assert!(!seg.is_writable);
    }

    #[test]
    fn test_x86_elf_creation() {
        let mut elf = X86Elf::new(0x400000);
        assert_eq!(elf.segments.len(), 0);

        elf.add_text(vec![0x90], 0x400000, 0);
        assert_eq!(elf.segments.len(), 1);
    }

    #[test]
    fn test_x86_elf_multiple_segments() {
        let mut elf = X86Elf::new(0x400000);

        elf.add_text(vec![0x90, 0xc3], 0x400000, 0x1000);
        elf.add_data(vec![1, 2, 3, 4], 0x600000, 0x2000);

        assert_eq!(elf.segments.len(), 2);
        assert_eq!(elf.executable_segments().len(), 1);
        assert_eq!(elf.writable_segments().len(), 0);
    }
}
