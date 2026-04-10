/// RISC-V instruction to x86-64 translation logic
/// 
/// This module contains the main translation dispatch for converting RISC-V instructions
/// to x86-64 bytecode. Each RISC-V instruction generates 1-4 x86-64 instructions.

use crate::decode::Instruction as RiscvInstruction;
use crate::translate::x86_insn::{X86Instruction, Operand, X86Register};
use crate::translate::translator::RiscvToX86Translator;

/// Translate a single RISC-V instruction to x86-64
/// 
/// This function pattern-matches on the RISC-V instruction type and emits
/// the appropriate x86-64 instruction sequence via the translator.
/// 
/// Returns an error if the instruction is not yet supported.
pub(crate) fn translate_instruction(
    translator: &mut RiscvToX86Translator,
    riscv_insn: &RiscvInstruction,
) -> Result<(), String> {
    use crate::decode::Instruction::*;

    match riscv_insn {
        // === Basic ALU Operations (R-type) ===
        // ADD rd, rs1, rs2 → X[rd] = (X[rs1] + X[rs2]) mod 2^64
        Add(r) => {
            let rd_reg = Operand::Register(X86Register::RAX); // TODO: map from register_mapping
            let rs1_reg = Operand::Register(X86Register::RBX);
            let rs2_reg = Operand::Register(X86Register::RCX);
            
            // Don't emit mov if rd == rs1
            if r.rd != r.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::Add { src: rs2_reg, dst: rd_reg })?;
        }
        
        // SUB rd, rs1, rs2 → X[rd] = (X[rs1] - X[rs2]) mod 2^64
        Sub(r) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let rs2_reg = Operand::Register(X86Register::RCX);
            
            if r.rd != r.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::Sub { src: rs2_reg, dst: rd_reg })?;
        }
        
        // AND rd, rs1, rs2 → X[rd] = X[rs1] & X[rs2]
        And(r) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let rs2_reg = Operand::Register(X86Register::RCX);
            
            if r.rd != r.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::And { src: rs2_reg, dst: rd_reg })?;
        }
        
        // OR rd, rs1, rs2 → X[rd] = X[rs1] | X[rs2]
        Or(r) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let rs2_reg = Operand::Register(X86Register::RCX);
            
            if r.rd != r.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::Or { src: rs2_reg, dst: rd_reg })?;
        }
        
        // XOR rd, rs1, rs2 → X[rd] = X[rs1] ^ X[rs2]
        Xor(r) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let rs2_reg = Operand::Register(X86Register::RCX);
            
            if r.rd != r.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::Xor { src: rs2_reg, dst: rd_reg })?;
        }
        
        // === ALU Immediate Operations (I-type) ===
        // ADDI rd, rs1, imm → X[rd] = (X[rs1] + sext(imm[11:0])) mod 2^64
        Addi(i) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let imm_op = Operand::Immediate(i.imm as i64);
            
            if i.rd != i.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::Add { src: imm_op, dst: rd_reg })?;
        }
        
        // ORI rd, rs1, imm → X[rd] = X[rs1] | sext(imm[11:0])
        Ori(i) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let imm_op = Operand::Immediate(i.imm as i64);
            
            if i.rd != i.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::Or { src: imm_op, dst: rd_reg })?;
        }
        
        // ANDI rd, rs1, imm → X[rd] = X[rs1] & sext(imm[11:0])
        Andi(i) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let imm_op = Operand::Immediate(i.imm as i64);
            
            if i.rd != i.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::And { src: imm_op, dst: rd_reg })?;
        }
        
        // XORI rd, rs1, imm → X[rd] = X[rs1] ^ sext(imm[11:0])
        Xori(i) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let imm_op = Operand::Immediate(i.imm as i64);
            
            if i.rd != i.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::Xor { src: imm_op, dst: rd_reg })?;
        }
        
        // === Shift Operations ===
        // SLLI rd, rs1, shamt → X[rd] = (X[rs1] << shamt) mod 2^64
        Slli(sh) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let shamt_op = Operand::Immediate(sh.shamt as i64);
            
            if sh.rd != sh.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::Shl { src: shamt_op, dst: rd_reg })?;
        }
        
        // SRLI rd, rs1, shamt → X[rd] = X[rs1] >> shamt (unsigned)
        Srli(sh) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let shamt_op = Operand::Immediate(sh.shamt as i64);
            
            if sh.rd != sh.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::Shr { src: shamt_op, dst: rd_reg })?;
        }
        
        // SRAI rd, rs1, shamt → X[rd] = X[rs1] >>> shamt (signed, arithmetic)
        Srai(sh) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let shamt_op = Operand::Immediate(sh.shamt as i64);
            
            if sh.rd != sh.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::Sar { src: shamt_op, dst: rd_reg })?;
        }
        
        // SLL rd, rs1, rs2 → X[rd] = (X[rs1] << (X[rs2] & 0x3F)) mod 2^64 (variable shift)
        Sll(r) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let rs2_reg = Operand::Register(X86Register::RCX);
            
            // Variable shifts in x86 require RCX for shift amount
            // mov RCX, rs2; mov rax, rs1; shl rax, cl
            if r.rd != r.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            // TODO: Emit proper RCX load before shift
            translator.emit_instruction(&X86Instruction::Shl { src: rs2_reg, dst: rd_reg })?;
        }
        
        // SRL rd, rs1, rs2 → X[rd] = X[rs1] >> (X[rs2] & 0x3F) (unsigned variable shift)
        Srl(r) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let rs2_reg = Operand::Register(X86Register::RCX);
            
            if r.rd != r.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::Shr { src: rs2_reg, dst: rd_reg })?;
        }
        
        // SRA rd, rs1, rs2 → X[rd] = X[rs1] >>> (X[rs2] & 0x3F) (signed variable shift)
        Sra(r) => {
            let rd_reg = Operand::Register(X86Register::RAX);
            let rs1_reg = Operand::Register(X86Register::RBX);
            let rs2_reg = Operand::Register(X86Register::RCX);
            
            if r.rd != r.rs1 {
                translator.emit_instruction(&X86Instruction::Mov { src: rs1_reg.clone(), dst: rd_reg.clone() })?;
            }
            translator.emit_instruction(&X86Instruction::Sar { src: rs2_reg, dst: rd_reg })?;
        }
        
        // === Comparison Operations ===
        // SLT rd, rs1, rs2 → X[rd] = (X[rs1] <_s X[rs2]) ? 1 : 0 (signed less than)
        Slt(_r) => {
            let rs1_reg = Operand::Register(X86Register::RBX);
            let rs2_reg = Operand::Register(X86Register::RCX);
            
            // cmp rs1, rs2; setl rd
            translator.emit_instruction(&X86Instruction::Cmp { src: rs2_reg, dst: rs1_reg })?;
            let rd_reg = Operand::Register(X86Register::RAX);
            translator.emit_instruction(&X86Instruction::Setl { dst: rd_reg })?;
        }
        
        // SLTU rd, rs1, rs2 → X[rd] = (X[rs1] <_u X[rs2]) ? 1 : 0 (unsigned less than)
        Sltu(_r) => {
            // For unsigned less than, we need different logic
            return Err("SLTU not yet fully implemented".to_string());
        }
        
        // SLTI rd, rs1, imm → X[rd] = (X[rs1] <_s sext(imm)) ? 1 : 0
        Slti(i) => {
            let rs1_reg = Operand::Register(X86Register::RBX);
            let imm_op = Operand::Immediate(i.imm as i64);
            
            // cmp rs1, imm; setl rd
            translator.emit_instruction(&X86Instruction::Cmp { src: imm_op, dst: rs1_reg })?;
            let rd_reg = Operand::Register(X86Register::RAX);
            translator.emit_instruction(&X86Instruction::Setl { dst: rd_reg })?;
        }
        
        // SLTIU rd, rs1, imm → X[rd] = (X[rs1] <_u sext(imm)) ? 1 : 0
        Sltiu(_i) => {
            // For unsigned less than immediate, similar complexity
            return Err("SLTIU not yet fully implemented".to_string());
        }
        
        _ => {
            return Err(format!("Instruction not yet translated: {:?}", riscv_insn));
        }
    }
    
    Ok(())
}

