//! x86-64 bytecode emitter
//!
//! This module generates x86-64 bytecode from high-level instructions.
//! It handles instruction encoding, REX prefixes, ModRM bytes, and label resolution.

use crate::translate::x86_insn::{Operand, X86Instruction, X86Register};
use std::collections::HashMap;

/// Emits x86-64 bytecode from instructions
#[derive(Debug, Clone)]
pub struct X86Emitter {
    /// Output bytecode buffer
    buffer: Vec<u8>,

    /// Label positions (label name → offset in buffer)
    labels: HashMap<String, usize>,

    /// Pending relocations (offset in buffer → label name)
    relocations: Vec<(usize, String)>,
}

impl X86Emitter {
    /// Create a new emitter with empty buffer
    pub fn new() -> Self {
        X86Emitter {
            buffer: Vec::new(),
            labels: HashMap::new(),
            relocations: Vec::new(),
        }
    }

    /// Get the current buffer size (current offset)
    pub fn offset(&self) -> usize {
        self.buffer.len()
    }

    /// Emit a single byte
    pub fn emit_byte(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    /// Emit multiple bytes
    pub fn emit_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// Emit a 32-bit immediate (little-endian)
    pub fn emit_i32(&mut self, value: i32) {
        self.emit_bytes(&value.to_le_bytes());
    }

    /// Emit a 64-bit immediate (little-endian)
    pub fn emit_i64(&mut self, value: i64) {
        self.emit_bytes(&value.to_le_bytes());
    }

    /// Record a label at the current offset
    pub fn emit_label(&mut self, name: String) {
        self.labels.insert(name, self.offset());
    }

    /// Get the complete bytecode buffer
    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Get mutable buffer for final relocation patching
    pub fn get_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    /// Apply relocations - patch label references with actual offsets
    pub fn apply_relocations(&mut self) -> Result<(), String> {
        for (offset, label) in &self.relocations {
            let target_offset = self
                .labels
                .get(label)
                .ok_or_else(|| format!("Undefined label: {}", label))?;

            // Calculate relative offset for JMP (rel32 is relative to next instruction)
            let current_offset = offset + 4; // +4 because rel32 is 4 bytes
            let relative_offset = (*target_offset as i32) - (current_offset as i32);

            // Patch the 4-byte offset at the relocation point
            let bytes = relative_offset.to_le_bytes();
            self.buffer[*offset..*offset + 4].copy_from_slice(&bytes);
        }
        Ok(())
    }

    /// Get label positions for debugging
    pub fn get_labels(&self) -> &HashMap<String, usize> {
        &self.labels
    }

    /// Record a relocation for label patching
    pub fn record_relocation(&mut self, offset: usize, label: String) {
        self.relocations.push((offset, label));
    }
}

impl Default for X86Emitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emitter_creation() {
        let emitter = X86Emitter::new();
        assert_eq!(emitter.offset(), 0);
        assert!(emitter.get_buffer().is_empty());
    }

    #[test]
    fn test_emit_byte() {
        let mut emitter = X86Emitter::new();
        emitter.emit_byte(0x90); // NOP
        assert_eq!(emitter.offset(), 1);
        assert_eq!(emitter.get_buffer()[0], 0x90);
    }

    #[test]
    fn test_emit_mov_imm_to_reg() {
        let mut emitter = X86Emitter::new();
        let src = Operand::Immediate(42);
        let dst = Operand::Register(X86Register::RAX);

        emitter.emit_mov(&src, &dst).unwrap();
        assert!(emitter.offset() > 0);
    }

    #[test]
    fn test_emit_mov_reg_to_reg() {
        let mut emitter = X86Emitter::new();
        let src = Operand::Register(X86Register::RAX);
        let dst = Operand::Register(X86Register::RBX);

        emitter.emit_mov(&src, &dst).unwrap();
        assert!(emitter.offset() > 0);
    }

    #[test]
    fn test_emit_add_reg_to_reg() {
        let mut emitter = X86Emitter::new();
        let src = Operand::Register(X86Register::RAX);
        let dst = Operand::Register(X86Register::RBX);

        emitter.emit_add(&src, &dst).unwrap();
        assert!(emitter.offset() > 0);
    }

    #[test]
    fn test_emit_label() {
        let mut emitter = X86Emitter::new();
        emitter.emit_byte(0x90);
        emitter.emit_label("test_label".to_string());

        assert_eq!(emitter.get_labels().get("test_label"), Some(&1));
    }

    #[test]
    fn test_emit_jmp() {
        let mut emitter = X86Emitter::new();
        emitter.emit_jmp("loop_start").unwrap();

        assert_eq!(emitter.offset(), 5); // 0xE9 + 4 bytes offset
    }

    #[test]
    fn test_emit_ret() {
        let mut emitter = X86Emitter::new();
        emitter.emit_ret();

        assert_eq!(emitter.offset(), 1);
        assert_eq!(emitter.get_buffer()[0], 0xC3);
    }

    #[test]
    fn test_rex_encoding() {
        let mut emitter = X86Emitter::new();
        emitter.emit_rex(true, false, false, false);

        assert_eq!(emitter.get_buffer()[0], 0x48); // REX.W = 1
    }

    #[test]
    fn test_modrm_encoding() {
        let mut emitter = X86Emitter::new();
        emitter.emit_modrm(0x3, 0, 0); // mod=11, reg=000, r/m=000

        assert_eq!(emitter.get_buffer()[0], 0xC0);
    }
}
