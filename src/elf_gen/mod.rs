//! ELF file generation for x86-64 binaries
//!
//! This module handles the generation of complete x86-64 ELF executable files
//! from compiled x86-64 bytecode with proper program headers for executability.

pub mod x86_elf;

pub use x86_elf::{X86Elf, X86Segment};

/// Standard page alignment for x86-64 ELF files
const PAGE_ALIGN: u64 = 0x1000;

// Generate an x86_64 Elf binary from a single executable input
pub(crate) fn generate_elf_v2(bytes: &[u8], v_addr: u64) -> Result<Vec<u8>, String> {
    const EHDR_SIZE: usize = 64;
    const PHDR_SIZE: usize = 56;

    let phoff: u64 = EHDR_SIZE as u64;
    let segment_file_offset = (EHDR_SIZE + PHDR_SIZE) as u64;

    // let entry = v_addr
    //     .checked_add(segment_file_offset)
    //     .ok_or_else(|| "entry address overflow".to_string())?;

    let entry = v_addr;

    let segment_size = bytes.len() as u64;

    let mut elf = build_elf_header_x86_64(entry, phoff, 1);

    elf.resize(EHDR_SIZE + PHDR_SIZE, 0);

    write_phdr_x86_64_single_load(
        &mut elf,
        EHDR_SIZE,
        segment_file_offset,
        v_addr,
        segment_size,
        segment_size,
    );

    elf.extend_from_slice(bytes);

    Ok(elf)
}

fn build_elf_header_x86_64(
    entry: u64,
    program_header_offset: u64,
    program_header_num: u16,
) -> Vec<u8> {
    let mut elf = vec![0; 64];

    // e_ident[16]
    elf[0] = 0x7f;
    elf[1] = b'E';
    elf[2] = b'L';
    elf[3] = b'F'; // magic
    elf[4] = 2; // EI_CLASS   = ELFCLASS64
    elf[5] = 1; // EI_DATA    = ELFDATA2LSB
    elf[6] = 1; // EI_VERSION = EV_CURRENT
    elf[7] = 0; // EI_OSABI   = ELFOSABI_SYSV
    elf[8] = 0; // EI_ABIVERSION
                // elf[9..16] left as 0 padding
    put_u16_le(&mut elf, 16, 2); // e_type      = ET_EXEC
    put_u16_le(&mut elf, 18, 62); // e_machine   = EM_X86_64
    put_u32_le(&mut elf, 20, 1); // e_version   = EV_CURRENT
    put_u64_le(&mut elf, 24, entry); // e_entry
    put_u64_le(&mut elf, 32, program_header_offset); // e_phoff
    put_u64_le(&mut elf, 40, 0); // e_shoff    = 0 (no sections)
    put_u32_le(&mut elf, 48, 0); // e_flags    = 0 for x86-64
    put_u16_le(&mut elf, 52, 64); // e_ehsize   = sizeof(Elf64_Ehdr)
    put_u16_le(&mut elf, 54, 56); // e_phentsize= sizeof(Elf64_Phdr)
    put_u16_le(&mut elf, 56, program_header_num); // e_phnum
    put_u16_le(&mut elf, 58, 0); // e_shentsize
    put_u16_le(&mut elf, 60, 0); // e_shnum
    put_u16_le(&mut elf, 62, 0); // e_shstrndx
    elf
}

fn write_phdr_x86_64_single_load(
    elf: &mut [u8],
    phoff: usize,
    segment_file_offset: u64, // where code bytes begin in file, e.g. 0x78
    segment_vaddr: u64,       // where segment is mapped, e.g. 0x400000
    segment_file_size: u64,   // code length in file
    segment_mem_size: u64,    // code length in memory (same for code-only)
) {
    // Elf64_Phdr layout:
    // 0x00 p_type   (u32)
    // 0x04 p_flags  (u32)
    // 0x08 p_offset (u64)
    // 0x10 p_vaddr  (u64)
    // 0x18 p_paddr  (u64)
    // 0x20 p_filesz (u64)
    // 0x28 p_memsz  (u64)
    // 0x30 p_align  (u64)
    put_u32_le(elf, phoff + 0x00, 1); // PT_LOAD
    put_u32_le(elf, phoff + 0x04, 0x5); // PF_R | PF_X
    put_u64_le(elf, phoff + 0x08, segment_file_offset);
    put_u64_le(elf, phoff + 0x10, segment_vaddr);
    put_u64_le(elf, phoff + 0x18, 0); // p_paddr (ignored on Linux)
    put_u64_le(elf, phoff + 0x20, segment_file_size);
    put_u64_le(elf, phoff + 0x28, segment_mem_size);
    put_u64_le(elf, phoff + 0x30, 0x1000); // page alignment
}

fn put_u16_le(buf: &mut [u8], off: usize, v: u16) {
    buf[off..off + 2].copy_from_slice(&v.to_le_bytes());
}

fn put_u32_le(buf: &mut [u8], off: usize, v: u32) {
    buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn put_u64_le(buf: &mut [u8], off: usize, v: u64) {
    buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

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
    let mut segment_offsets: Vec<u64> = Vec::with_capacity(num_segments);
    let mut current_offset = headers_size as u64;

    for segment in &x86_elf.segments {
        if segment.data.is_empty() && segment.mem_size == 0 {
            // BSS segment - no file space needed
            segment_offsets.push(0);
        } else if !segment.data.is_empty() {
            // Align to page boundary if executable
            if segment.is_executable {
                current_offset = ((current_offset + PAGE_ALIGN - 1) / PAGE_ALIGN) * PAGE_ALIGN;
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

#[cfg(test)]
mod tests {
    use std::fs;

    use dynasmrt::{dynasm, x64, x86, DynasmApi};

    use super::*;

    #[test]
    fn generate_halt_exec() {
        let mut ops = x64::Assembler::new().unwrap();
        dynasm!(ops ; mov rax, 60);
        dynasm!(ops ; mov rdi, 0);
        dynasm!(ops ; syscall);
        let res = ops.finalize().unwrap().to_vec();
        let elf_bytes = generate_elf_v2(&res, 0x400078).unwrap();
        fs::write("./test-bin/halt.elf", elf_bytes).unwrap();

        let mut m = X86Elf::new(0x400000);
        m.add_text(res, 0x40000, 897374939);
        let elf_bytes = generate_elf(&m).unwrap();
        fs::write("./test-bin/halt2.elf", elf_bytes).unwrap();
    }

    #[test]
    fn generate_echo_exec() {
        let mut ops = x86::Assembler::new().unwrap();
        dynasm!(
            ops;

            // read the user input
            mov rax, 0;
            mov rdi, 0;
            mov rsi, rsp;
            mov rdx, 10;
            syscall;

            // write to screen
            mov rax, 1;
            mov rdi, 1;
            mov rsi, rsp;
            mov rdx, 10;
            syscall;

            // halt
            mov rax, 60;
            mov rdi, 0;
            syscall
        );
        let res = ops.finalize().unwrap().to_vec();
        let elf_bytes = generate_elf_v2(&res, 0x400078).unwrap();
        fs::write("./test-bin/echo.elf", elf_bytes).unwrap();
    }

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
