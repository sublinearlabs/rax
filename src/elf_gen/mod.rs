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

    let mut elf = Vec::new();

    // Calculate number of program headers needed
    let num_segments = x86_elf.segments.len();

    // ELF header size is always 64 bytes
    let elf_header_size = 64;
    // Each program header is 56 bytes for 64-bit
    let program_header_size = 56 * num_segments;
    let headers_size = elf_header_size + program_header_size;

    // Calculate file offsets for each segment (aligned to page boundaries)
    // Key constraint: (p_vaddr % p_align) == (p_offset % p_align)
    let mut segment_offsets: Vec<u64> = Vec::with_capacity(num_segments);
    let mut current_offset = headers_size as u64;

    for segment in &x86_elf.segments {
        if segment.data.is_empty() {
            // BSS segment or empty segment - no file space needed
            segment_offsets.push(0);
        } else {
            // Align p_offset such that (p_vaddr % PAGE_ALIGN) == (p_offset % PAGE_ALIGN)
            // First, align to page boundary
            current_offset = ((current_offset + PAGE_ALIGN - 1) / PAGE_ALIGN) * PAGE_ALIGN;

            // Then adjust so the alignment constraint is satisfied
            let vaddr_offset = segment.vaddr % PAGE_ALIGN;
            let file_offset_mod = current_offset % PAGE_ALIGN;

            if vaddr_offset != file_offset_mod {
                // Need to adjust current_offset by (vaddr_offset - file_offset_mod)
                // If vaddr_offset < file_offset_mod, we need to go to next page
                if vaddr_offset < file_offset_mod {
                    current_offset += PAGE_ALIGN - (file_offset_mod - vaddr_offset);
                } else {
                    current_offset += vaddr_offset - file_offset_mod;
                }
            }

            segment_offsets.push(current_offset);
            current_offset += segment.data.len() as u64;
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

    // Calculate section header offset (after all segment data)
    // Use the segment_offsets we already calculated
    let mut section_header_offset = headers_size as u64;
    for (i, segment) in x86_elf.segments.iter().enumerate() {
        if !segment.data.is_empty() {
            let offset = segment_offsets[i];
            section_header_offset = section_header_offset.max(offset + segment.data.len() as u64);
        }
    }

    let num_sections = 4;
    let shstrtab_offset = section_header_offset + (64 * num_sections) as u64;
    let shstrtab_data = b"\0_start\0.data\0.shstrtab\0";

    elf.extend_from_slice(&64u64.to_le_bytes()); // e_phoff (program header offset = 64 bytes)
    elf.extend_from_slice(&section_header_offset.to_le_bytes()); // e_shoff (section header offset)
    elf.extend_from_slice(&0u32.to_le_bytes()); // e_flags
    elf.extend_from_slice(&64u16.to_le_bytes()); // e_ehsize
    elf.extend_from_slice(&56u16.to_le_bytes()); // e_phentsize
    elf.extend_from_slice(&(num_segments as u16).to_le_bytes()); // e_phnum
    elf.extend_from_slice(&64u16.to_le_bytes()); // e_shentsize (section header entry size)
    elf.extend_from_slice(&(num_sections as u16).to_le_bytes()); // e_shnum (number of sections)
    elf.extend_from_slice(&3u16.to_le_bytes()); // e_shstrndx (section header string table index)

    // ============ Program Headers ============
    // Track the last file offset for BSS segments
    let mut last_file_offset = headers_size as u64;

    for (i, segment) in x86_elf.segments.iter().enumerate() {
        let file_offset = segment_offsets[i];
        let file_size = if segment.data.is_empty() {
            0
        } else {
            segment.data.len() as u64
        };
        let mem_size = if segment.data.is_empty() {
            segment.mem_size as u64
        } else {
            segment.data.len() as u64
        };

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

        // For BSS segments (empty data), set p_offset to point after file data
        // but with p_filesz = 0
        let p_offset = if file_size == 0 {
            // For BSS: use the last valid file offset after all data
            last_file_offset
        } else {
            last_file_offset = file_offset + file_size;
            file_offset
        };

        // Program header entry (56 bytes)
        elf.extend_from_slice(&1u32.to_le_bytes()); // p_type: PT_LOAD
        elf.extend_from_slice(&flags.to_le_bytes()); // p_flags
        elf.extend_from_slice(&p_offset.to_le_bytes()); // p_offset
        elf.extend_from_slice(&segment.vaddr.to_le_bytes()); // p_vaddr
        elf.extend_from_slice(&segment.vaddr.to_le_bytes()); // p_paddr
        elf.extend_from_slice(&file_size.to_le_bytes()); // p_filesz
        elf.extend_from_slice(&mem_size.to_le_bytes()); // p_memsz
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

    // ============ Section Headers ============
    // Pad to section header offset
    if elf.len() < section_header_offset as usize {
        elf.resize(section_header_offset as usize, 0);
    }

    // Section 0: null section
    elf.extend_from_slice(&[0u8; 64]);

    // Section 1: .text (executable code)
    let mut text_offset = 0u64;
    let mut text_size = 0u64;
    let mut exec_vaddr = 0u64;
    for (i, segment) in x86_elf.segments.iter().enumerate() {
        if segment.is_executable {
            text_offset = segment_offsets[i];
            text_size = segment.data.len() as u64;
            exec_vaddr = segment.vaddr;
            break;
        }
    }

    elf.extend_from_slice(&1u32.to_le_bytes()); // sh_name: ".text" at offset 1
    elf.extend_from_slice(&1u32.to_le_bytes()); // sh_type: SHT_PROGBITS
    elf.extend_from_slice(&6u64.to_le_bytes()); // sh_flags: SHF_ALLOC | SHF_EXECINSTR
    elf.extend_from_slice(&exec_vaddr.to_le_bytes()); // sh_addr
    elf.extend_from_slice(&text_offset.to_le_bytes()); // sh_offset
    elf.extend_from_slice(&text_size.to_le_bytes()); // sh_size
    elf.extend_from_slice(&0u32.to_le_bytes()); // sh_link
    elf.extend_from_slice(&0u32.to_le_bytes()); // sh_info
    elf.extend_from_slice(&1u64.to_le_bytes()); // sh_addralign
    elf.extend_from_slice(&0u64.to_le_bytes()); // sh_entsize

    // Section 2: .data (writable data)
    let mut data_offset = 0u64;
    let mut data_size = 0u64;
    let mut data_vaddr = 0u64;
    for (i, segment) in x86_elf.segments.iter().enumerate() {
        if segment.is_writable && !segment.is_executable {
            data_offset = segment_offsets[i];
            data_size = segment.data.len() as u64;
            data_vaddr = segment.vaddr;
            break;
        }
    }

    elf.extend_from_slice(&7u32.to_le_bytes()); // sh_name: ".data" at offset 7
    elf.extend_from_slice(&1u32.to_le_bytes()); // sh_type: SHT_PROGBITS
    elf.extend_from_slice(&3u64.to_le_bytes()); // sh_flags: SHF_ALLOC | SHF_WRITE
    elf.extend_from_slice(&data_vaddr.to_le_bytes()); // sh_addr
    elf.extend_from_slice(&data_offset.to_le_bytes()); // sh_offset
    elf.extend_from_slice(&data_size.to_le_bytes()); // sh_size
    elf.extend_from_slice(&0u32.to_le_bytes()); // sh_link
    elf.extend_from_slice(&0u32.to_le_bytes()); // sh_info
    elf.extend_from_slice(&1u64.to_le_bytes()); // sh_addralign
    elf.extend_from_slice(&0u64.to_le_bytes()); // sh_entsize

    // Section 3: .shstrtab (section header string table)
    elf.extend_from_slice(&13u32.to_le_bytes()); // sh_name: ".shstrtab" at offset 13
    elf.extend_from_slice(&3u32.to_le_bytes()); // sh_type: SHT_STRTAB
    elf.extend_from_slice(&0u64.to_le_bytes()); // sh_flags
    elf.extend_from_slice(&0u64.to_le_bytes()); // sh_addr
    elf.extend_from_slice(&shstrtab_offset.to_le_bytes()); // sh_offset
    elf.extend_from_slice(&(shstrtab_data.len() as u64).to_le_bytes()); // sh_size
    elf.extend_from_slice(&0u32.to_le_bytes()); // sh_link
    elf.extend_from_slice(&0u32.to_le_bytes()); // sh_info
    elf.extend_from_slice(&1u64.to_le_bytes()); // sh_addralign
    elf.extend_from_slice(&0u64.to_le_bytes()); // sh_entsize

    // ============ String Table ============
    elf.extend_from_slice(shstrtab_data);

    Ok(elf)
}

#[cfg(test)]
mod tests {
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
