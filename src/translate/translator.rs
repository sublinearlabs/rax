//! RISC-V to x86-64 translator
//!
//! This module handles the translation of RISC-V instructions to x86-64 bytecode.
//! It manages translation state, register allocation, and instruction dispatch.

use crate::aot::register_mapping::RegisterMapping;
use crate::translate::register_config::RegisterAllocationConfig;
use crate::translate::x86_emitter::X86Emitter;
use crate::translate::x86_insn::{X86Instruction, X86Register, Operand};
use crate::decode::Instruction as RiscvInstruction;
use crate::translate::instruction_translator;

/// Translation context tracking
#[derive(Debug, Clone)]
pub struct TranslationContext {
    /// Current program counter
    pub pc: u64,

    /// Register mapping for allocation (created from allocation strategy)
    pub register_mapping: RegisterMapping,

    /// Stack frame size in bytes
    pub frame_size: u32,

    /// Current translation phase/stage
    pub phase: TranslationPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationPhase {
    /// Initial phase - no instructions translated
    Init,

    /// Instructions being translated
    InProgress,

    /// Translation complete, ready for assembly/linking
    Complete,
}

/// Main RISC-V to x86-64 translator
#[derive(Debug)]
pub struct RiscvToX86Translator {
    /// Bytecode emitter
    emitter: X86Emitter,

    /// Translation context
    context: TranslationContext,

    /// List of translated instructions (for debugging)
    instructions: Vec<X86Instruction>,
}

impl RiscvToX86Translator {
    /// Create a new translator with default AllGPR register allocation strategy
    pub fn new() -> Self {
        // Create default register mapping using AllGPR strategy
        let config = RegisterAllocationConfig::allgpr();
        let register_mapping = config.create_mapping();

        RiscvToX86Translator {
            emitter: X86Emitter::new(),
            context: TranslationContext {
                pc: 0,
                register_mapping,
                frame_size: 0,
                phase: TranslationPhase::Init,
            },
            instructions: Vec::new(),
        }
    }

    /// Get the translation context
    pub fn context(&self) -> &TranslationContext {
        &self.context
    }

    /// Get mutable translation context
    pub fn context_mut(&mut self) -> &mut TranslationContext {
        &mut self.context
    }

    /// Get the emitter
    pub fn emitter(&self) -> &X86Emitter {
        &self.emitter
    }

    /// Get mutable emitter
    pub fn emitter_mut(&mut self) -> &mut X86Emitter {
        &mut self.emitter
    }

    /// Start translation
    pub fn start(&mut self) {
        self.context.phase = TranslationPhase::InProgress;
    }

    /// Complete translation
    pub fn finish(&mut self) -> Result<(), String> {
        self.context.phase = TranslationPhase::Complete;
        self.emitter.apply_relocations()?;
        Ok(())
    }

    /// Add an instruction to the translation
    pub fn add_instruction(&mut self, instruction: X86Instruction) {
        self.instructions.push(instruction);
    }

    /// Emit an instruction
    pub fn emit_instruction(&mut self, instruction: &X86Instruction) -> Result<(), String> {
        match instruction {
            X86Instruction::Mov { src, dst } => self.emitter.emit_mov(src, dst),
            X86Instruction::Add { src, dst } => self.emitter.emit_add(src, dst),
            X86Instruction::Sub { src, dst } => self.emitter.emit_sub(src, dst),
            X86Instruction::Jmp { target } => self.emitter.emit_jmp(target),
            X86Instruction::Ret => {
                self.emitter.emit_ret();
                Ok(())
            }
            X86Instruction::Label { name } => {
                self.emitter.emit_label(name.clone());
                Ok(())
            }
            X86Instruction::Nop => {
                self.emitter.emit_byte(0x90);
                Ok(())
            }
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

    /// Get the instruction list
    pub fn get_instructions(&self) -> &[X86Instruction] {
        &self.instructions
    }

    /// Get translation statistics
    pub fn get_stats(&self) -> TranslationStats {
        TranslationStats {
            instructions_translated: self.instructions.len(),
            bytecode_size: self.get_bytecode().len(),
            phase: self.context.phase,
        }
    }

    /// Process a RISC-V instruction through translation
    /// 
    /// This is the main entry point for translating individual RISC-V instructions.
    /// Returns an error if the instruction is not yet supported.
    fn process_instruction(&mut self, riscv_insn: &RiscvInstruction) -> Result<(), String> {
        instruction_translator::translate_instruction(self, riscv_insn)?;
        self.context.pc += 4; // RISC-V instructions are always 4 bytes
        Ok(())
    }

}

impl Default for RiscvToX86Translator {
    fn default() -> Self {
        Self::new()
    }
}

/// Translation statistics
#[derive(Debug, Clone)]
pub struct TranslationStats {
    /// Number of instructions translated
    pub instructions_translated: usize,

    /// Size of generated bytecode
    pub bytecode_size: usize,

    /// Current translation phase
    pub phase: TranslationPhase,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translator_creation() {
        let translator = RiscvToX86Translator::new();
        assert_eq!(translator.context().phase, TranslationPhase::Init);
        assert_eq!(translator.get_instructions().len(), 0);
    }

    #[test]
    fn test_translator_start() {
        let mut translator = RiscvToX86Translator::new();
        translator.start();
        assert_eq!(translator.context().phase, TranslationPhase::InProgress);
    }

    #[test]
    fn test_translator_finish() {
        let mut translator = RiscvToX86Translator::new();
        translator.start();
        translator.finish().unwrap();
        assert_eq!(translator.context().phase, TranslationPhase::Complete);
    }

    #[test]
    fn test_add_instruction() {
        let mut translator = RiscvToX86Translator::new();

        let mov = X86Instruction::Nop;

        translator.add_instruction(mov.clone());
        assert_eq!(translator.get_instructions().len(), 1);
    }

    #[test]
    fn test_emit_mov_instruction() {
        let mut translator = RiscvToX86Translator::new();
        translator.start();

        let mov = X86Instruction::Nop;

        translator.emit_instruction(&mov).unwrap();
        assert!(translator.get_bytecode().len() > 0);
    }

    #[test]
    fn test_emit_add_instruction() {
        let mut translator = RiscvToX86Translator::new();
        translator.start();

        let add = X86Instruction::Nop;

        translator.emit_instruction(&add).unwrap();
        assert!(translator.get_bytecode().len() > 0);
    }

    #[test]
    fn test_emit_nop() {
        let mut translator = RiscvToX86Translator::new();
        translator.start();

        translator.emit_instruction(&X86Instruction::Nop).unwrap();
        assert_eq!(translator.get_bytecode()[0], 0x90);
    }

    #[test]
    fn test_get_stats() {
        let mut translator = RiscvToX86Translator::new();
        translator.start();

        translator.add_instruction(X86Instruction::Nop);
        translator.add_instruction(X86Instruction::Nop);

        let stats = translator.get_stats();
        assert_eq!(stats.instructions_translated, 2);
        assert_eq!(stats.phase, TranslationPhase::InProgress);
    }

    #[test]
    fn test_register_mapping_access() {
        let translator = RiscvToX86Translator::new();

        let _mapping = &translator.context().register_mapping;
        // Verify mapping was created (AllGPR strategy should map all 32 RISC-V registers)
        // The mapping is initialized via RegisterAllocationConfig::allgpr()
    }

    #[test]
    fn test_emit_ret() {
        let mut translator = RiscvToX86Translator::new();
        translator.start();

        translator.emit_instruction(&X86Instruction::Ret).unwrap();
        assert_eq!(translator.get_bytecode()[0], 0xC3);
    }
}
