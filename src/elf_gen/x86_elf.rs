//! x86-64 ELF segment representation
//!
//! This module provides structures for representing x86-64 code segments

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

    /// Create a data segment (.data)
    pub fn data(data: Vec<u8>, vaddr: u64, offset: usize) -> Self {
        let file_size = data.len();
        Self::new(
            data, vaddr, offset, file_size, file_size, false, // not executable
            true,  // writable
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
        assert_eq!(elf.writable_segments().len(), 1);
    }
}
