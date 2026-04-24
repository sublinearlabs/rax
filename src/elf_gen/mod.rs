//! ELF file generation for x86-64 binaries
//!
//! This module handles the generation of complete x86-64 ELF executable files
//! from compiled x86-64 bytecode with proper program headers for executability.

pub mod x86_elf;

pub use x86_elf::{X86Elf, X86Segment};

/// Standard page alignment for x86-64 ELF files
const PAGE_ALIGN: u64 = 0x1000;

/// Generate an x86-64 ELF binary from segments
pub fn generate_elf(x86_elf: &X86Elf) -> Result<Vec<u8>, String> {
    if x86_elf.segments.is_empty() {
        return Err("No segments to generate ELF".to_string());
    }

    // TODO: this only works for single segment executable
    assert_eq!(x86_elf.segments.len(), 1);
    assert!(x86_elf.segments[0].is_executable);

    let mut elf = Vec::new();

    // Calculate number of program headers needed
    let num_segments = x86_elf.segments.len();

    // ELF header size is always 64 bytes
    let elf_header_size = 64;
    // Each program header is 56 bytes for 64-bit
    let program_header_size = 56 * num_segments;
    let headers_size = elf_header_size + program_header_size;

    // Calculate file offsets for each segment (aligned to page boundaries)
    let mut segment_offsets: Vec<u64> = Vec::with_capacity(num_segments);
    let mut current_offset = headers_size as u64;

    for segment in &x86_elf.segments {
        if segment.data.is_empty() && segment.mem_size == 0 {
            // BSS segment - no file space needed
            segment_offsets.push(0);
        } else if !segment.data.is_empty() {
            // since it is one executable, the global entry
            // should be the same as the segment entry
            assert_eq!(x86_elf.entry_point, segment.vaddr);

            // compute the vaddr delta aligned elf segment offset
            let page_delta = segment.vaddr % PAGE_ALIGN;
            let next_page_aligned_offset = aligned_up(current_offset, PAGE_ALIGN);
            let offset = next_page_aligned_offset + page_delta;

            // Align to page boundary if executable
            if segment.is_executable {
                current_offset = offset;
            }
            segment_offsets.push(current_offset);
            current_offset += segment.data.len() as u64;
        } else {
            segment_offsets.push(current_offset);
        }
    }

    // ============ ELF Header (64 bytes) ============
    elf.extend_from_slice(b"\x7FELF"); // Magic number
    elf.push(2); // e_ident[4]: ELFCLASS64
    elf.push(1); // e_ident[5]: ELFDATA2LSB (little-endian)
    elf.push(1); // e_ident[6]: EV_CURRENT
    elf.push(0); // e_ident[7]: ELFOSABI_SYSV
    elf.push(0); // e_ident[8]: ABI version
    elf.extend_from_slice(&[0; 7]); // e_ident[9:16] - padding

    elf.extend_from_slice(&2u16.to_le_bytes()); // e_type: ET_EXEC
    elf.extend_from_slice(&62u16.to_le_bytes()); // e_machine: EM_X86_64
    elf.extend_from_slice(&1u32.to_le_bytes()); // e_version
    elf.extend_from_slice(&x86_elf.entry_point.to_le_bytes()); // e_entry
    elf.extend_from_slice(&64u64.to_le_bytes()); // e_phoff (program header offset)
    elf.extend_from_slice(&0u64.to_le_bytes()); // e_shoff (no section headers)
    elf.extend_from_slice(&0u32.to_le_bytes()); // e_flags
    elf.extend_from_slice(&64u16.to_le_bytes()); // e_ehsize
    elf.extend_from_slice(&56u16.to_le_bytes()); // e_phentsize
    elf.extend_from_slice(&(num_segments as u16).to_le_bytes()); // e_phnum
    elf.extend_from_slice(&0u16.to_le_bytes()); // e_shentsize
    elf.extend_from_slice(&0u16.to_le_bytes()); // e_shnum
    elf.extend_from_slice(&0u16.to_le_bytes()); // e_shstrndx

    // ============ Program Headers ============
    for (i, segment) in x86_elf.segments.iter().enumerate() {
        let file_offset = segment_offsets[i];

        // Compute flags: PF_X=1, PF_W=2, PF_R=4
        let mut flags = 0u32;
        if segment.is_readable {
            flags |= 4;
        }
        if segment.is_writable {
            flags |= 2;
        }
        if segment.is_executable {
            flags |= 1;
        }

        // Program header entry (56 bytes)
        elf.extend_from_slice(&1u32.to_le_bytes()); // p_type: PT_LOAD
        elf.extend_from_slice(&flags.to_le_bytes()); // p_flags
        elf.extend_from_slice(&file_offset.to_le_bytes()); // p_offset
        elf.extend_from_slice(&segment.vaddr.to_le_bytes()); // p_vaddr
        elf.extend_from_slice(&segment.vaddr.to_le_bytes()); // p_paddr
        elf.extend_from_slice(&(segment.file_size as u64).to_le_bytes()); // p_filesz
        elf.extend_from_slice(&(segment.mem_size as u64).to_le_bytes()); // p_memsz
        elf.extend_from_slice(&PAGE_ALIGN.to_le_bytes()); // p_align
    }

    // ============ Segment Data ============
    // Pad to first segment offset
    let padding_needed = segment_offsets
        .iter()
        .filter(|&&offset| offset > 0)
        .min()
        .map(|&offset| offset as usize)
        .unwrap_or(elf.len());

    if elf.len() < padding_needed {
        elf.resize(padding_needed, 0);
    }

    // Write each segment's data
    for (i, segment) in x86_elf.segments.iter().enumerate() {
        let file_offset = segment_offsets[i] as usize;

        if !segment.data.is_empty() {
            // Ensure we have enough space
            if elf.len() < file_offset {
                elf.resize(file_offset, 0);
            }
            elf.extend_from_slice(&segment.data);
        }
    }

    Ok(elf)
}

/// Computes the closest page boundary less than addr
fn aligned_down(addr: u64, page_size: u64) -> u64 {
    assert!(page_size > 0);
    addr - (addr % page_size)
}

/// Computes the closest page boundary greater than addr
fn aligned_up(addr: u64, page_size: u64) -> u64 {
    assert!(page_size > 0);
    if addr % page_size == 0 {
        addr
    } else {
        addr + (page_size - (addr % page_size))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use dynasmrt::{dynasm, x64, x86, DynasmApi};

    use super::*;

    #[test]
    fn test_generate_elf_single_segment() {
        let mut elf = X86Elf::new(0x400000);
        elf.add_text(vec![0x90, 0xc3], 0x400000, 0); // NOP; RET

        let result = generate_elf(&elf);
        assert!(result.is_ok());

        let binary = result.unwrap();
        assert_eq!(&binary[0..4], b"\x7FELF");
        assert_eq!(binary[4], 2); // ELFCLASS64
        assert_eq!(binary[5], 1); // ELFDATA2LSB
    }

    #[test]
    fn test_generate_elf_multiple_segments() {
        let mut elf = X86Elf::new(0x400000);
        elf.add_text(vec![0x90, 0xc3], 0x400000, 0);
        elf.add_data(vec![1, 2, 3, 4], 0x600000, 0x2000);

        let result = generate_elf(&elf);
        assert!(result.is_ok());

        let binary = result.unwrap();
        assert!(!binary.is_empty());
    }

    #[test]
    fn test_generate_elf_no_segments() {
        let elf = X86Elf::new(0x400000);
        let result = generate_elf(&elf);
        assert!(result.is_err());
    }
}
