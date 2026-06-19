use std::ops::Range;

use elf::{
    abi::{EM_RISCV, ET_EXEC, PF_R, PF_W, PF_X, PT_LOAD},
    endian::LittleEndian,
    file::Class,
    parse::ParseError,
    ElfBytes,
};

/// Segment-oriented layout for an AOT output ELF.
///
/// The AOT pipeline cares about what the loader maps into memory. Sections are
/// intentionally ignored here because they are a tooling/linker view, not the
/// runtime layout we need to preserve or validate. During analysis, the
/// executable segment is moved after all preserved non-executable load segments
/// so translation knows the output code address before emitting bytes.
#[derive(Debug)]
pub struct ElfLayout {
    /// Entry address for the output ELF.
    pub entry: u64,
    /// Original RISC-V ELF entry address.
    pub source_entry_vaddr: u64,
    /// Original RISC-V virtual address of the executable segment.
    pub source_executable_vaddr: u64,
    /// Planned output loadable segments.
    pub segments: Vec<ElfSegment>,
    /// Index of the single executable segment in `segments`.
    pub executable_segment_index: usize,
}

impl ElfLayout {
    /// Returns the single executable segment in the planned output layout.
    pub fn executable_segment(&self) -> &ElfSegment {
        &self.segments[self.executable_segment_index]
    }

    /// Returns the single executable segment mutably for layout updates.
    pub fn executable_segment_mut(&mut self) -> &mut ElfSegment {
        &mut self.segments[self.executable_segment_index]
    }

    /// Replaces the executable segment with translated x86 bytes.
    ///
    /// The segment's virtual address is not changed here. Analysis already chose
    /// that address so the translator can use it as its output base while
    /// emitting code.
    pub fn replace_executable(&mut self, translated: Vec<u8>) {
        let len = translated.len() as u64;
        let executable = self.executable_segment_mut();
        executable.filesz = len;
        executable.memsz = len;
        executable.data = translated;
    }
}

#[derive(Debug)]
pub struct ElfSegment {
    /// Program-header index in the original ELF.
    pub index: usize,
    /// Input ELF file offset where this segment's initialized bytes start.
    pub source_offset: u64,
    /// Virtual address where this segment is planned to be loaded.
    pub vaddr: u64,
    /// Number of segment bytes present in the ELF file.
    pub filesz: u64,
    /// Total segment size in memory, including zero-filled bytes.
    pub memsz: u64,
    /// Raw `p_flags` from the program header.
    pub flags: u32,
    /// Per-segment alignment constraint for file offset and virtual address.
    pub align: u64,
    /// Initialized bytes copied from the ELF file, or replacement translated bytes.
    pub data: Vec<u8>,
}

impl ElfSegment {
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

    /// Returns whether a virtual address starts a full file-backed instruction.
    pub fn contains_file_instruction_vaddr(&self, addr: u64) -> bool {
        let Some(instruction_end) = addr.checked_add(4) else {
            return false;
        };
        self.vaddr <= addr && instruction_end <= self.vaddr + self.filesz
    }
}

#[derive(Debug)]
/// Errors returned while analyzing a RISC-V ELF into an AOT layout.
pub enum AnalyzeElfError {
    /// The underlying `elf` crate failed to parse the input bytes.
    Parse(ParseError),
    /// The input ELF is not a 64-bit ELF.
    NotElf64,
    /// The input ELF is not an executable ELF.
    NotExecutable,
    /// The input ELF is not for the RISC-V architecture.
    NotRiscv,
    /// The input ELF has no program header table.
    MissingProgramHeaders,
    /// The input ELF has no loadable `PT_LOAD` segments.
    NoLoadSegments,
    /// A segment's file range overflows or extends past the input bytes.
    SegmentFileRangeOutOfBounds { index: usize },
    /// A segment's in-memory size is smaller than its file-backed size.
    SegmentMemSmallerThanFile { index: usize },
    /// A segment violates `p_offset % p_align == p_vaddr % p_align`.
    SegmentMisaligned { index: usize },
    /// No executable loadable segment was found.
    NoExecutableSegment,
    /// More than one executable loadable segment was found.
    MultipleExecutableSegments,
    /// The ELF entry point is not inside executable file-backed bytes.
    EntryNotInExecutableSegment,
    /// The ELF entry point is not aligned to a 4-byte instruction boundary.
    EntryMisaligned,
    /// Checked arithmetic overflowed while analyzing layout.
    IntegerOverflow,
    /// Two loadable segment memory ranges overlap.
    LoadSegmentsOverlap { first: usize, second: usize },
}

impl From<ParseError> for AnalyzeElfError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}

/// Parses a RISC-V ELF and returns the planned loadable layout for AOT output.
///
/// All non-executable `PT_LOAD` segments keep their original virtual addresses.
/// The executable segment keeps its original bytes for translation input, but is
/// moved after the preserved non-executable segments so its output address is
/// known before translation begins.
pub fn analyze_elf(bytes: &[u8]) -> Result<ElfLayout, AnalyzeElfError> {
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
    let mut segments = Vec::new();
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
        let segment = ElfSegment {
            index,
            source_offset: phdr.p_offset,
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
            executable_segment_index = Some(segments.len());
        }

        segments.push(segment);
    }

    if segments.is_empty() {
        return Err(AnalyzeElfError::NoLoadSegments);
    }

    reject_memory_overlaps(&segments)?;

    let executable_segment_index =
        executable_segment_index.ok_or(AnalyzeElfError::NoExecutableSegment)?;

    let executable = &segments[executable_segment_index];
    if !executable.contains_file_instruction_vaddr(ehdr.e_entry) {
        return Err(AnalyzeElfError::EntryNotInExecutableSegment);
    }

    if (ehdr.e_entry - executable.vaddr) % 4 != 0 {
        return Err(AnalyzeElfError::EntryMisaligned);
    }

    let source_entry_vaddr = ehdr.e_entry;
    let source_executable_vaddr = segments[executable_segment_index].vaddr;
    let entry = move_executable_after_preserved_segments(&mut segments, executable_segment_index)?;

    Ok(ElfLayout {
        entry,
        source_entry_vaddr,
        source_executable_vaddr,
        segments,
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
fn reject_memory_overlaps(segments: &[ElfSegment]) -> Result<(), AnalyzeElfError> {
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

/// Moves the executable segment after all preserved non-executable memory ranges.
///
/// Only the executable segment is movable in this layout policy. If the input ELF
/// has no non-executable loadable segments, the executable segment remains at its
/// original virtual address.
fn move_executable_after_preserved_segments(
    segments: &mut [ElfSegment],
    executable_segment_index: usize,
) -> Result<u64, AnalyzeElfError> {
    let mut preserved_end = None;
    for (index, segment) in segments.iter().enumerate() {
        if index == executable_segment_index {
            continue;
        }

        let end = checked_end(segment.vaddr, segment.memsz, segment.index)?;
        preserved_end = Some(preserved_end.map_or(end, |max: u64| max.max(end)));
    }

    if let Some(preserved_end) = preserved_end {
        let executable = &mut segments[executable_segment_index];
        executable.vaddr = align_up(preserved_end, executable.align)?;
    }

    Ok(segments[executable_segment_index].vaddr)
}

/// Rounds `value` up to the next multiple of `align`.
///
/// ELF treats `p_align` values of 0 and 1 as no alignment requirement, so those
/// values return `value` unchanged.
fn align_up(value: u64, align: u64) -> Result<u64, AnalyzeElfError> {
    if align <= 1 {
        return Ok(value);
    }

    let remainder = value % align;
    if remainder == 0 {
        Ok(value)
    } else {
        value
            .checked_add(align - remainder)
            .ok_or(AnalyzeElfError::IntegerOverflow)
    }
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
        assert_eq!(elf.source_entry_vaddr, 0x400000);
        assert_eq!(elf.source_executable_vaddr, 0x400000);
        assert_eq!(elf.segments.len(), 1);
        assert_eq!(elf.executable_segment().data, vec![0x13; 4]);
        assert_eq!(elf.executable_segment().vaddr, 0x400000);
        assert!(elf.executable_segment().is_executable());
    }

    #[test]
    fn allows_entry_inside_executable_file_bytes() {
        let bytes = elf_with(
            0x400004,
            &[TestSegment {
                offset: 0x1000,
                vaddr: 0x400000,
                filesz: 8,
                memsz: 8,
                flags: PF_R | PF_X,
                align: 0x1000,
            }],
        );

        let elf = analyze_elf(&bytes).unwrap();
        assert_eq!(elf.source_entry_vaddr, 0x400004);
        assert_eq!(elf.source_executable_vaddr, 0x400000);
        assert_eq!(elf.entry, 0x400000);
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
        assert_eq!(elf.segments.len(), 2);
        assert_eq!(elf.segments[1].data, Vec::<u8>::new());
        assert_eq!(elf.segments[1].memory_range(), 0x401000..0x401100);
        assert_eq!(elf.source_executable_vaddr, 0x400000);
        assert_eq!(elf.executable_segment().vaddr, 0x402000);
        assert_eq!(elf.entry, 0x402000);
    }

    #[test]
    fn moves_executable_after_preserved_non_executable_segments() {
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
                    vaddr: 0x600000,
                    filesz: 4,
                    memsz: 0x1234,
                    flags: PF_R | PF_W,
                    align: 0x1000,
                },
            ],
        );

        let elf = analyze_elf(&bytes).unwrap();
        assert_eq!(elf.source_executable_vaddr, 0x400000);
        assert_eq!(elf.segments[1].vaddr, 0x600000);
        assert_eq!(elf.executable_segment().vaddr, 0x602000);
        assert_eq!(elf.entry, 0x602000);
    }

    #[test]
    fn replace_executable_updates_bytes_and_sizes() {
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

        let mut elf = analyze_elf(&bytes).unwrap();
        elf.replace_executable(vec![0x90, 0xc3]);

        assert_eq!(elf.executable_segment().filesz, 2);
        assert_eq!(elf.executable_segment().memsz, 2);
        assert_eq!(elf.executable_segment().data, vec![0x90, 0xc3]);
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

    #[test]
    fn rejects_entry_outside_executable_file_bytes() {
        let bytes = elf_with(
            0x400004,
            &[TestSegment {
                offset: 0x1000,
                vaddr: 0x400000,
                filesz: 4,
                memsz: 8,
                flags: PF_R | PF_X,
                align: 0x1000,
            }],
        );

        assert!(matches!(
            analyze_elf(&bytes),
            Err(AnalyzeElfError::EntryNotInExecutableSegment)
        ));
    }

    #[test]
    fn rejects_entry_without_full_file_backed_instruction() {
        let bytes = elf_with(
            0x400004,
            &[TestSegment {
                offset: 0x1000,
                vaddr: 0x400000,
                filesz: 6,
                memsz: 6,
                flags: PF_R | PF_X,
                align: 0x1000,
            }],
        );

        assert!(matches!(
            analyze_elf(&bytes),
            Err(AnalyzeElfError::EntryNotInExecutableSegment)
        ));
    }

    #[test]
    fn rejects_misaligned_entry() {
        let bytes = elf_with(
            0x400002,
            &[TestSegment {
                offset: 0x1000,
                vaddr: 0x400000,
                filesz: 8,
                memsz: 8,
                flags: PF_R | PF_X,
                align: 0x1000,
            }],
        );

        assert!(matches!(
            analyze_elf(&bytes),
            Err(AnalyzeElfError::EntryMisaligned)
        ));
    }
}
