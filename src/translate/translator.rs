//! RISC-V to x86-64 translator
//!
//! This module handles the translation of RISC-V instructions to x86-64 bytecode.
//! It manages translation state, register allocation, and instruction dispatch.

use crate::decode::Instruction as RiscvInstruction;
use crate::translate::x86_emitter::X86Emitter;
use crate::translate::x86_insn::X86Instruction;
use crate::translate::{instruction_translator, RegisterMapper};

/// PC mapping from RISC-V PCs to x86-64 bytecode offsets
/// Uses direct indexing: index = (riscv_pc - entry_point) / 4
#[derive(Debug, Clone)]
pub struct PcMapping {
    /// The entry point RISC-V PC (first instruction)
    pub entry_point: u64,

    /// Array of x86-64 bytecode offsets, indexed by (riscv_pc - entry_point) / 4
    pub offsets: Vec<u64>,
}

impl PcMapping {
    /// Create a new empty PC mapping
    pub fn new(entry_point: u64) -> Self {
        PcMapping {
            entry_point,
            offsets: Vec::new(),
        }
    }

    /// Add a mapping for a RISC-V PC to an x86-64 offset
    pub fn add_mapping(&mut self, riscv_pc: u64, x86_offset: u64) {
        // Calculate the index
        let index = ((riscv_pc - self.entry_point) / 4) as usize;

        // Ensure the vector is large enough
        if index >= self.offsets.len() {
            self.offsets.resize(index + 1, 0);
        }

        // Store the offset
        self.offsets[index] = x86_offset;
    }

    /// Look up the x86-64 offset for a RISC-V PC
    pub fn lookup(&self, riscv_pc: u64) -> Option<u64> {
        let index = ((riscv_pc - self.entry_point) / 4) as usize;
        self.offsets.get(index).copied()
    }
}

/// Translation context
#[derive(Debug, Clone)]
pub struct TranslationContext<M: RegisterMapper> {
    /// Entry point RISC-V PC (first instruction)
    pub entry_point: u64,

    /// Current program counter
    pub pc: u64,

    /// Register mapping
    pub register_mapping: M,

    /// PC mapping from RISC-V to x86-64
    pub pc_mapping: PcMapping,
}

/// Main RISC-V to x86-64 translator
#[derive(Debug, Clone)]
pub struct RiscvToX86Translator<M: RegisterMapper> {
    /// Bytecode emitter
    pub emitter: X86Emitter,

    /// Translation context
    pub(crate) context: TranslationContext<M>,
}

impl<M: RegisterMapper> RiscvToX86Translator<M> {
    pub fn new(pc: u64, code_base: u64, bss_base: u64) -> Self {
        // Create a new register mapping
        let register_mapping = M::new(bss_base);

        RiscvToX86Translator {
            emitter: X86Emitter::new(code_base),
            context: TranslationContext {
                entry_point: pc,
                pc,
                register_mapping,
                pc_mapping: PcMapping::new(pc),
            },
        }
    }

    /// Get the translation context
    pub fn context(&self) -> &TranslationContext<M> {
        &self.context
    }

    /// Get mutable translation context
    pub fn context_mut(&mut self) -> &mut TranslationContext<M> {
        &mut self.context
    }

    /// Record a RISC-V PC to x86-64 bytecode offset mapping
    pub fn record_pc_mapping(&mut self, riscv_pc: u64, x86_offset: u64) {
        self.context.pc_mapping.add_mapping(riscv_pc, x86_offset);
    }

    /// Get the PC mapping table
    pub fn get_pc_mapping(&self) -> &PcMapping {
        &self.context.pc_mapping
    }

    /// Get the emitter
    pub fn emitter(&self) -> &X86Emitter {
        &self.emitter
    }

    /// Get mutable emitter
    pub fn emitter_mut(&mut self) -> &mut X86Emitter {
        &mut self.emitter
    }

    /// Complete translation
    pub fn finish(&mut self) -> Result<(), String> {
        self.emitter.apply_relocations()?;
        Ok(())
    }

    /// Emit an instruction
    pub fn emit_instruction(&mut self, instruction: &X86Instruction) -> Result<(), String> {
        match instruction {
            X86Instruction::Mov { src, dst } => self.emitter.emit_mov(src, dst),
            X86Instruction::Movzx { src, dst } => self.emitter.emit_movzx(src, dst),
            X86Instruction::Add { src, dst } => self.emitter.emit_add(src, dst),
            X86Instruction::Sub { src, dst } => self.emitter.emit_sub(src, dst),
            X86Instruction::And { src, dst } => self.emitter.emit_and(src, dst),
            X86Instruction::Or { src, dst } => self.emitter.emit_or(src, dst),
            X86Instruction::Xor { src, dst } => self.emitter.emit_xor(src, dst),
            X86Instruction::Cmp { src, dst } => self.emitter.emit_cmp(src, dst),
            X86Instruction::Jmp { target } => self.emitter.emit_jmp(target),
            X86Instruction::JmpReg { target } => self.emitter.emit_jmp_reg(target),
            X86Instruction::Je { target } => self.emitter.emit_je(target),
            X86Instruction::Jne { target } => self.emitter.emit_jne(target),
            X86Instruction::Jl { target } => self.emitter.emit_jl(target),
            X86Instruction::Jle { target } => self.emitter.emit_jle(target),
            X86Instruction::Jg { target } => self.emitter.emit_jg(target),
            X86Instruction::Jge { target } => self.emitter.emit_jge(target),
            X86Instruction::Jb { target } => self.emitter.emit_jb(target),
            X86Instruction::Jbe { target } => self.emitter.emit_jbe(target),
            X86Instruction::Ja { target } => self.emitter.emit_ja(target),
            X86Instruction::Jae { target } => self.emitter.emit_jae(target),
            X86Instruction::Ret => {
                self.emitter.emit_ret();
                Ok(())
            }
            X86Instruction::Syscall => {
                self.emitter.emit_syscall();
                Ok(())
            }
            X86Instruction::Push { src } => self.emitter.emit_push(src),
            X86Instruction::Pop { dst } => self.emitter.emit_pop(dst),
            X86Instruction::Shl { src, dst } => self.emitter.emit_shl(src, dst),
            X86Instruction::Shr { src, dst } => self.emitter.emit_shr(src, dst),
            X86Instruction::Sar { src, dst } => self.emitter.emit_sar(src, dst),
            X86Instruction::Setl { dst } => self.emitter.emit_setl(dst),
            X86Instruction::Setle { dst } => self.emitter.emit_setle(dst),
            X86Instruction::Setg { dst } => self.emitter.emit_setg(dst),
            X86Instruction::Setge { dst } => self.emitter.emit_setge(dst),
            X86Instruction::Sete { dst } => self.emitter.emit_sete(dst),
            X86Instruction::Setne { dst } => self.emitter.emit_setne(dst),
            X86Instruction::Setb { dst } => self.emitter.emit_setb(dst),
            X86Instruction::Setbe { dst } => self.emitter.emit_setbe(dst),
            X86Instruction::Label { name } => {
                self.emitter.emit_label(name.clone());
                Ok(())
            }
            X86Instruction::Nop => {
                self.emitter.emit_byte(0x90);
                Ok(())
            }
            X86Instruction::Imul { src, dst } => self.emitter.emit_imul(src, dst),
            X86Instruction::Mul { src } => self.emitter.emit_mul(src),
            _ => Err(format!(
                "Instruction not yet implemented: {:?}",
                instruction
            )),
        }
    }

    /// Get the bytecode
    pub fn get_bytecode(&self) -> &[u8] {
        self.emitter.get_buffer()
    }

    /// Process a RISC-V instruction through translation
    ///
    /// This is the main entry point for translating individual RISC-V instructions.
    /// Returns an error if the instruction is not yet supported.
    pub fn process_instruction(&mut self, riscv_insn: &RiscvInstruction) -> Result<(), String> {
        // Emit a label for the current riscv pc to enable relocation
        let current_riscv_pc = self.context.pc;
        let label = format!("L_{:x}", current_riscv_pc);
        self.emitter.emit_label(label);

        // Record the PC mapping (RISC-V PC → x86-64 offset)
        // We map after the label is emitted
        let x86_offset_after_label = self.emitter.offset() as u64;
        self.record_pc_mapping(current_riscv_pc, x86_offset_after_label);

        // Translate the instruction
        instruction_translator::translate_instruction(self, riscv_insn)?;

        // Move to next RISC-V instruction (always 4 bytes since c extension is not supported yet)
        self.context.pc += 4;
        Ok(())
    }
}

impl<M: RegisterMapper> Default for RiscvToX86Translator<M> {
    fn default() -> Self {
        Self::new(0, 0x400000u64, 0x601000u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aot::register_mapping::RegisterMapping;

    #[test]
    fn test_emit_nop() {
        let mut translator: RiscvToX86Translator<RegisterMapping> = RiscvToX86Translator::default();

        translator.emit_instruction(&X86Instruction::Nop).unwrap();
        assert_eq!(translator.get_bytecode()[0], 0x90);
    }

    #[test]
    fn test_register_mapping_access() {
        let translator: RiscvToX86Translator<RegisterMapping> = RiscvToX86Translator::default();

        let _mapping = &translator.context().register_mapping;
        // Verify mapping was created
    }

    #[test]
    fn test_emit_ret() {
        let mut translator: RiscvToX86Translator<RegisterMapping> = RiscvToX86Translator::default();

        translator.emit_instruction(&X86Instruction::Ret).unwrap();
        assert_eq!(translator.get_bytecode()[0], 0xC3);
    }
}
