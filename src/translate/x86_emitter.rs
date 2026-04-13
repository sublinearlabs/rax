//! x86-64 bytecode emitter
//!
//! This module generates x86-64 bytecode from high-level instructions.
//! It handles instruction encoding, REX prefixes, ModRM bytes, and label resolution.

use crate::translate::x86_insn::{Operand, X86Register};
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

    /// Finalize the emitter: apply all relocations and return the bytecode
    /// This must be called before using the bytecode for ELF generation
    pub fn finalize(mut self) -> Result<Vec<u8>, String> {
        self.apply_relocations()?;
        Ok(self.buffer)
    }

    /// Take ownership of the buffer and return it (consumes the emitter)
    pub fn into_buffer(self) -> Vec<u8> {
        self.buffer
    }

    /// Emit a REX prefix for 64-bit operations
    /// REX = 0x48 for basic 64-bit operations
    /// With register extensions: 0x4C (add R bit for dest), etc.
    pub fn emit_rex(&mut self, w: bool, r: bool, x: bool, b: bool) {
        let mut rex = 0x40u8;
        if w {
            rex |= 0x08;
        } // W bit - 64-bit operand
        if r {
            rex |= 0x04;
        } // R bit - extend ModRM.reg
        if x {
            rex |= 0x02;
        } // X bit - extend SIB.index
        if b {
            rex |= 0x01;
        } // B bit - extend ModRM.r/m or SIB.base
        self.emit_byte(rex);
    }

    /// Emit a ModRM byte
    /// mod (2 bits) | reg (3 bits) | r/m (3 bits)
    pub fn emit_modrm(&mut self, mode: u8, reg: u8, rm: u8) {
        let byte = ((mode & 0x3) << 6) | ((reg & 0x7) << 3) | (rm & 0x7);
        self.emit_byte(byte);
    }

    /// Emit RET instruction
    pub fn emit_ret(&mut self) {
        self.emit_byte(0xC3);
    }

    /// Emit MOV instruction: mov dst, src
    /// Encoding: 0x89 /r for reg→reg, 0xC7 /0 for imm→reg, 0x8B for reg←mem, etc.
    pub fn emit_mov(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            // mov reg64, imm64 - needs 0x48 REX + 0xB8-0xBF opcode
            (Operand::Immediate(imm), Operand::Register(dst_reg)) => {
                let reg_code = dst_reg.code();

                // REX.W = 1 for 64-bit, REX.B for register extension
                let needs_rex_b = *dst_reg as u8 > 7;
                self.emit_rex(true, false, false, needs_rex_b);

                // MOVABS: 0xB8 + reg_code (with REX.B adjustment)
                self.emit_byte(0xB8 + (reg_code & 0x7));

                // Emit 64-bit immediate
                self.emit_i64(*imm);
                Ok(())
            }

            // mov reg64, reg64 - REX.W + 0x89 + ModRM
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                let src_code = src_reg.code();
                let dst_code = dst_reg.code();

                let src_ext = src_code >= 8;
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, src_ext, false, dst_ext);
                self.emit_byte(0x89); // MOV r64, r64
                self.emit_modrm(0x3, src_code & 0x7, dst_code & 0x7);

                Ok(())
            }

            // mov r64, [mem] - REX.W + 0x8B + ModRM
            (Operand::Memory { base, offset }, Operand::Register(dst_reg)) => {
                let base_code = base.code();
                let dst_code = dst_reg.code();

                let base_ext = base_code >= 8;
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, dst_ext, false, base_ext);
                self.emit_byte(0x8B); // MOV r64, m64

                // Handle displacement
                if *offset == 0 {
                    self.emit_modrm(0x0, dst_code & 0x7, base_code & 0x7);
                } else if *offset >= -128 && *offset <= 127 {
                    self.emit_modrm(0x1, dst_code & 0x7, base_code & 0x7);
                    self.emit_byte(*offset as u8);
                } else {
                    self.emit_modrm(0x2, dst_code & 0x7, base_code & 0x7);
                    self.emit_i32(*offset);
                }

                Ok(())
            }

            // mov [mem], r64 - REX.W + 0x89 + ModRM
            (Operand::Register(src_reg), Operand::Memory { base, offset }) => {
                let src_code = src_reg.code();
                let base_code = base.code();

                let src_ext = src_code >= 8;
                let base_ext = base_code >= 8;

                self.emit_rex(true, src_ext, false, base_ext);
                self.emit_byte(0x89); // MOV m64, r64

                // Handle displacement
                if *offset == 0 {
                    self.emit_modrm(0x0, src_code & 0x7, base_code & 0x7);
                } else if *offset >= -128 && *offset <= 127 {
                    self.emit_modrm(0x1, src_code & 0x7, base_code & 0x7);
                    self.emit_byte(*offset as u8);
                } else {
                    self.emit_modrm(0x2, src_code & 0x7, base_code & 0x7);
                    self.emit_i32(*offset);
                }

                Ok(())
            }

            _ => Err(format!("Invalid MOV operands: {} {}", src, dst)),
        }
    }

    /// Emit JMP instruction (placeholder for label resolution)
    pub fn emit_jmp(&mut self, target: &str) -> Result<(), String> {
        self.relocations
            .push((self.offset() + 1, target.to_string()));

        // JMP rel32 - 0xE9 followed by 32-bit offset
        self.emit_byte(0xE9);
        self.emit_i32(0); // Placeholder, will be patched

        Ok(())
    }

    /// Emit ADD instruction
    pub fn emit_add(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            // add r64, r64 - REX.W + 0x01 + ModRM
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                let src_code = src_reg.code();
                let dst_code = dst_reg.code();

                let src_ext = src_code >= 8;
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, src_ext, false, dst_ext);
                self.emit_byte(0x01); // ADD r64, r64
                self.emit_modrm(0x3, src_code & 0x7, dst_code & 0x7);

                Ok(())
            }

            // add r64, imm64 - REX.W + 0x81 + ModRM for imm32, or MOVABS + ADD for full imm64
            (Operand::Immediate(imm), Operand::Register(dst_reg)) => {
                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                // Check if immediate fits in 32-bit sign-extended form
                if *imm >= i32::MIN as i64 && *imm <= i32::MAX as i64 {
                    // Use direct ADD r64, imm32
                    self.emit_rex(true, false, false, dst_ext);
                    self.emit_byte(0x81);
                    self.emit_modrm(0x3, 0, dst_code & 0x7); // 0 for ADD
                    self.emit_i32(*imm as i32);
                } else {
                    // For full 64-bit immediate: load into RAX, then ADD dst, RAX
                    // Save RAX first if needed, then restore
                    // For simplicity: MOVABS RAX, imm64 then ADD dst, RAX
                    self.emit_mov(
                        &Operand::Immediate(*imm),
                        &Operand::Register(X86Register::RAX),
                    )?;
                    self.emit_add(&Operand::Register(X86Register::RAX), dst)?;
                }

                Ok(())
            }

            _ => Err(format!("Invalid ADD operands: {} {}", src, dst)),
        }
    }

    /// Emit SUB instruction
    pub fn emit_sub(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            // sub r64, r64 - REX.W + 0x29 + ModRM
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                let src_code = src_reg.code();
                let dst_code = dst_reg.code();

                let src_ext = src_code >= 8;
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, src_ext, false, dst_ext);
                self.emit_byte(0x29); // SUB r64, r64
                self.emit_modrm(0x3, src_code & 0x7, dst_code & 0x7);

                Ok(())
            }

            // sub r64, imm64 - REX.W + 0x81 + ModRM for imm32, or MOVABS + SUB for full imm64
            (Operand::Immediate(imm), Operand::Register(dst_reg)) => {
                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                // Check if immediate fits in 32-bit sign-extended form
                if *imm >= i32::MIN as i64 && *imm <= i32::MAX as i64 {
                    // Use direct SUB r64, imm32
                    self.emit_rex(true, false, false, dst_ext);
                    self.emit_byte(0x81);
                    self.emit_modrm(0x3, 5, dst_code & 0x7); // 5 for SUB
                    self.emit_i32(*imm as i32);
                } else {
                    // For full 64-bit immediate: load into RAX, then SUB dst, RAX
                    self.emit_mov(
                        &Operand::Immediate(*imm),
                        &Operand::Register(X86Register::RAX),
                    )?;
                    self.emit_sub(&Operand::Register(X86Register::RAX), dst)?;
                }

                Ok(())
            }

            _ => Err(format!("Invalid SUB operands: {} {}", src, dst)),
        }
    }

    /// Emit AND instruction
    pub fn emit_and(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            // and r64, r64 - REX.W + 0x21 + ModRM
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                let src_code = src_reg.code();
                let dst_code = dst_reg.code();

                let src_ext = src_code >= 8;
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, src_ext, false, dst_ext);
                self.emit_byte(0x21); // AND r64, r64
                self.emit_modrm(0x3, src_code & 0x7, dst_code & 0x7);

                Ok(())
            }

            // and r64, imm64 - REX.W + 0x81 + ModRM for imm32, or MOVABS + AND for full imm64
            (Operand::Immediate(imm), Operand::Register(dst_reg)) => {
                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                // Check if immediate fits in 32-bit sign-extended form
                if *imm >= i32::MIN as i64 && *imm <= i32::MAX as i64 {
                    // Use direct AND r64, imm32
                    self.emit_rex(true, false, false, dst_ext);
                    self.emit_byte(0x81);
                    self.emit_modrm(0x3, 4, dst_code & 0x7); // 4 for AND
                    self.emit_i32(*imm as i32);
                } else {
                    // For full 64-bit immediate: load into RAX, then AND dst, RAX
                    self.emit_mov(
                        &Operand::Immediate(*imm),
                        &Operand::Register(X86Register::RAX),
                    )?;
                    self.emit_and(&Operand::Register(X86Register::RAX), dst)?;
                }

                Ok(())
            }

            _ => Err(format!("Invalid AND operands: {} {}", src, dst)),
        }
    }

    /// Emit OR instruction
    pub fn emit_or(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            // or r64, r64 - REX.W + 0x09 + ModRM
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                let src_code = src_reg.code();
                let dst_code = dst_reg.code();

                let src_ext = src_code >= 8;
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, src_ext, false, dst_ext);
                self.emit_byte(0x09); // OR r64, r64
                self.emit_modrm(0x3, src_code & 0x7, dst_code & 0x7);

                Ok(())
            }

            // or r64, imm64 - REX.W + 0x81 + ModRM for imm32, or MOVABS + OR for full imm64
            (Operand::Immediate(imm), Operand::Register(dst_reg)) => {
                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                // Check if immediate fits in 32-bit sign-extended form
                if *imm >= i32::MIN as i64 && *imm <= i32::MAX as i64 {
                    // Use direct OR r64, imm32
                    self.emit_rex(true, false, false, dst_ext);
                    self.emit_byte(0x81);
                    self.emit_modrm(0x3, 1, dst_code & 0x7); // 1 for OR
                    self.emit_i32(*imm as i32);
                } else {
                    // For full 64-bit immediate: load into RAX, then OR dst, RAX
                    self.emit_mov(
                        &Operand::Immediate(*imm),
                        &Operand::Register(X86Register::RAX),
                    )?;
                    self.emit_or(&Operand::Register(X86Register::RAX), dst)?;
                }

                Ok(())
            }

            _ => Err(format!("Invalid OR operands: {} {}", src, dst)),
        }
    }

    /// Emit XOR instruction
    pub fn emit_xor(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            // xor r64, r64 - REX.W + 0x31 + ModRM
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                let src_code = src_reg.code();
                let dst_code = dst_reg.code();

                let src_ext = src_code >= 8;
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, src_ext, false, dst_ext);
                self.emit_byte(0x31); // XOR r64, r64
                self.emit_modrm(0x3, src_code & 0x7, dst_code & 0x7);

                Ok(())
            }

            // xor r64, imm64 - REX.W + 0x81 + ModRM for imm32, or MOVABS + XOR for full imm64
            (Operand::Immediate(imm), Operand::Register(dst_reg)) => {
                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                // Check if immediate fits in 32-bit sign-extended form
                if *imm >= i32::MIN as i64 && *imm <= i32::MAX as i64 {
                    // Use direct XOR r64, imm32
                    self.emit_rex(true, false, false, dst_ext);
                    self.emit_byte(0x81);
                    self.emit_modrm(0x3, 6, dst_code & 0x7); // 6 for XOR
                    self.emit_i32(*imm as i32);
                } else {
                    // For full 64-bit immediate: load into RAX, then XOR dst, RAX
                    self.emit_mov(
                        &Operand::Immediate(*imm),
                        &Operand::Register(X86Register::RAX),
                    )?;
                    self.emit_xor(&Operand::Register(X86Register::RAX), dst)?;
                }

                Ok(())
            }

            _ => Err(format!("Invalid XOR operands: {} {}", src, dst)),
        }
    }

    /// Emit CMP instruction (Compare - performs subtraction and sets flags, discards result)
    pub fn emit_cmp(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            // cmp r64, r64 - REX.W + 0x39 + ModRM
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                let src_code = src_reg.code();
                let dst_code = dst_reg.code();

                let src_ext = src_code >= 8;
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, src_ext, false, dst_ext);
                self.emit_byte(0x39); // CMP r64, r64
                self.emit_modrm(0x3, src_code & 0x7, dst_code & 0x7);

                Ok(())
            }

            // cmp r64, imm64 - REX.W + 0x81 + ModRM for imm32, or MOVABS + CMP for full imm64
            (Operand::Immediate(imm), Operand::Register(dst_reg)) => {
                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                // Check if immediate fits in 32-bit sign-extended form
                if *imm >= i32::MIN as i64 && *imm <= i32::MAX as i64 {
                    // Use direct CMP r64, imm32
                    self.emit_rex(true, false, false, dst_ext);
                    self.emit_byte(0x81);
                    self.emit_modrm(0x3, 7, dst_code & 0x7); // 7 for CMP
                    self.emit_i32(*imm as i32);
                } else {
                    // For full 64-bit immediate: load into RAX, then CMP dst, RAX
                    self.emit_mov(
                        &Operand::Immediate(*imm),
                        &Operand::Register(X86Register::RAX),
                    )?;
                    self.emit_cmp(&Operand::Register(X86Register::RAX), dst)?;
                }

                Ok(())
            }

            _ => Err(format!("Invalid CMP operands: {} {}", src, dst)),
        }
    }

    /// Emit TEST instruction (Bitwise AND with flags set, result discarded)
    /// TEST affects only ZF, SF, PF; CF and OF are cleared
    pub fn emit_test(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            // test r64, r64 - REX.W + 0x85 + ModRM
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                let src_code = src_reg.code();
                let dst_code = dst_reg.code();

                let src_ext = src_code >= 8;
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, src_ext, false, dst_ext);
                self.emit_byte(0x85); // TEST r64, r64
                self.emit_modrm(0x3, src_code & 0x7, dst_code & 0x7);

                Ok(())
            }

            // test r64, imm64 - REX.W + 0xF7 + ModRM for imm32, or MOVABS + TEST for full imm64
            (Operand::Immediate(imm), Operand::Register(dst_reg)) => {
                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                // Check if immediate fits in 32-bit sign-extended form
                if *imm >= i32::MIN as i64 && *imm <= i32::MAX as i64 {
                    // Use direct TEST r64, imm32
                    self.emit_rex(true, false, false, dst_ext);
                    self.emit_byte(0xF7);
                    self.emit_modrm(0x3, 0, dst_code & 0x7); // 0 for TEST
                    self.emit_i32(*imm as i32);
                } else {
                    // For full 64-bit immediate: load into RAX, then TEST dst, RAX
                    self.emit_mov(
                        &Operand::Immediate(*imm),
                        &Operand::Register(X86Register::RAX),
                    )?;
                    self.emit_test(&Operand::Register(X86Register::RAX), dst)?;
                }

                Ok(())
            }

            _ => Err(format!("Invalid TEST operands: {} {}", src, dst)),
        }
    }

    /// Emit JE instruction (Jump if Equal / Jump if Zero)
    /// Uses ZF flag from previous comparison or test
    pub fn emit_je(&mut self, target: &str) -> Result<(), String> {
        self.relocations
            .push((self.offset() + 2, target.to_string()));

        // JE rel32 - 0x0F 0x84 followed by 32-bit offset
        self.emit_byte(0x0F);
        self.emit_byte(0x84);
        self.emit_i32(0); // Placeholder, will be patched

        Ok(())
    }

    /// Emit JNE instruction (Jump if Not Equal / Jump if Not Zero)
    /// Uses ZF flag from previous comparison or test
    pub fn emit_jne(&mut self, target: &str) -> Result<(), String> {
        self.relocations
            .push((self.offset() + 2, target.to_string()));

        // JNE rel32 - 0x0F 0x85 followed by 32-bit offset
        self.emit_byte(0x0F);
        self.emit_byte(0x85);
        self.emit_i32(0); // Placeholder, will be patched

        Ok(())
    }

    /// Emit JL instruction (Jump if Less - signed comparison)
    /// SF != OF (Sign Flag not equal to Overflow Flag)
    pub fn emit_jl(&mut self, target: &str) -> Result<(), String> {
        self.relocations
            .push((self.offset() + 2, target.to_string()));

        // JL rel32 - 0x0F 0x8C followed by 32-bit offset
        self.emit_byte(0x0F);
        self.emit_byte(0x8C);
        self.emit_i32(0); // Placeholder, will be patched

        Ok(())
    }

    /// Emit JLE instruction (Jump if Less or Equal - signed comparison)
    /// ZF=1 or SF != OF
    pub fn emit_jle(&mut self, target: &str) -> Result<(), String> {
        self.relocations
            .push((self.offset() + 2, target.to_string()));

        // JLE rel32 - 0x0F 0x8E followed by 32-bit offset
        self.emit_byte(0x0F);
        self.emit_byte(0x8E);
        self.emit_i32(0); // Placeholder, will be patched

        Ok(())
    }

    /// Emit JG instruction (Jump if Greater - signed comparison)
    /// ZF=0 and SF = OF
    pub fn emit_jg(&mut self, target: &str) -> Result<(), String> {
        self.relocations
            .push((self.offset() + 2, target.to_string()));

        // JG rel32 - 0x0F 0x8F followed by 32-bit offset
        self.emit_byte(0x0F);
        self.emit_byte(0x8F);
        self.emit_i32(0); // Placeholder, will be patched

        Ok(())
    }

    /// Emit JGE instruction (Jump if Greater or Equal - signed comparison)
    /// SF = OF
    pub fn emit_jge(&mut self, target: &str) -> Result<(), String> {
        self.relocations
            .push((self.offset() + 2, target.to_string()));

        // JGE rel32 - 0x0F 0x8D followed by 32-bit offset
        self.emit_byte(0x0F);
        self.emit_byte(0x8D);
        self.emit_i32(0); // Placeholder, will be patched

        Ok(())
    }

    /// Emit JB instruction (Jump if Below - unsigned comparison)
    /// CF = 1 (Carry Flag set)
    pub fn emit_jb(&mut self, target: &str) -> Result<(), String> {
        self.relocations
            .push((self.offset() + 2, target.to_string()));

        // JB rel32 - 0x0F 0x82 followed by 32-bit offset
        self.emit_byte(0x0F);
        self.emit_byte(0x82);
        self.emit_i32(0); // Placeholder, will be patched

        Ok(())
    }

    /// Emit JBE instruction (Jump if Below or Equal - unsigned comparison)
    /// CF = 1 or ZF = 1
    pub fn emit_jbe(&mut self, target: &str) -> Result<(), String> {
        self.relocations
            .push((self.offset() + 2, target.to_string()));

        // JBE rel32 - 0x0F 0x86 followed by 32-bit offset
        self.emit_byte(0x0F);
        self.emit_byte(0x86);
        self.emit_i32(0); // Placeholder, will be patched

        Ok(())
    }

    /// Emit JA instruction (Jump if Above - unsigned comparison)
    /// CF = 0 and ZF = 0
    pub fn emit_ja(&mut self, target: &str) -> Result<(), String> {
        self.relocations
            .push((self.offset() + 2, target.to_string()));

        // JA rel32 - 0x0F 0x87 followed by 32-bit offset
        self.emit_byte(0x0F);
        self.emit_byte(0x87);
        self.emit_i32(0); // Placeholder, will be patched

        Ok(())
    }

    /// Emit JAE instruction (Jump if Above or Equal - unsigned comparison)
    /// CF = 0
    pub fn emit_jae(&mut self, target: &str) -> Result<(), String> {
        self.relocations
            .push((self.offset() + 2, target.to_string()));

        // JAE rel32 - 0x0F 0x83 followed by 32-bit offset
        self.emit_byte(0x0F);
        self.emit_byte(0x83);
        self.emit_i32(0); // Placeholder, will be patched

        Ok(())
    }

    /// Emit JZ instruction (Jump if Zero) - alias for JE
    pub fn emit_jz(&mut self, target: &str) -> Result<(), String> {
        self.emit_je(target)
    }

    /// Emit JNZ instruction (Jump if Not Zero) - alias for JNE
    pub fn emit_jnz(&mut self, target: &str) -> Result<(), String> {
        self.emit_jne(target)
    }

    /// Emit SHL instruction (Shift Left Logical)
    /// Can shift by CL register or immediate
    pub fn emit_shl(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            // shl r64, imm8 - REX.W + 0xC1 + ModRM + imm8
            (Operand::Immediate(imm), Operand::Register(dst_reg)) => {
                if *imm < 0 || *imm > 63 {
                    return Err(format!("SHL immediate must be 0-63, got {}", imm));
                }

                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, false, false, dst_ext);
                self.emit_byte(0xC1);
                self.emit_modrm(0x3, 4, dst_code & 0x7); // 4 for SHL
                self.emit_byte(*imm as u8);

                Ok(())
            }

            // shl r64, cl - REX.W + 0xD3 + ModRM (CL register implicit)
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                if *src_reg != X86Register::RCX {
                    return Err(format!(
                        "SHL register form only supports CL (RCX), got {:?}",
                        src_reg
                    ));
                }

                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, false, false, dst_ext);
                self.emit_byte(0xD3);
                self.emit_modrm(0x3, 4, dst_code & 0x7); // 4 for SHL

                Ok(())
            }

            _ => Err(format!("Invalid SHL operands: {} {}", src, dst)),
        }
    }

    /// Emit SHR instruction (Shift Right Logical)
    pub fn emit_shr(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            // shr r64, imm8 - REX.W + 0xC1 + ModRM + imm8
            (Operand::Immediate(imm), Operand::Register(dst_reg)) => {
                if *imm < 0 || *imm > 63 {
                    return Err(format!("SHR immediate must be 0-63, got {}", imm));
                }

                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, false, false, dst_ext);
                self.emit_byte(0xC1);
                self.emit_modrm(0x3, 5, dst_code & 0x7); // 5 for SHR
                self.emit_byte(*imm as u8);

                Ok(())
            }

            // shr r64, cl - REX.W + 0xD3 + ModRM
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                if *src_reg != X86Register::RCX {
                    return Err(format!(
                        "SHR register form only supports CL (RCX), got {:?}",
                        src_reg
                    ));
                }

                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, false, false, dst_ext);
                self.emit_byte(0xD3);
                self.emit_modrm(0x3, 5, dst_code & 0x7); // 5 for SHR

                Ok(())
            }

            _ => Err(format!("Invalid SHR operands: {} {}", src, dst)),
        }
    }

    /// Emit SAR instruction (Shift Right Arithmetic - sign-extending)
    pub fn emit_sar(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            // sar r64, imm8 - REX.W + 0xC1 + ModRM + imm8
            (Operand::Immediate(imm), Operand::Register(dst_reg)) => {
                if *imm < 0 || *imm > 63 {
                    return Err(format!("SAR immediate must be 0-63, got {}", imm));
                }

                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, false, false, dst_ext);
                self.emit_byte(0xC1);
                self.emit_modrm(0x3, 7, dst_code & 0x7); // 7 for SAR
                self.emit_byte(*imm as u8);

                Ok(())
            }

            // sar r64, cl - REX.W + 0xD3 + ModRM
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                if *src_reg != X86Register::RCX {
                    return Err(format!(
                        "SAR register form only supports CL (RCX), got {:?}",
                        src_reg
                    ));
                }

                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, false, false, dst_ext);
                self.emit_byte(0xD3);
                self.emit_modrm(0x3, 7, dst_code & 0x7); // 7 for SAR

                Ok(())
            }

            _ => Err(format!("Invalid SAR operands: {} {}", src, dst)),
        }
    }

    /// Emit IMUL instruction (Signed Multiply)
    /// Two-operand form: imul r64, r64
    pub fn emit_imul(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            // imul r64, r64 - REX.W + 0x0F 0xAF + ModRM
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                let src_code = src_reg.code();
                let dst_code = dst_reg.code();

                let src_ext = src_code >= 8;
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, dst_ext, false, src_ext);
                self.emit_byte(0x0F);
                self.emit_byte(0xAF);
                self.emit_modrm(0x3, dst_code & 0x7, src_code & 0x7);

                Ok(())
            }

            // imul r64, imm32 - REX.W + 0x69 + ModRM + imm32
            (Operand::Immediate(imm), Operand::Register(dst_reg)) => {
                if *imm < i32::MIN as i64 || *imm > i32::MAX as i64 {
                    return Err(format!("IMUL immediate must fit in i32, got {}", imm));
                }

                let dst_code = dst_reg.code();
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, false, false, dst_ext);
                self.emit_byte(0x69);
                self.emit_modrm(0x3, dst_code & 0x7, dst_code & 0x7); // dst is both source and destination
                self.emit_i32(*imm as i32);

                Ok(())
            }

            _ => Err(format!("Invalid IMUL operands: {} {}", src, dst)),
        }
    }

    /// Emit MUL instruction (Unsigned Multiply)
    /// One-operand form: mul r64 (multiplies RAX by r64, result in RDX:RAX)
    pub fn emit_mul(&mut self, src: &Operand) -> Result<(), String> {
        match src {
            // mul r64 - REX.W + 0xF7 + ModRM
            Operand::Register(src_reg) => {
                let src_code = src_reg.code();
                let src_ext = src_code >= 8;

                self.emit_rex(true, false, false, src_ext);
                self.emit_byte(0xF7);
                self.emit_modrm(0x3, 4, src_code & 0x7); // 4 for MUL

                Ok(())
            }

            _ => Err(format!("MUL only supports register operand, got {}", src)),
        }
    }

    /// Emit IDIV instruction (Signed Divide)
    /// One-operand form: idiv r64 (divides RDX:RAX by r64)
    /// Quotient in RAX, remainder in RDX
    pub fn emit_idiv(&mut self, src: &Operand) -> Result<(), String> {
        match src {
            // idiv r64 - REX.W + 0xF7 + ModRM
            Operand::Register(src_reg) => {
                let src_code = src_reg.code();
                let src_ext = src_code >= 8;

                self.emit_rex(true, false, false, src_ext);
                self.emit_byte(0xF7);
                self.emit_modrm(0x3, 7, src_code & 0x7); // 7 for IDIV

                Ok(())
            }

            _ => Err(format!("IDIV only supports register operand, got {}", src)),
        }
    }

    /// Emit DIV instruction (Unsigned Divide)
    /// One-operand form: div r64 (divides RDX:RAX by r64)
    /// Quotient in RAX, remainder in RDX
    pub fn emit_div(&mut self, src: &Operand) -> Result<(), String> {
        match src {
            // div r64 - REX.W + 0xF7 + ModRM
            Operand::Register(src_reg) => {
                let src_code = src_reg.code();
                let src_ext = src_code >= 8;

                self.emit_rex(true, false, false, src_ext);
                self.emit_byte(0xF7);
                self.emit_modrm(0x3, 6, src_code & 0x7); // 6 for DIV

                Ok(())
            }

            _ => Err(format!("DIV only supports register operand, got {}", src)),
        }
    }

    /// Emit CQO instruction (Convert Quadword to Octaword)
    /// Sign-extends RAX to RDX:RAX for signed division
    pub fn emit_cqo(&mut self) {
        self.emit_rex(true, false, false, false);
        self.emit_byte(0x99);
    }

    /// Emit CDQ instruction (Convert Doubleword to Quadword)
    /// Sign-extends EAX to EDX:EAX for signed 32-bit division
    pub fn emit_cdq(&mut self) {
        self.emit_byte(0x99); // No REX needed for 32-bit
    }

    /// Emit XOR r64, r64 to zero a register
    pub fn emit_xor_self(&mut self, reg: &X86Register) -> Result<(), String> {
        self.emit_xor(&Operand::Register(*reg), &Operand::Register(*reg))
    }

    /// Emit MOVSX instruction (Move with Sign Extension)
    /// movsx r64, r32 - sign-extends 32-bit to 64-bit
    pub fn emit_movsx_32_to_64(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                let src_code = src_reg.code();
                let dst_code = dst_reg.code();

                let src_ext = src_code >= 8;
                let dst_ext = dst_code >= 8;

                self.emit_rex(true, dst_ext, false, src_ext);
                self.emit_byte(0x63); // MOVSXD r64, r32
                self.emit_modrm(0x3, dst_code & 0x7, src_code & 0x7);

                Ok(())
            }

            _ => Err(format!(
                "MOVSX only supports register operands, got {} {}",
                src, dst
            )),
        }
    }

    /// Emit MOVZX instruction (Move with Zero Extension)
    /// movzx r64, r32 - zero-extends 32-bit to 64-bit
    pub fn emit_movzx_32_to_64(&mut self, src: &Operand, dst: &Operand) -> Result<(), String> {
        match (src, dst) {
            (Operand::Register(src_reg), Operand::Register(dst_reg)) => {
                let src_code = src_reg.code();
                let dst_code = dst_reg.code();

                let src_ext = src_code >= 8;
                let dst_ext = dst_code >= 8;

                // movzx r64, r32 can be done with MOV r32, r32 (which zero-extends to 64-bit)
                // Or use REX.W = 0 and regular MOV, which zero-extends
                self.emit_rex(false, dst_ext, false, src_ext); // REX.W = 0 for zero-extend
                self.emit_byte(0x89);
                self.emit_modrm(0x3, src_code & 0x7, dst_code & 0x7);

                Ok(())
            }

            _ => Err(format!(
                "MOVZX only supports register operands, got {} {}",
                src, dst
            )),
        }
    }

    /// Emit memory load instructions (various sizes)
    /// MOV r64, [mem] - already handled by emit_mov()
    /// But we need explicit size-specific versions for type safety

    /// Load 64-bit from memory: MOV r64, [base + offset]
    pub fn emit_load64(
        &mut self,
        base: X86Register,
        offset: i32,
        dst: X86Register,
    ) -> Result<(), String> {
        let base_code = base.code();
        let dst_code = dst.code();

        let base_ext = base_code >= 8;
        let dst_ext = dst_code >= 8;

        self.emit_rex(true, dst_ext, false, base_ext);
        self.emit_byte(0x8B); // MOV r64, m64

        if offset == 0 {
            self.emit_modrm(0x0, dst_code & 0x7, base_code & 0x7);
        } else if offset >= -128 && offset <= 127 {
            self.emit_modrm(0x1, dst_code & 0x7, base_code & 0x7);
            self.emit_byte(offset as u8);
        } else {
            self.emit_modrm(0x2, dst_code & 0x7, base_code & 0x7);
            self.emit_i32(offset);
        }

        Ok(())
    }

    /// Load 32-bit sign-extended to 64-bit: MOVSXD r64, [base + offset]
    pub fn emit_load32_sext(
        &mut self,
        base: X86Register,
        offset: i32,
        dst: X86Register,
    ) -> Result<(), String> {
        let base_code = base.code();
        let dst_code = dst.code();

        let base_ext = base_code >= 8;
        let dst_ext = dst_code >= 8;

        self.emit_rex(true, dst_ext, false, base_ext);
        self.emit_byte(0x63); // MOVSXD r64, m32

        if offset == 0 {
            self.emit_modrm(0x0, dst_code & 0x7, base_code & 0x7);
        } else if offset >= -128 && offset <= 127 {
            self.emit_modrm(0x1, dst_code & 0x7, base_code & 0x7);
            self.emit_byte(offset as u8);
        } else {
            self.emit_modrm(0x2, dst_code & 0x7, base_code & 0x7);
            self.emit_i32(offset);
        }

        Ok(())
    }

    /// Load 32-bit zero-extended to 64-bit: MOV r32, [base + offset]
    /// (32-bit move automatically zero-extends to 64-bit)
    pub fn emit_load32_zext(
        &mut self,
        base: X86Register,
        offset: i32,
        dst: X86Register,
    ) -> Result<(), String> {
        let base_code = base.code();
        let dst_code = dst.code();

        let base_ext = base_code >= 8;
        let dst_ext = dst_code >= 8;

        self.emit_rex(false, dst_ext, false, base_ext); // REX.W = 0 for 32-bit (zero-extends)
        self.emit_byte(0x8B); // MOV r32, m32

        if offset == 0 {
            self.emit_modrm(0x0, dst_code & 0x7, base_code & 0x7);
        } else if offset >= -128 && offset <= 127 {
            self.emit_modrm(0x1, dst_code & 0x7, base_code & 0x7);
            self.emit_byte(offset as u8);
        } else {
            self.emit_modrm(0x2, dst_code & 0x7, base_code & 0x7);
            self.emit_i32(offset);
        }

        Ok(())
    }

    /// Load 16-bit sign-extended to 64-bit: MOVSX r64, [base + offset]
    pub fn emit_load16_sext(
        &mut self,
        base: X86Register,
        offset: i32,
        dst: X86Register,
    ) -> Result<(), String> {
        let base_code = base.code();
        let dst_code = dst.code();

        let base_ext = base_code >= 8;
        let dst_ext = dst_code >= 8;

        self.emit_rex(true, dst_ext, false, base_ext);
        self.emit_byte(0x0F);
        self.emit_byte(0xBF); // MOVSX r64, m16

        if offset == 0 {
            self.emit_modrm(0x0, dst_code & 0x7, base_code & 0x7);
        } else if offset >= -128 && offset <= 127 {
            self.emit_modrm(0x1, dst_code & 0x7, base_code & 0x7);
            self.emit_byte(offset as u8);
        } else {
            self.emit_modrm(0x2, dst_code & 0x7, base_code & 0x7);
            self.emit_i32(offset);
        }

        Ok(())
    }

    /// Load 16-bit zero-extended to 64-bit: MOVZX r64, [base + offset]
    pub fn emit_load16_zext(
        &mut self,
        base: X86Register,
        offset: i32,
        dst: X86Register,
    ) -> Result<(), String> {
        let base_code = base.code();
        let dst_code = dst.code();

        let base_ext = base_code >= 8;
        let dst_ext = dst_code >= 8;

        self.emit_rex(true, dst_ext, false, base_ext);
        self.emit_byte(0x0F);
        self.emit_byte(0xB7); // MOVZX r64, m16

        if offset == 0 {
            self.emit_modrm(0x0, dst_code & 0x7, base_code & 0x7);
        } else if offset >= -128 && offset <= 127 {
            self.emit_modrm(0x1, dst_code & 0x7, base_code & 0x7);
            self.emit_byte(offset as u8);
        } else {
            self.emit_modrm(0x2, dst_code & 0x7, base_code & 0x7);
            self.emit_i32(offset);
        }

        Ok(())
    }

    /// Load 8-bit sign-extended to 64-bit: MOVSX r64, [base + offset]
    pub fn emit_load8_sext(
        &mut self,
        base: X86Register,
        offset: i32,
        dst: X86Register,
    ) -> Result<(), String> {
        let base_code = base.code();
        let dst_code = dst.code();

        let base_ext = base_code >= 8;
        let dst_ext = dst_code >= 8;

        self.emit_rex(true, dst_ext, false, base_ext);
        self.emit_byte(0x0F);
        self.emit_byte(0xBE); // MOVSX r64, m8

        if offset == 0 {
            self.emit_modrm(0x0, dst_code & 0x7, base_code & 0x7);
        } else if offset >= -128 && offset <= 127 {
            self.emit_modrm(0x1, dst_code & 0x7, base_code & 0x7);
            self.emit_byte(offset as u8);
        } else {
            self.emit_modrm(0x2, dst_code & 0x7, base_code & 0x7);
            self.emit_i32(offset);
        }

        Ok(())
    }

    /// Load 8-bit zero-extended to 64-bit: MOVZX r64, [base + offset]
    pub fn emit_load8_zext(
        &mut self,
        base: X86Register,
        offset: i32,
        dst: X86Register,
    ) -> Result<(), String> {
        let base_code = base.code();
        let dst_code = dst.code();

        let base_ext = base_code >= 8;
        let dst_ext = dst_code >= 8;

        self.emit_rex(true, dst_ext, false, base_ext);
        self.emit_byte(0x0F);
        self.emit_byte(0xB6); // MOVZX r64, m8

        if offset == 0 {
            self.emit_modrm(0x0, dst_code & 0x7, base_code & 0x7);
        } else if offset >= -128 && offset <= 127 {
            self.emit_modrm(0x1, dst_code & 0x7, base_code & 0x7);
            self.emit_byte(offset as u8);
        } else {
            self.emit_modrm(0x2, dst_code & 0x7, base_code & 0x7);
            self.emit_i32(offset);
        }

        Ok(())
    }

    /// Emit memory store instructions
    /// Store 64-bit to memory: MOV [base + offset], r64
    pub fn emit_store64(
        &mut self,
        src: X86Register,
        base: X86Register,
        offset: i32,
    ) -> Result<(), String> {
        let src_code = src.code();
        let base_code = base.code();

        let src_ext = src_code >= 8;
        let base_ext = base_code >= 8;

        self.emit_rex(true, src_ext, false, base_ext);
        self.emit_byte(0x89); // MOV m64, r64

        if offset == 0 {
            self.emit_modrm(0x0, src_code & 0x7, base_code & 0x7);
        } else if offset >= -128 && offset <= 127 {
            self.emit_modrm(0x1, src_code & 0x7, base_code & 0x7);
            self.emit_byte(offset as u8);
        } else {
            self.emit_modrm(0x2, src_code & 0x7, base_code & 0x7);
            self.emit_i32(offset);
        }

        Ok(())
    }

    /// Store 32-bit to memory: MOV [base + offset], r32
    pub fn emit_store32(
        &mut self,
        src: X86Register,
        base: X86Register,
        offset: i32,
    ) -> Result<(), String> {
        let src_code = src.code();
        let base_code = base.code();

        let src_ext = src_code >= 8;
        let base_ext = base_code >= 8;

        self.emit_rex(false, src_ext, false, base_ext); // REX.W = 0 for 32-bit
        self.emit_byte(0x89); // MOV m32, r32

        if offset == 0 {
            self.emit_modrm(0x0, src_code & 0x7, base_code & 0x7);
        } else if offset >= -128 && offset <= 127 {
            self.emit_modrm(0x1, src_code & 0x7, base_code & 0x7);
            self.emit_byte(offset as u8);
        } else {
            self.emit_modrm(0x2, src_code & 0x7, base_code & 0x7);
            self.emit_i32(offset);
        }

        Ok(())
    }

    /// Store 16-bit to memory: MOV [base + offset], r16
    pub fn emit_store16(
        &mut self,
        src: X86Register,
        base: X86Register,
        offset: i32,
    ) -> Result<(), String> {
        let src_code = src.code();
        let base_code = base.code();

        let src_ext = src_code >= 8;
        let base_ext = base_code >= 8;

        // Need 0x66 prefix for 16-bit operand size
        self.emit_byte(0x66);
        self.emit_rex(false, src_ext, false, base_ext); // REX without W bit
        self.emit_byte(0x89); // MOV m16, r16

        if offset == 0 {
            self.emit_modrm(0x0, src_code & 0x7, base_code & 0x7);
        } else if offset >= -128 && offset <= 127 {
            self.emit_modrm(0x1, src_code & 0x7, base_code & 0x7);
            self.emit_byte(offset as u8);
        } else {
            self.emit_modrm(0x2, src_code & 0x7, base_code & 0x7);
            self.emit_i32(offset);
        }

        Ok(())
    }

    /// Store 8-bit to memory: MOV [base + offset], r8
    pub fn emit_store8(
        &mut self,
        src: X86Register,
        base: X86Register,
        offset: i32,
    ) -> Result<(), String> {
        let src_code = src.code();
        let base_code = base.code();

        let src_ext = src_code >= 8;
        let base_ext = base_code >= 8;

        if src_ext || base_ext {
            self.emit_rex(false, src_ext, false, base_ext);
        }
        self.emit_byte(0x88); // MOV m8, r8

        if offset == 0 {
            self.emit_modrm(0x0, src_code & 0x7, base_code & 0x7);
        } else if offset >= -128 && offset <= 127 {
            self.emit_modrm(0x1, src_code & 0x7, base_code & 0x7);
            self.emit_byte(offset as u8);
        } else {
            self.emit_modrm(0x2, src_code & 0x7, base_code & 0x7);
            self.emit_i32(offset);
        }

        Ok(())
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
