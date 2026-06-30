use elf::{
    abi::{EM_RISCV, ET_EXEC, PF_X, PT_LOAD},
    endian::LittleEndian,
    file::Class,
    ElfBytes,
};

use riscv_core::decode::{decode, Instruction};
use super::memory::MemoryDefault;

/// Decodes the elf bytes,
/// loads segments into memory and return the pc.
pub(crate) fn decode_elf(bytes: &[u8]) -> (MemoryDefault, u64) {
    let file =
        ElfBytes::<LittleEndian>::minimal_parse(bytes).expect("failed to parse the elf file");
    let ehdr = file.ehdr;

    assert_eq!(ehdr.class, Class::ELF64);
    assert_eq!(ehdr.e_type, ET_EXEC);
    assert_eq!(ehdr.e_machine, EM_RISCV);

    let entry = ehdr.e_entry;

    // load the program headers into memory
    let mut memory = MemoryDefault::default();

    // iterate over the program headers
    // load header of type `PT_LOAD` to memory
    let segments = file.segments().expect("has no program headers");
    for ph in segments.iter() {
        if ph.p_type != PT_LOAD {
            continue;
        }

        let offset = ph.p_offset as usize;
        let filesz = ph.p_filesz as usize;
        let vaddr = ph.p_vaddr;
        let memsz = ph.p_memsz as usize;

        if memsz < filesz {
            panic!("malformed elf file");
        }

        if filesz > 0 {
            let data = &bytes[offset..offset + filesz];
            memory.write_n_bytes(vaddr, data);
        }

        if memsz > filesz {
            memory.zero_fill(vaddr + filesz as u64, memsz - filesz);
        }
    }

    (memory, entry)
}

pub(crate) struct Segment {
    data: Vec<u8>,
    insns: Vec<Instruction>,
    entry: u64,
    offset: usize,
    file_size: usize,
    mem_size: usize,
    is_executable: bool,
}

impl Segment {
    pub(crate) fn new(
        data: Vec<u8>,
        insns: Vec<Instruction>,
        entry: u64,
        offset: usize,
        file_size: usize,
        mem_size: usize,
        is_executable: bool,
    ) -> Self {
        Self {
            data,
            insns,
            entry,
            offset,
            file_size,
            mem_size,
            is_executable,
        }
    }

    pub(crate) fn decode(&mut self) {
        // Only decode executable segments
        if !self.is_executable {
            return;
        }

        let mut instructions = Vec::new();

        // TODO: For now, we assume we are not working with C extension
        for chunk in self.data.chunks(4) {
            let mut insn_bytes = [0u8; 4];
            // Copy available bytes and pad with zeros if incomplete
            for (i, &byte) in chunk.iter().enumerate() {
                insn_bytes[i] = byte;
            }
            let insn_u32 = u32::from_le_bytes(insn_bytes);
            let insn = decode(insn_u32);
            instructions.push(insn);
        }

        self.insns = instructions;
    }

    pub(crate) fn is_executable(&self) -> bool {
        self.is_executable
    }

    pub(crate) fn insns(&self) -> &[Instruction] {
        &self.insns
    }

    pub(crate) fn entry(&self) -> u64 {
        self.entry
    }

    pub(crate) fn offset(&self) -> usize {
        self.offset
    }
}

pub(crate) struct Elf {
    pub(crate) segments: Vec<Segment>,
    pub(crate) global_entry: u64,
}

impl Elf {
    pub(crate) fn new(segments: Vec<Segment>, global_entry: u64) -> Self {
        Self {
            segments,
            global_entry,
        }
    }
}

/// Parses the elf file
pub(crate) fn parse_elf(bytes: &[u8]) -> Elf {
    let file =
        ElfBytes::<LittleEndian>::minimal_parse(bytes).expect("failed to parse the elf file");
    let ehdr = file.ehdr;

    assert_eq!(ehdr.class, Class::ELF64);
    assert_eq!(ehdr.e_type, ET_EXEC);
    assert_eq!(ehdr.e_machine, EM_RISCV);

    let global_entry = ehdr.e_entry;
    let mut parsed_segments: Vec<Segment> = vec![];

    // Iterate over all program headers
    // load header of type `PT_LOAD` to memory
    let segments = file.segments().expect("has no program headers");
    for ph in segments.iter() {
        if ph.p_type != PT_LOAD {
            continue;
        }

        let offset = ph.p_offset as usize;
        let filesz = ph.p_filesz as usize;
        let vaddr = ph.p_vaddr;
        let memsz = ph.p_memsz as usize;
        let is_executable = (ph.p_flags & PF_X) != 0;

        if memsz < filesz {
            panic!("malformed elf file");
        }

        if filesz > 0 {
            let data = &bytes[offset..offset + filesz];
            let value = Segment::new(
                data.to_vec(),
                vec![],
                vaddr,
                offset,
                filesz,
                memsz,
                is_executable,
            );
            parsed_segments.push(value);
        }
    }

    Elf::new(parsed_segments, global_entry)
}

#[cfg(test)]
mod tests {
    use riscv_core::decode::I;

    use super::*;

    #[test]
    fn test_segment_decode_empty() {
        // Test with empty data
        let mut segment = Segment::new(vec![], vec![], 0, 0, 0, 0, true);
        segment.decode();
        assert_eq!(segment.insns.len(), 0);
    }

    #[test]
    fn test_segment_decode_single_instruction() {
        // Test with a single 4-byte instruction
        // Example: ADDI x2, x1, 164 (0x0a408113 in little-endian)
        let data = vec![0x13, 0x81, 0x40, 0x0A];
        let mut segment = Segment::new(data, vec![], 0, 0, 4, 4, true);
        segment.decode();
        assert_eq!(segment.insns.len(), 1);
        assert_eq!(
            segment.insns[0],
            Instruction::Addi(I {
                rd: 2,
                rs1: 1,
                imm: 164
            })
        );
    }

    #[test]
    fn test_segment_decode_multiple_instructions() {
        // Test with multiple 4-byte instructions
        let mut data = Vec::new();
        // Add 3 instructions (12 bytes total)
        data.extend_from_slice(&[0x93, 0x80, 0x40, 0x0A]); // Instruction 1
        data.extend_from_slice(&[0x13, 0x81, 0x41, 0x0B]); // Instruction 2
        data.extend_from_slice(&[0x33, 0x82, 0x42, 0x0C]); // Instruction 3

        let mut segment = Segment::new(data, vec![], 0, 0, 12, 12, true);
        segment.decode();
        assert_eq!(segment.insns.len(), 3);
    }

    #[test]
    fn test_segment_decode_incomplete_chunk() {
        // Test with data that's not a multiple of 4
        // Incomplete chunks should be padded with zeros and decoded
        let data = vec![0x93, 0x80, 0x40, 0x0A, 0x13, 0x81]; // 6 bytes = 1 complete + 2 incomplete
        let mut segment = Segment::new(data, vec![], 0, 0, 6, 6, true);
        segment.decode();
        // Should decode 2 instructions: 1 complete + 1 padded with zeros
        assert_eq!(segment.insns.len(), 2);
        // First instruction is the complete one: ADDI x1, x1, 164
        assert_eq!(
            segment.insns[0],
            Instruction::Addi(I {
                rd: 1,
                rs1: 1,
                imm: 164
            })
        );
        // Second instruction is padded: [0x13, 0x81, 0x00, 0x00]
    }

    #[test]
    fn test_segment_decode_little_endian_conversion() {
        // Test that little-endian conversion works correctly
        // Bytes [0x93, 0x80, 0x40, 0x0A] should convert to u32: 0x0A408093
        // This is ADDI x1, x1, 164 (rd=1, rs1=1, imm=164)
        let data = vec![0x93, 0x80, 0x40, 0x0A];
        let mut segment = Segment::new(data, vec![], 0, 0, 4, 4, true);
        segment.decode();
        assert_eq!(segment.insns.len(), 1);
        assert_eq!(
            segment.insns[0],
            Instruction::Addi(I {
                rd: 1,
                rs1: 1,
                imm: 164
            })
        );
    }

    #[test]
    fn test_segment_with_custom_entry() {
        // Test segment with a custom entry point
        let data = vec![
            0x93, 0x80, 0x40, 0x0A, // Instruction 1
            0x13, 0x81, 0x41, 0x0B, // Instruction 2
        ];
        let mut segment = Segment::new(data.clone(), vec![], 0x1000, 0, 8, 8, true);
        segment.decode();
        assert_eq!(segment.insns.len(), 2);
        assert_eq!(segment.entry, 0x1000);
    }

    #[test]
    fn test_segment_non_executable_early_return() {
        // Test that decode() returns early without decoding when is_executable=false
        let data = vec![0x93, 0x80, 0x40, 0x0A]; // Would be 1 instruction if decoded
        let mut segment = Segment::new(data, vec![], 0, 0, 4, 4, false);
        segment.decode();
        // Should remain empty because segment is not executable
        assert_eq!(segment.insns.len(), 0);
    }

    #[test]
    fn test_segment_non_executable_with_multiple_chunks() {
        // Test that non-executable segments are not decoded even with multiple chunks
        let mut data = Vec::new();
        data.extend_from_slice(&[0x93, 0x80, 0x40, 0x0A]); // Would be 3 instructions
        data.extend_from_slice(&[0x13, 0x81, 0x41, 0x0B]); // if decoded
        data.extend_from_slice(&[0x33, 0x82, 0x42, 0x0C]);
        let mut segment = Segment::new(data, vec![], 0, 0, 12, 12, false);
        segment.decode();
        // Should remain empty because segment is not executable
        assert_eq!(segment.insns.len(), 0);
    }
}
