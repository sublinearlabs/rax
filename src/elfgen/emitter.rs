use elf::abi::{EM_X86_64, ET_EXEC, PT_LOAD};

use crate::elfgen::analyzer::{ElfLayout, ElfSegment};

const ELF_HEADER_SIZE: u64 = 64;
const PROGRAM_HEADER_SIZE: u64 = 56;

#[derive(Debug)]
/// Errors returned while serializing an `ElfLayout` as an x86-64 ELF.
pub enum EmitElfError {
    /// Segment count does not fit in the ELF header's `e_phnum` field.
    TooManySegments,
    /// Checked arithmetic overflowed while computing the output ELF.
    IntegerOverflow,
    /// A planned segment's memory size is smaller than its output file data.
    SegmentMemSmallerThanFile { index: usize },
    /// A computed output file offset does not fit in `usize`.
    SegmentFileOffsetOverflow { index: usize },
    /// Two planned output segment memory ranges overlap.
    SegmentMemoryOverlap { first: usize, second: usize },
}

impl ElfLayout {
    /// Emits this layout as a sectionless x86-64 ELF executable.
    ///
    /// Segment virtual addresses come from the layout, but file offsets are
    /// assigned freshly here. Offsets are only a serialization detail; the
    /// runtime contract is that each output `p_offset` is congruent with its
    /// segment's `p_vaddr` modulo that segment's `p_align`.
    pub fn emit_x86_elf(&self) -> Result<Vec<u8>, EmitElfError> {
        validate_output_segments(&self.segments)?;

        let output_offsets = compute_output_offsets(&self.segments)?;
        let phnum =
            u16::try_from(self.segments.len()).map_err(|_| EmitElfError::TooManySegments)?;
        let mut elf = Vec::new();

        // ELF header.
        elf.extend_from_slice(b"\x7FELF");
        elf.push(2); // ELFCLASS64
        elf.push(1); // little endian
        elf.push(1); // current version
        elf.push(0); // SYSV ABI
        elf.push(0); // ABI version
        elf.extend_from_slice(&[0; 7]);
        elf.extend_from_slice(&ET_EXEC.to_le_bytes());
        elf.extend_from_slice(&EM_X86_64.to_le_bytes());
        elf.extend_from_slice(&1u32.to_le_bytes());
        elf.extend_from_slice(&self.entry.to_le_bytes());
        elf.extend_from_slice(&ELF_HEADER_SIZE.to_le_bytes());
        elf.extend_from_slice(&0u64.to_le_bytes()); // no section headers yet
        elf.extend_from_slice(&0u32.to_le_bytes());
        elf.extend_from_slice(&(ELF_HEADER_SIZE as u16).to_le_bytes());
        elf.extend_from_slice(&(PROGRAM_HEADER_SIZE as u16).to_le_bytes());
        elf.extend_from_slice(&phnum.to_le_bytes());
        elf.extend_from_slice(&0u16.to_le_bytes());
        elf.extend_from_slice(&0u16.to_le_bytes());
        elf.extend_from_slice(&0u16.to_le_bytes());

        // Program headers.
        for (segment, output_offset) in self.segments.iter().zip(output_offsets.iter()) {
            elf.extend_from_slice(&PT_LOAD.to_le_bytes());
            elf.extend_from_slice(&segment.flags.to_le_bytes());
            elf.extend_from_slice(&output_offset.to_le_bytes());
            elf.extend_from_slice(&segment.vaddr.to_le_bytes());
            elf.extend_from_slice(&segment.vaddr.to_le_bytes());
            elf.extend_from_slice(&(segment.data.len() as u64).to_le_bytes());
            elf.extend_from_slice(&segment.memsz.to_le_bytes());
            elf.extend_from_slice(&segment.align.to_le_bytes());
        }

        // Segment data. BSS-like segments have a program header but no file bytes.
        for (segment, output_offset) in self.segments.iter().zip(output_offsets.iter()) {
            let output_offset = usize::try_from(*output_offset).map_err(|_| {
                EmitElfError::SegmentFileOffsetOverflow {
                    index: segment.index,
                }
            })?;
            if elf.len() < output_offset {
                elf.resize(output_offset, 0);
            }
            elf.extend_from_slice(&segment.data);
        }

        Ok(elf)
    }
}

/// Computes fresh output file offsets for every segment in order.
///
/// The smallest valid offset is chosen for each segment. For aligned segments,
/// the offset must satisfy the ELF congruence rule with the planned virtual
/// address. For unaligned segments (`p_align <= 1`), the current file end is
/// used directly.
fn compute_output_offsets(segments: &[ElfSegment]) -> Result<Vec<u64>, EmitElfError> {
    let phdr_bytes = PROGRAM_HEADER_SIZE
        .checked_mul(segments.len() as u64)
        .ok_or(EmitElfError::IntegerOverflow)?;
    let mut current_offset = ELF_HEADER_SIZE
        .checked_add(phdr_bytes)
        .ok_or(EmitElfError::IntegerOverflow)?;
    let mut offsets = Vec::with_capacity(segments.len());

    for segment in segments {
        let offset = align_offset_to_vaddr(current_offset, segment.vaddr, segment.align)?;
        offsets.push(offset);
        current_offset = offset
            .checked_add(segment.data.len() as u64)
            .ok_or(EmitElfError::IntegerOverflow)?;
    }

    Ok(offsets)
}

/// Finds the smallest file offset at or after `offset` that is congruent with `vaddr`.
///
/// ELF load segments require `p_offset % p_align == p_vaddr % p_align` when
/// `p_align > 1`. This helper preserves compact output while satisfying that
/// per-segment rule.
fn align_offset_to_vaddr(offset: u64, vaddr: u64, align: u64) -> Result<u64, EmitElfError> {
    if align <= 1 {
        return Ok(offset);
    }

    // target alignment delta
    let want = vaddr % align;
    // current alignment delta
    let got = offset % align;

    if got <= want {
        // here the target is larger than current
        // so we just add the difference
        offset
            .checked_add(want - got)
            .ok_or(EmitElfError::IntegerOverflow)
    } else {
        // in this case the current is larger than the target
        // we are supposed to subtract from the offset to fix this
        // but the current offset is already the minimum valid offset
        // hence we need to first add segment.align to the offset
        // and then subtract the delta difference from that result
        offset
            .checked_add(align - got)
            .and_then(|offset| offset.checked_add(want))
            .ok_or(EmitElfError::IntegerOverflow)
    }
}

/// Validates the planned output memory ranges before serializing the ELF.
fn validate_output_segments(segments: &[ElfSegment]) -> Result<(), EmitElfError> {
    let mut ranges = Vec::with_capacity(segments.len());
    for segment in segments {
        if segment.memsz < segment.data.len() as u64 {
            return Err(EmitElfError::SegmentMemSmallerThanFile {
                index: segment.index,
            });
        }

        let end = segment
            .vaddr
            .checked_add(segment.memsz)
            .ok_or(EmitElfError::IntegerOverflow)?;
        ranges.push((segment.vaddr, end, segment.index));
    }

    ranges.sort_by_key(|&(start, _, _)| start);
    for pair in ranges.windows(2) {
        let (_, first_end, first_index) = pair[0];
        let (second_start, _, second_index) = pair[1];
        if first_end > second_start {
            return Err(EmitElfError::SegmentMemoryOverlap {
                first: first_index,
                second: second_index,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use elf::abi::{EM_RISCV, PF_R, PF_W, PF_X};

    use crate::elfgen::analyzer::analyze_elf;

    use super::*;

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
    /// This keeps emitter tests focused on output serialization without relying
    /// on a fixture binary or linker.
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
        bytes.extend_from_slice(&ELF_HEADER_SIZE.to_le_bytes());
        bytes.extend_from_slice(&0u64.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&(ELF_HEADER_SIZE as u16).to_le_bytes());
        bytes.extend_from_slice(&(PROGRAM_HEADER_SIZE as u16).to_le_bytes());
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
                // Emitter tests only care that segment bytes are preserved; the
                // actual instruction encoding is irrelevant here.
                bytes[offset] = 0x13;
            }
        }

        bytes
    }

    fn read_u16(bytes: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
    }

    fn read_u32(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }

    fn read_u64(bytes: &[u8], offset: usize) -> u64 {
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
    }

    fn phdr_offset(index: usize) -> usize {
        ELF_HEADER_SIZE as usize + index * PROGRAM_HEADER_SIZE as usize
    }

    #[test]
    fn emits_x86_elf_with_fresh_offsets() {
        let bytes = elf_with(
            0x400000,
            &[
                TestSegment {
                    offset: 0x3000,
                    vaddr: 0x400000,
                    filesz: 4,
                    memsz: 4,
                    flags: PF_R | PF_X,
                    align: 0x1000,
                },
                TestSegment {
                    offset: 0x1000,
                    vaddr: 0x600000,
                    filesz: 4,
                    memsz: 4,
                    flags: PF_R | PF_W,
                    align: 0x1000,
                },
            ],
        );

        let mut layout = analyze_elf(&bytes).unwrap();
        layout.replace_executable(vec![0x90, 0xc3]);
        let output = layout.emit_x86_elf().unwrap();

        assert_eq!(&output[0..4], b"\x7FELF");
        assert_eq!(output[4], 2);
        assert_eq!(output[5], 1);
        assert_eq!(read_u16(&output, 16), ET_EXEC);
        assert_eq!(read_u16(&output, 18), EM_X86_64);
        assert_eq!(read_u64(&output, 24), layout.entry);
        assert_eq!(read_u64(&output, 32), ELF_HEADER_SIZE);
        assert_eq!(read_u16(&output, 56), 2);

        let exec_ph = phdr_offset(0);
        let data_ph = phdr_offset(1);
        let exec_offset = read_u64(&output, exec_ph + 8);
        let data_offset = read_u64(&output, data_ph + 8);

        assert_eq!(read_u32(&output, exec_ph), PT_LOAD);
        assert_eq!(read_u32(&output, exec_ph + 4), PF_R | PF_X);
        assert_eq!(read_u64(&output, exec_ph + 16), 0x601000);
        assert_eq!(read_u64(&output, exec_ph + 32), 2);
        assert_eq!(read_u64(&output, exec_ph + 40), 2);
        assert_eq!(read_u64(&output, exec_ph + 48), 0x1000);
        assert_eq!(exec_offset % 0x1000, 0x601000 % 0x1000);
        assert_eq!(
            &output[exec_offset as usize..exec_offset as usize + 2],
            &[0x90, 0xc3]
        );

        assert_eq!(read_u32(&output, data_ph), PT_LOAD);
        assert_eq!(read_u32(&output, data_ph + 4), PF_R | PF_W);
        assert_eq!(read_u64(&output, data_ph + 16), 0x600000);
        assert_eq!(read_u64(&output, data_ph + 32), 4);
        assert_eq!(read_u64(&output, data_ph + 40), 4);
        assert_eq!(read_u64(&output, data_ph + 48), 0x1000);
        assert_eq!(data_offset % 0x1000, 0x600000 % 0x1000);
        assert_eq!(
            &output[data_offset as usize..data_offset as usize + 4],
            &[0x13; 4]
        );

        assert_ne!(exec_offset, layout.executable_segment().source_offset);
        assert_ne!(data_offset, layout.segments[1].source_offset);
    }

    #[test]
    fn emits_bss_like_segment_without_file_bytes() {
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
                    filesz: 0,
                    memsz: 0x100,
                    flags: PF_R | PF_W,
                    align: 0x1000,
                },
            ],
        );

        let mut layout = analyze_elf(&bytes).unwrap();
        layout.replace_executable(vec![0x90, 0xc3]);
        let output = layout.emit_x86_elf().unwrap();

        let bss_ph = phdr_offset(1);
        let bss_offset = read_u64(&output, bss_ph + 8);
        assert_eq!(read_u64(&output, bss_ph + 16), 0x600000);
        assert_eq!(read_u64(&output, bss_ph + 32), 0);
        assert_eq!(read_u64(&output, bss_ph + 40), 0x100);
        assert_eq!(bss_offset % 0x1000, 0x600000 % 0x1000);
        assert!(output.len() >= bss_offset as usize);
    }
}
