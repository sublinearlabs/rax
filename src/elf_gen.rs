//! ELF file generation for x86-64 binaries
//!
//! This module handles the generation of complete x86-64 ELF executable files
//! from compiled x86-64 bytecode with proper program headers for executability.

/// Configuration for ELF generation
#[derive(Debug, Clone)]
pub struct ElfConfig {
    /// Entry point address (typically 0x400000 for x86-64 executables)
    pub entry_point: u64,

    /// Virtual address for .text section
    pub text_vaddr: u64,

    /// Virtual address for .data section
    pub data_vaddr: u64,

    /// Virtual address for .bss section
    pub bss_vaddr: u64,

    /// Page alignment (typically 0x1000 = 4KB)
    pub page_align: u64,
}

impl Default for ElfConfig {
    fn default() -> Self {
        ElfConfig {
            entry_point: 0x400000,
            text_vaddr: 0x400000,
            data_vaddr: 0x600000,
            bss_vaddr: 0x601000,
            page_align: 0x1000,
        }
    }
}

/// ELF binary builder
pub struct ElfBuilder {
    config: ElfConfig,
    text_data: Vec<u8>,
    data_data: Vec<u8>,
}

impl ElfBuilder {
    /// Create a new ELF builder with default configuration
    pub fn new() -> Self {
        Self::with_config(ElfConfig::default())
    }

    /// Create a new ELF builder with custom configuration
    pub fn with_config(config: ElfConfig) -> Self {
        ElfBuilder {
            config,
            text_data: Vec::new(),
            data_data: Vec::new(),
        }
    }

    /// Add x86-64 bytecode to the .text section
    pub fn add_text(&mut self, bytecode: Vec<u8>) -> &mut Self {
        self.text_data = bytecode;
        self
    }

    /// Add data to the .data section
    pub fn add_data(&mut self, data: Vec<u8>) -> &mut Self {
        self.data_data = data;
        self
    }

    /// Build the complete ELF binary with proper program headers
    pub fn build(self) -> Result<Vec<u8>, String> {
        let mut elf = Vec::new();

        // Text section will be aligned to page boundary
        // Size: ELF header (64) + Program header (56) + padding + text data
        let text_offset = 64 + 56; // After ELF header and program header
        let text_offset_aligned = ((text_offset + self.config.page_align as usize - 1)
            / self.config.page_align as usize)
            * self.config.page_align as usize;

        // ============ ELF Header (64 bytes) ============
        // e_ident[16] - Identification bytes
        elf.extend_from_slice(b"\x7FELF"); // Magic number
        elf.push(2); // e_ident[4]: ELFCLASS64
        elf.push(1); // e_ident[5]: ELFDATA2LSB (little-endian)
        elf.push(1); // e_ident[6]: EV_CURRENT
        elf.push(0); // e_ident[7]: ELFOSABI_SYSV
        elf.push(0); // e_ident[8]: ABI version
        elf.extend_from_slice(&[0; 7]); // e_ident[9:16] - padding

        // e_type (u16): ET_EXEC = 2 (executable file)
        elf.extend_from_slice(&2u16.to_le_bytes());

        // e_machine (u16): EM_X86_64 = 62
        elf.extend_from_slice(&62u16.to_le_bytes());

        // e_version (u32): EV_CURRENT = 1
        elf.extend_from_slice(&1u32.to_le_bytes());

        // e_entry (u64): Entry point address (where .text actually starts in memory)
        elf.extend_from_slice(&self.config.text_vaddr.to_le_bytes());

        // e_phoff (u64): Offset to program header table (right after ELF header)
        let e_phoff: u64 = 64;
        elf.extend_from_slice(&e_phoff.to_le_bytes());

        // e_shoff (u64): Offset to section header table (we don't have sections)
        elf.extend_from_slice(&0u64.to_le_bytes());

        // e_flags (u32): Processor-specific flags
        elf.extend_from_slice(&0u32.to_le_bytes());

        // e_ehsize (u16): ELF header size = 64
        elf.extend_from_slice(&64u16.to_le_bytes());

        // e_phentsize (u16): Program header entry size = 56 (for 64-bit)
        elf.extend_from_slice(&56u16.to_le_bytes());

        // e_phnum (u16): Number of program header entries
        let num_ph = if !self.text_data.is_empty() { 1 } else { 0 };
        elf.extend_from_slice(&(num_ph as u16).to_le_bytes());

        // e_shentsize (u16): Section header entry size
        elf.extend_from_slice(&0u16.to_le_bytes());

        // e_shnum (u16): Number of section header entries
        elf.extend_from_slice(&0u16.to_le_bytes());

        // e_shstrndx (u16): Section header string table index
        elf.extend_from_slice(&0u16.to_le_bytes());

        // ============ Program Headers ============
        // PT_LOAD segment for .text
        if !self.text_data.is_empty() {
            // Program Header for .text (56 bytes)
            // p_type (u32): PT_LOAD = 1
            elf.extend_from_slice(&1u32.to_le_bytes());

            // p_flags (u32): PF_X | PF_R = 5 (execute | read)
            elf.extend_from_slice(&5u32.to_le_bytes());

            // p_offset (u64): Offset in file where segment starts
            elf.extend_from_slice(&(text_offset_aligned as u64).to_le_bytes());

            // p_vaddr (u64): Virtual address where segment is loaded
            elf.extend_from_slice(&self.config.text_vaddr.to_le_bytes());

            // p_paddr (u64): Physical address (same as vaddr for executables)
            elf.extend_from_slice(&self.config.text_vaddr.to_le_bytes());

            // p_filesz (u64): Size in file
            elf.extend_from_slice(&(self.text_data.len() as u64).to_le_bytes());

            // p_memsz (u64): Size in memory (same as filesz for .text)
            elf.extend_from_slice(&(self.text_data.len() as u64).to_le_bytes());

            // p_align (u64): Alignment
            elf.extend_from_slice(&self.config.page_align.to_le_bytes());
        }

        // Pad to aligned offset with zeros
        let padding_size = text_offset_aligned - elf.len();
        elf.extend_from_slice(&vec![0; padding_size]);

        // Append the text data at the aligned offset
        elf.extend_from_slice(&self.text_data);

        Ok(elf)
    }
}

impl Default for ElfBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_builder_creation() {
        let builder = ElfBuilder::new();
        assert_eq!(builder.config.entry_point, 0x400000);
    }

    #[test]
    fn test_elf_builder_with_config() {
        let config = ElfConfig {
            entry_point: 0x500000,
            ..Default::default()
        };
        let builder = ElfBuilder::with_config(config);
        assert_eq!(builder.config.entry_point, 0x500000);
    }

    #[test]
    fn test_minimal_elf_generation() {
        let mut builder = ElfBuilder::new();

        // Add minimal x86-64 code: RET instruction (0xC3)
        builder.add_text(vec![0xC3]);

        let result = builder.build();
        assert!(result.is_ok());

        let elf_binary = result.unwrap();
        assert!(!elf_binary.is_empty());

        // Check ELF magic number (0x7F 'E' 'L' 'F')
        assert_eq!(&elf_binary[0..4], b"\x7FELF");

        // Check it's 64-bit little-endian
        assert_eq!(elf_binary[4], 2); // ELFCLASS64
        assert_eq!(elf_binary[5], 1); // ELFDATA2LSB (little-endian)
    }

    #[test]
    fn test_elf_generation_with_entry_point() {
        let config = ElfConfig {
            entry_point: 0x400100,
            ..Default::default()
        };
        let mut builder = ElfBuilder::with_config(config);
        builder.add_text(vec![0x90]); // NOP

        let result = builder.build();
        assert!(result.is_ok());
    }
}
