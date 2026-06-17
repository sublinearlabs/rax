use std::ops::Range;

use elf::{
    abi::{EM_RISCV, ET_EXEC, PF_R, PF_W, PF_X, PT_LOAD},
    endian::LittleEndian,
    file::Class,
    parse::ParseError,
    ElfBytes,
};

/// Segment-oriented view of an ELF input.
///
/// The AOT pipeline cares about what the loader maps into memory. Sections are
/// intentionally ignored here because they are a tooling/linker view, not the
/// runtime layout we need to preserve or validate.
#[derive(Debug)]
pub struct AnalyzedElf {
    pub entry: u64,
    pub load_segments: Vec<AnalyzedSegment>,
    pub executable_segment_index: usize,
}

impl AnalyzedElf {
    pub fn executable_segment(&self) -> &AnalyzedSegment {
        &self.load_segments[self.executable_segment_index]
    }
}

#[derive(Debug)]
pub struct AnalyzedSegment {
    // Program-header index in the original ELF.
    pub index: usize,
    // File offset where this segment's initialized bytes start.
    pub offset: u64,
    // Virtual address where the loader maps this segment.
    pub vaddr: u64,
    // Number of segment bytes present in the ELF file.
    pub filesz: u64,
    // Total segment size in memory, including zero-filled bytes.
    pub memsz: u64,
    // Raw p_flags from the program header.
    pub flags: u32,
    // Per-segment alignment constraint for file offset and virtual address.
    pub align: u64,
    // Initialized bytes copied from the ELF file.
    pub data: Vec<u8>,
}

impl AnalyzedSegment {
    /// Returns the byte range occupied by this segment in the ELF file.
    pub fn file_range(&self) -> Range<u64> {
        self.offset..self.offset + self.filesz
    }

    /// Returns the virtual memory range reserved by this segment after loading.
    pub fn memory_range(&self) -> Range<u64> {
        self.vaddr..self.vaddr + self.memsz
    }

    /// Returns whether the segment has the loader-readable flag set.
    pub fn is_readable(&self) -> bool {
        (self.flags & PF_R) != 0
    }

    /// Returns whether the segment has the loader-writable flag set.
    pub fn is_writable(&self) -> bool {
        (self.flags & PF_W) != 0
    }

    /// Returns whether the segment has the loader-executable flag set.
    pub fn is_executable(&self) -> bool {
        (self.flags & PF_X) != 0
    }

    /// Returns whether a virtual address falls inside this segment's memory range.
    pub fn contains_vaddr(&self, addr: u64) -> bool {
        self.vaddr <= addr && addr < self.vaddr + self.memsz
    }
}

#[derive(Debug)]
pub enum AnalyzeElfError {
    Parse(ParseError),
    NotElf64,
    NotExecutable,
    NotRiscv,
    MissingProgramHeaders,
    NoLoadSegments,
    SegmentFileRangeOutOfBounds { index: usize },
    SegmentMemSmallerThanFile { index: usize },
    SegmentMisaligned { index: usize },
    NoExecutableSegment,
    MultipleExecutableSegments,
    EntryNotInExecutableSegment,
    LoadSegmentsOverlap { first: usize, second: usize },
}

impl From<ParseError> for AnalyzeElfError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}

pub fn analyze_elf(bytes: &[u8]) -> Result<AnalyzedElf, AnalyzeElfError> {
    let file = ElfBytes::<LittleEndian>::minimal_parse(bytes)?;
    let ehdr = file.ehdr;

    if ehdr.class != Class::ELF64 {
        return Err(AnalyzeElfError::NotElf64);
    }
    if ehdr.e_type != ET_EXEC {
        return Err(AnalyzeElfError::NotExecutable);
    }
    if ehdr.e_machine != EM_RISCV {
        return Err(AnalyzeElfError::NotRiscv);
    }

    let phdrs = file
        .segments()
        .ok_or(AnalyzeElfError::MissingProgramHeaders)?;
    let mut load_segments = Vec::new();
    let mut executable_segment_index = None;

    for (index, phdr) in phdrs.iter().enumerate() {
        if phdr.p_type != PT_LOAD {
            continue;
        }

        if phdr.p_memsz < phdr.p_filesz {
            return Err(AnalyzeElfError::SegmentMemSmallerThanFile { index });
        }

        if phdr.p_align > 1 && phdr.p_vaddr % phdr.p_align != phdr.p_offset % phdr.p_align {
            return Err(AnalyzeElfError::SegmentMisaligned { index });
        }

        let file_end = checked_end(phdr.p_offset, phdr.p_filesz, index)?;
        if file_end > bytes.len() as u64 {
            return Err(AnalyzeElfError::SegmentFileRangeOutOfBounds { index });
        }

        let start = phdr.p_offset as usize;
        let end = file_end as usize;
        let segment = AnalyzedSegment {
            index,
            offset: phdr.p_offset,
            vaddr: phdr.p_vaddr,
            filesz: phdr.p_filesz,
            memsz: phdr.p_memsz,
            flags: phdr.p_flags,
            align: phdr.p_align,
            // A filesz=0 segment still matters: it reserves zero-filled memory
            // that translated code must not overlap later.
            data: bytes[start..end].to_vec(),
        };

        if segment.is_executable() {
            if executable_segment_index.is_some() {
                return Err(AnalyzeElfError::MultipleExecutableSegments);
            }
            executable_segment_index = Some(load_segments.len());
        }

        load_segments.push(segment);
    }

    if load_segments.is_empty() {
        return Err(AnalyzeElfError::NoLoadSegments);
    }

    reject_memory_overlaps(&load_segments)?;

    let executable_segment_index =
        executable_segment_index.ok_or(AnalyzeElfError::NoExecutableSegment)?;
    if !load_segments[executable_segment_index].contains_vaddr(ehdr.e_entry) {
        return Err(AnalyzeElfError::EntryNotInExecutableSegment);
    }

    Ok(AnalyzedElf {
        entry: ehdr.e_entry,
        load_segments,
        executable_segment_index,
    })
}

/// Computes `start + size` while reporting the segment that overflowed.
fn checked_end(start: u64, size: u64, index: usize) -> Result<u64, AnalyzeElfError> {
    start
        .checked_add(size)
        .ok_or(AnalyzeElfError::SegmentFileRangeOutOfBounds { index })
}

/// Rejects loadable segments whose runtime memory ranges intersect.
fn reject_memory_overlaps(segments: &[AnalyzedSegment]) -> Result<(), AnalyzeElfError> {
    let mut ranges = Vec::with_capacity(segments.len());
    for segment in segments {
        let end = segment.vaddr.checked_add(segment.memsz).ok_or(
            AnalyzeElfError::LoadSegmentsOverlap {
                first: segment.index,
                second: segment.index,
            },
        )?;
        ranges.push((segment.vaddr, end, segment.index));
    }

    ranges.sort_by_key(|&(start, _, _)| start);
    for pair in ranges.windows(2) {
        let (_, first_end, first_index) = pair[0];
        let (second_start, _, second_index) = pair[1];
        if first_end > second_start {
            return Err(AnalyzeElfError::LoadSegmentsOverlap {
                first: first_index,
                second: second_index,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EHDR_SIZE: u64 = 64;
    const PHDR_SIZE: u64 = 56;

    #[derive(Clone, Copy)]
    struct TestSegment {
        offset: u64,
        vaddr: u64,
        filesz: u64,
        memsz: u64,
        flags: u32,
        align: u64,
    }

    /// Builds a minimal RISC-V ELF64 executable with the supplied load segments.
    ///
    /// This keeps tests focused on analyzer behavior without depending on a full
    /// linker or fixture binary.
    fn elf_with(entry: u64, segments: &[TestSegment]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"\x7FELF");
        bytes.push(2); // ELFCLASS64
        bytes.push(1); // little endian
        bytes.push(1); // current version
        bytes.push(0); // SYSV ABI
        bytes.push(0); // ABI version
        bytes.extend_from_slice(&[0; 7]);
        bytes.extend_from_slice(&ET_EXEC.to_le_bytes());
        bytes.extend_from_slice(&EM_RISCV.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&entry.to_le_bytes());
        bytes.extend_from_slice(&EHDR_SIZE.to_le_bytes());
        bytes.extend_from_slice(&0u64.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&(EHDR_SIZE as u16).to_le_bytes());
        bytes.extend_from_slice(&(PHDR_SIZE as u16).to_le_bytes());
        bytes.extend_from_slice(&(segments.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());

        for segment in segments {
            bytes.extend_from_slice(&PT_LOAD.to_le_bytes());
            bytes.extend_from_slice(&segment.flags.to_le_bytes());
            bytes.extend_from_slice(&segment.offset.to_le_bytes());
            bytes.extend_from_slice(&segment.vaddr.to_le_bytes());
            bytes.extend_from_slice(&segment.vaddr.to_le_bytes());
            bytes.extend_from_slice(&segment.filesz.to_le_bytes());
            bytes.extend_from_slice(&segment.memsz.to_le_bytes());
            bytes.extend_from_slice(&segment.align.to_le_bytes());
        }

        for segment in segments {
            let end = (segment.offset + segment.filesz) as usize;
            if bytes.len() < end {
                bytes.resize(end, 0);
            }
            for offset in segment.offset as usize..end {
                // Analyzer tests only care that segment bytes are preserved; the
                // actual instruction encoding is irrelevant here.
                bytes[offset] = 0x13;
            }
        }

        bytes
    }

    #[test]
    fn analyzes_single_executable_segment() {
        let bytes = elf_with(
            0x400000,
            &[TestSegment {
                offset: 0x1000,
                vaddr: 0x400000,
                filesz: 4,
                memsz: 4,
                flags: PF_R | PF_X,
                align: 0x1000,
            }],
        );

        let elf = analyze_elf(&bytes).unwrap();
        assert_eq!(elf.entry, 0x400000);
        assert_eq!(elf.load_segments.len(), 1);
        assert_eq!(elf.executable_segment().data, vec![0x13; 4]);
        assert!(elf.executable_segment().is_executable());
    }

    #[test]
    fn keeps_bss_like_load_segments() {
        let bytes = elf_with(
            0x400000,
            &[
                TestSegment {
                    offset: 0x1000,
                    vaddr: 0x400000,
                    filesz: 4,
                    memsz: 4,
                    flags: PF_R | PF_X,
                    align: 0x1000,
                },
                TestSegment {
                    offset: 0x1000,
                    vaddr: 0x401000,
                    filesz: 0,
                    memsz: 0x100,
                    flags: PF_R | PF_W,
                    align: 0x1000,
                },
            ],
        );

        let elf = analyze_elf(&bytes).unwrap();
        assert_eq!(elf.load_segments.len(), 2);
        assert_eq!(elf.load_segments[1].data, Vec::<u8>::new());
        assert_eq!(elf.load_segments[1].memory_range(), 0x401000..0x401100);
    }

    #[test]
    fn rejects_multiple_executable_segments() {
        let bytes = elf_with(
            0x400000,
            &[
                TestSegment {
                    offset: 0x1000,
                    vaddr: 0x400000,
                    filesz: 4,
                    memsz: 4,
                    flags: PF_R | PF_X,
                    align: 0x1000,
                },
                TestSegment {
                    offset: 0x2000,
                    vaddr: 0x401000,
                    filesz: 4,
                    memsz: 4,
                    flags: PF_R | PF_X,
                    align: 0x1000,
                },
            ],
        );

        assert!(matches!(
            analyze_elf(&bytes),
            Err(AnalyzeElfError::MultipleExecutableSegments)
        ));
    }

    #[test]
    fn rejects_misaligned_segments() {
        let bytes = elf_with(
            0x400001,
            &[TestSegment {
                offset: 0x1000,
                vaddr: 0x400001,
                filesz: 4,
                memsz: 4,
                flags: PF_R | PF_X,
                align: 0x1000,
            }],
        );

        assert!(matches!(
            analyze_elf(&bytes),
            Err(AnalyzeElfError::SegmentMisaligned { index: 0 })
        ));
    }

    #[test]
    fn rejects_entry_outside_executable_segment() {
        let bytes = elf_with(
            0x500000,
            &[TestSegment {
                offset: 0x1000,
                vaddr: 0x400000,
                filesz: 4,
                memsz: 4,
                flags: PF_R | PF_X,
                align: 0x1000,
            }],
        );

        assert!(matches!(
            analyze_elf(&bytes),
            Err(AnalyzeElfError::EntryNotInExecutableSegment)
        ));
    }
}
