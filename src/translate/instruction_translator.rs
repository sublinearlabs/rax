use crate::aot::register_mapping::RegisterLocation;
/// RISC-V instruction to x86-64 translation logic
///
/// This module contains the main translation dispatch for converting RISC-V instructions
/// to x86-64 bytecode. Each RISC-V instruction generates 1-4 x86-64 instructions.
use crate::decode::Instruction as RiscvInstruction;
use crate::translate::register_mapper::RegisterMapper;
use crate::translate::translator::RiscvToX86Translator;
use crate::translate::x86_insn::{Operand, X86Instruction};

/// Emit a binary operation with RegisterLocation operands
/// Handles both GPR and memory locations transparently
fn emit_bin_op<M: RegisterMapper>(
    translator: &mut RiscvToX86Translator<M>,
    src_loc: RegisterLocation,
    dst_loc: RegisterLocation,
    op: impl FnOnce(Operand, Operand) -> X86Instruction,
) -> Result<(), String> {
    let mapping = &translator.context().register_mapping;
    // Convert locations to operands
    let src_op = mapping.location_to_operand(src_loc)?;
    let dst_op = mapping.location_to_operand(dst_loc)?;
    translator.emit_instruction(&op(src_op, dst_op))
}

/// Translate a single RISC-V instruction to x86-64
///
/// This function pattern-matches on the RISC-V instruction type and emits
/// the appropriate x86-64 instruction sequence via the translator.
///
/// Returns an error if the instruction is not yet supported.
pub(crate) fn translate_instruction<M: RegisterMapper>(
    translator: &mut RiscvToX86Translator<M>,
    riscv_insn: &RiscvInstruction,
) -> Result<(), String> {
    use crate::decode::Instruction::*;

    // Get the register mapping from the translator context
    let mapping = &translator.context().register_mapping;

    match riscv_insn {
        // === Basic ALU Operations (R-type) ===
        // ADD rd, rs1, rs2 → X[rd] = (X[rs1] + X[rs2]) mod 2^64
        Add(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);

            if r.rd != r.rs1 {
                emit_bin_op(translator, rs1_loc, rd_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
            }
            emit_bin_op(translator, rs2_loc, rd_loc, |src, dst| {
                X86Instruction::Add { src, dst }
            })?;
        }

        // SUB rd, rs1, rs2 → X[rd] = (X[rs1] - X[rs2]) mod 2^64
        Sub(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);

            if r.rd != r.rs1 {
                emit_bin_op(translator, rs1_loc, rd_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
            }
            emit_bin_op(translator, rs2_loc, rd_loc, |src, dst| {
                X86Instruction::Sub { src, dst }
            })?;
        }

        // AND rd, rs1, rs2 → X[rd] = X[rs1] & X[rs2]
        And(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);

            if r.rd != r.rs1 {
                emit_bin_op(translator, rs1_loc, rd_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
            }
            emit_bin_op(translator, rs2_loc, rd_loc, |src, dst| {
                X86Instruction::And { src, dst }
            })?;
        }

        // OR rd, rs1, rs2 → X[rd] = X[rs1] | X[rs2]
        Or(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);

            if r.rd != r.rs1 {
                emit_bin_op(translator, rs1_loc, rd_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
            }
            emit_bin_op(translator, rs2_loc, rd_loc, |src, dst| X86Instruction::Or {
                src,
                dst,
            })?;
        }

        // XOR rd, rs1, rs2 → X[rd] = X[rs1] ^ X[rs2]
        Xor(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);

            if r.rd != r.rs1 {
                emit_bin_op(translator, rs1_loc, rd_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
            }
            emit_bin_op(translator, rs2_loc, rd_loc, |src, dst| {
                X86Instruction::Xor { src, dst }
            })?;
        }

        // === ALU Immediate Operations (I-type) ===
        // TODO: I-type instructions need refactoring to use RegisterLocation pattern
        Addi(_i) => return Err("ADDI: Refactoring in progress".to_string()),
        Ori(_i) => return Err("ORI: Refactoring in progress".to_string()),
        Andi(_i) => return Err("ANDI: Refactoring in progress".to_string()),
        Xori(_i) => return Err("XORI: Refactoring in progress".to_string()),

        // === Shift Operations ===
        // TODO: Shift instructions need refactoring to use RegisterLocation pattern
        Slli(_sh) => return Err("SLLI: Refactoring in progress".to_string()),
        Srli(_sh) => return Err("SRLI: Refactoring in progress".to_string()),
        Srai(_sh) => return Err("SRAI: Refactoring in progress".to_string()),
        Sll(_r) => return Err("SLL: Refactoring in progress".to_string()),
        Srl(_r) => return Err("SRL: Refactoring in progress".to_string()),
        Sra(_r) => return Err("SRA: Refactoring in progress".to_string()),

        // === Comparison Operations ===
        // TODO: Comparison instructions need refactoring to use RegisterLocation pattern
        Slt(_r) => return Err("SLT: Refactoring in progress".to_string()),
        Sltu(_r) => return Err("SLTU: Refactoring in progress".to_string()),
        Slti(_i) => return Err("SLTI: Refactoring in progress".to_string()),
        Sltiu(_i) => return Err("SLTIU: Refactoring in progress".to_string()),

        _ => {
            return Err(format!("Instruction not yet translated: {:?}", riscv_insn));
        }
    }

    Ok(())
}
