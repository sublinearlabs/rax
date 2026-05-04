use crate::aot::register_mapping::RegisterLocation;
/// RISC-V instruction to x86-64 translation logic
///
/// This module contains the main translation dispatch for converting RISC-V instructions
/// to x86-64 bytecode. Each RISC-V instruction generates 1-4 x86-64 instructions.
use crate::decode::Instruction as RiscvInstruction;
use crate::translate::register_mapper::RegisterMapper;
use crate::translate::translator::RiscvToX86Translator;
use crate::translate::x86_insn::{Operand, X86Instruction, X86Register};

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

    // Check if we need to use temporary registers for memory-to-memory operations
    let is_src_mem = matches!(src_op, Operand::Memory { .. } | Operand::AbsoluteAddress(_));
    let is_dst_mem = matches!(dst_op, Operand::Memory { .. } | Operand::AbsoluteAddress(_));

    if is_src_mem && is_dst_mem {
        // Memory-to-memory operation: need two temp registers
        // Load src into R12, load dst into R13, perform operation on R13, store back to dst
        let src_temp = Operand::Register(X86Register::R12);
        let dst_temp = Operand::Register(X86Register::R13);

        // Load source into R12
        translator.emit_instruction(&X86Instruction::Mov {
            src: src_op,
            dst: src_temp.clone(),
        })?;

        // Load destination into R13
        translator.emit_instruction(&X86Instruction::Mov {
            src: dst_op.clone(),
            dst: dst_temp.clone(),
        })?;

        // Perform operation: op(R12, R13) where R13 is the destination
        translator.emit_instruction(&op(src_temp, dst_temp.clone()))?;

        // Store result back to original destination
        translator.emit_instruction(&X86Instruction::Mov {
            src: dst_temp,
            dst: dst_op,
        })?;
        Ok(())
    } else if is_src_mem {
        // Source in memory, destination in register: load src into temp, then operate
        let temp_reg = Operand::Register(X86Register::R12);
        translator.emit_instruction(&X86Instruction::Mov {
            src: src_op,
            dst: temp_reg.clone(),
        })?;
        translator.emit_instruction(&op(temp_reg, dst_op))
    } else {
        // Standard case: at least one operand is a register
        translator.emit_instruction(&op(src_op, dst_op))
    }
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

            // Special case: if rd == rs2, we need a temp register to avoid clobbering rs2
            if r.rd == r.rs2 && r.rd != r.rs1 {
                let temp_loc = RegisterLocation::GPR(12); // r12 is available as temp
                                                          // temp = rs1
                emit_bin_op(translator, rs1_loc, temp_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
                // temp += rs2
                emit_bin_op(translator, rs2_loc, temp_loc, |src, dst| {
                    X86Instruction::Add { src, dst }
                })?;
                // rd = temp
                emit_bin_op(translator, temp_loc, rd_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
            } else {
                // Normal case: rd != rs2 (or rd == rs1)
                if r.rd != r.rs1 {
                    emit_bin_op(translator, rs1_loc, rd_loc, |src, dst| {
                        X86Instruction::Mov { src, dst }
                    })?;
                }
                emit_bin_op(translator, rs2_loc, rd_loc, |src, dst| {
                    X86Instruction::Add { src, dst }
                })?;
            }
        }

        // SUB rd, rs1, rs2 → X[rd] = (X[rs1] - X[rs2]) mod 2^64
        Sub(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);

            // Special case: if rd == rs2, we need a temp register to avoid clobbering rs2
            if r.rd == r.rs2 && r.rd != r.rs1 {
                let temp_loc = RegisterLocation::GPR(12); // r12 is available as temp
                                                          // temp = rs1
                emit_bin_op(translator, rs1_loc, temp_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
                // temp -= rs2
                emit_bin_op(translator, rs2_loc, temp_loc, |src, dst| {
                    X86Instruction::Sub { src, dst }
                })?;
                // rd = temp
                emit_bin_op(translator, temp_loc, rd_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
            } else {
                // Normal case: rd != rs2 (or rd == rs1)
                if r.rd != r.rs1 {
                    emit_bin_op(translator, rs1_loc, rd_loc, |src, dst| {
                        X86Instruction::Mov { src, dst }
                    })?;
                }
                emit_bin_op(translator, rs2_loc, rd_loc, |src, dst| {
                    X86Instruction::Sub { src, dst }
                })?;
            }
        }

        // AND rd, rs1, rs2 → X[rd] = X[rs1] & X[rs2]
        And(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);

            // Special case: if rd == rs2, we need a temp register to avoid clobbering rs2
            if r.rd == r.rs2 && r.rd != r.rs1 {
                let temp_loc = RegisterLocation::GPR(12); // r12 is available as temp
                                                          // temp = rs1
                emit_bin_op(translator, rs1_loc, temp_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
                // temp &= rs2
                emit_bin_op(translator, rs2_loc, temp_loc, |src, dst| {
                    X86Instruction::And { src, dst }
                })?;
                // rd = temp
                emit_bin_op(translator, temp_loc, rd_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
            } else {
                // Normal case: rd != rs2 (or rd == rs1)
                if r.rd != r.rs1 {
                    emit_bin_op(translator, rs1_loc, rd_loc, |src, dst| {
                        X86Instruction::Mov { src, dst }
                    })?;
                }
                emit_bin_op(translator, rs2_loc, rd_loc, |src, dst| {
                    X86Instruction::And { src, dst }
                })?;
            }
        }

        // OR rd, rs1, rs2 → X[rd] = X[rs1] | X[rs2]
        Or(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);

            // Special case: if rd == rs2, we need a temp register to avoid clobbering rs2
            if r.rd == r.rs2 && r.rd != r.rs1 {
                let temp_loc = RegisterLocation::GPR(12); // r12 is available as temp
                                                          // temp = rs1
                emit_bin_op(translator, rs1_loc, temp_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
                // temp |= rs2
                emit_bin_op(translator, rs2_loc, temp_loc, |src, dst| {
                    X86Instruction::Or { src, dst }
                })?;
                // rd = temp
                emit_bin_op(translator, temp_loc, rd_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
            } else {
                // Normal case: rd != rs2 (or rd == rs1)
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
        }

        // XOR rd, rs1, rs2 → X[rd] = X[rs1] ^ X[rs2]
        Xor(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);

            // Special case: if rd == rs2, we need a temp register to avoid clobbering rs2
            if r.rd == r.rs2 && r.rd != r.rs1 {
                let temp_loc = RegisterLocation::GPR(12); // r12 is available as temp
                                                          // temp = rs1
                emit_bin_op(translator, rs1_loc, temp_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
                // temp ^= rs2
                emit_bin_op(translator, rs2_loc, temp_loc, |src, dst| {
                    X86Instruction::Xor { src, dst }
                })?;
                // rd = temp
                emit_bin_op(translator, temp_loc, rd_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
            } else {
                // Normal case: rd != rs2 (or rd == rs1)
                if r.rd != r.rs1 {
                    emit_bin_op(translator, rs1_loc, rd_loc, |src, dst| {
                        X86Instruction::Mov { src, dst }
                    })?;
                }
                emit_bin_op(translator, rs2_loc, rd_loc, |src, dst| {
                    X86Instruction::Xor { src, dst }
                })?;
            }
        }

        // MUL rd, rs1, rs2 → X[rd] = (X[rs1] × X[rs2])[63:0] (lower 64 bits)
        Mul(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);
            let rd_op = mapping.location_to_operand(rd_loc)?;
            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rs2_op = mapping.location_to_operand(rs2_loc)?;

            let r13 = Operand::Register(X86Register::R13);
            let r14 = Operand::Register(X86Register::R14);

            // For IMUL (2-operand form), we can use any registers
            // IMUL dst, src → dst ← dst × src
            // We'll use IMUL with R13 and R14 as temporaries

            // Move rs1 to R13
            translator.emit_instruction(&X86Instruction::Mov {
                src: rs1_op,
                dst: r13.clone(),
            })?;

            // Move rs2 to R14
            translator.emit_instruction(&X86Instruction::Mov {
                src: rs2_op,
                dst: r14.clone(),
            })?;

            // IMUL R13, R14 → R13 ← R13 × R14
            translator.emit_instruction(&X86Instruction::Imul {
                src: r14,
                dst: r13.clone(),
            })?;

            // Move result to destination
            translator.emit_instruction(&X86Instruction::Mov {
                src: r13,
                dst: rd_op,
            })?;
        }

        // MULH rd, rs1, rs2 → X[rd] = (X[rs1] × X[rs2])[127:64] (upper 64 bits, signed)
        Mulh(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);
            let rd_op = mapping.location_to_operand(rd_loc)?;
            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rs2_op = mapping.location_to_operand(rs2_loc)?;

            let rax = Operand::Register(X86Register::RAX);
            let rdx = Operand::Register(X86Register::RDX);
            let r12 = Operand::Register(X86Register::R12);

            // Save rax and rdx values
            translator.emit_instruction(&X86Instruction::Push { src: rax.clone() })?;
            translator.emit_instruction(&X86Instruction::Push { src: rdx.clone() })?;

            // Move rs1 to RAX
            translator.emit_instruction(&X86Instruction::Mov {
                src: rs1_op,
                dst: rax.clone(),
            })?;

            // Move rs2 to R12
            translator.emit_instruction(&X86Instruction::Mov {
                src: rs2_op,
                dst: r12.clone(),
            })?;

            // Sign-extend RAX to RDX:RAX
            translator.emit_instruction(&X86Instruction::Cqo)?;

            // IMUL R12 (1-operand form): RDX:RAX = RAX * R12
            translator.emit_instruction(&X86Instruction::Imul {
                src: r12,
                dst: rax.clone(),
            })?;

            // Result upper 64 bits are in RDX, move to destination
            translator.emit_instruction(&X86Instruction::Mov {
                src: rdx.clone(),
                dst: rd_op,
            })?;

            // Restore rax and rdx
            translator.emit_instruction(&X86Instruction::Pop { dst: rdx })?;
            translator.emit_instruction(&X86Instruction::Pop { dst: rax })?;
        }

        // === ALU Immediate Operations (I-type) ===
        // ADDI rd, rs1, imm → X[rd] = X[rs1] + sign_extend(imm)
        Addi(i) => {
            let rd_loc = mapping.get_register_location(i.rd);
            let rs1_loc = mapping.get_register_location(i.rs1);
            let src_op = mapping.location_to_operand(rs1_loc)?;
            let dst_op = mapping.location_to_operand(rd_loc)?;
            let imm_op = Operand::Immediate(i.imm as i64);

            // Handle destination based on type
            match rd_loc {
                RegisterLocation::MEM(_) => {
                    // Memory destination: need to use temp register
                    if i.rs1 == 0 {
                        // Zero register: move immediate to temp, then to memory
                        let temp = Operand::Register(X86Register::R12);
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: imm_op,
                            dst: temp.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: temp,
                            dst: dst_op,
                        })?;
                    } else {
                        // Non-zero source

                        let temp = Operand::Register(X86Register::R13);

                        // Load rs1 into temp
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: src_op,
                            dst: temp.clone(),
                        })?;

                        // Add immediate to temp
                        if i.imm != 0 {
                            translator.emit_instruction(&X86Instruction::Add {
                                src: imm_op,
                                dst: temp.clone(),
                            })?;
                        }

                        // Move temp to memory destination
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: temp,
                            dst: dst_op,
                        })?;
                    }
                }
                _ => {
                    // Register destination
                    if i.rs1 == 0 {
                        // Zero register: li rd, imm
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: imm_op,
                            dst: dst_op,
                        })?;
                    } else {
                        // Standard addi: rd = rs1 + imm
                        // If rd != rs1, move rs1 to rd first
                        if i.rd != i.rs1 {
                            translator.emit_instruction(&X86Instruction::Mov {
                                src: src_op,
                                dst: dst_op.clone(),
                            })?;
                        }

                        // Add immediate to rd
                        if i.imm != 0 {
                            translator.emit_instruction(&X86Instruction::Add {
                                src: imm_op,
                                dst: dst_op,
                            })?;
                        }
                    }
                }
            }
        }
        // ORI rd, rs1, imm → X[rd] = X[rs1] | imm
        Ori(i) => {
            let rd_loc = mapping.get_register_location(i.rd);
            let rs1_loc = mapping.get_register_location(i.rs1);
            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rd_op = mapping.location_to_operand(rd_loc)?;

            // Handle memory destinations
            match rd_op {
                Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                    // Memory destination: load rs1 into R14, OR immediate, store back
                    let temp_reg = Operand::Register(X86Register::R14);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op,
                        dst: temp_reg.clone(),
                    })?;
                    let imm_op = Operand::Immediate(i.imm as i64);
                    translator.emit_instruction(&X86Instruction::Or {
                        src: imm_op,
                        dst: temp_reg.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: temp_reg,
                        dst: rd_op,
                    })?;
                }
                Operand::Register(_) => {
                    // Register destination: standard path
                    if i.rd != i.rs1 {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: rd_op.clone(),
                        })?;
                    }
                    let imm_op = Operand::Immediate(i.imm as i64);
                    translator.emit_instruction(&X86Instruction::Or {
                        src: imm_op,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("ORI: invalid destination operand".to_string()),
            }
        }

        // ANDI rd, rs1, imm → X[rd] = X[rs1] & imm
        Andi(i) => {
            let rd_loc = mapping.get_register_location(i.rd);
            let rs1_loc = mapping.get_register_location(i.rs1);
            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rd_op = mapping.location_to_operand(rd_loc)?;

            // Handle memory destinations
            match rd_op {
                Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                    // Memory destination: load rs1 into R14, AND immediate, store back
                    let temp_reg = Operand::Register(X86Register::R14);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op,
                        dst: temp_reg.clone(),
                    })?;
                    let imm_op = Operand::Immediate(i.imm as i64);
                    translator.emit_instruction(&X86Instruction::And {
                        src: imm_op,
                        dst: temp_reg.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: temp_reg,
                        dst: rd_op,
                    })?;
                }
                Operand::Register(_) => {
                    // Register destination: standard path
                    if i.rd != i.rs1 {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: rd_op.clone(),
                        })?;
                    }
                    let imm_op = Operand::Immediate(i.imm as i64);
                    translator.emit_instruction(&X86Instruction::And {
                        src: imm_op,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("ANDI: invalid destination operand".to_string()),
            }
        }

        // XORI rd, rs1, imm → X[rd] = X[rs1] ^ imm
        Xori(i) => {
            let rd_loc = mapping.get_register_location(i.rd);
            let rs1_loc = mapping.get_register_location(i.rs1);
            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rd_op = mapping.location_to_operand(rd_loc)?;

            // Handle memory destinations
            match rd_op {
                Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                    // Memory destination: load rs1 into R14, XOR immediate, store back
                    let temp_reg = Operand::Register(X86Register::R14);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op,
                        dst: temp_reg.clone(),
                    })?;
                    let imm_op = Operand::Immediate(i.imm as i64);
                    translator.emit_instruction(&X86Instruction::Xor {
                        src: imm_op,
                        dst: temp_reg.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: temp_reg,
                        dst: rd_op,
                    })?;
                }
                Operand::Register(_) => {
                    // Register destination: standard path
                    if i.rd != i.rs1 {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: rd_op.clone(),
                        })?;
                    }
                    let imm_op = Operand::Immediate(i.imm as i64);
                    translator.emit_instruction(&X86Instruction::Xor {
                        src: imm_op,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("XORI: invalid destination operand".to_string()),
            }
        }

        // === 32-bit Word Operations (RV64I) ===
        // These perform 32-bit arithmetic and sign-extend the result to 64 bits

        // ADDW rd, rs1, rs2 → X[rd] = (X[rs1][31:0] + X[rs2][31:0])[31:0] sign-extended
        Addw(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);
            let rd_op = mapping.location_to_operand(rd_loc)?;

            // Move rs1 to rd if different
            if r.rd != r.rs1 {
                emit_bin_op(translator, rs1_loc, rd_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
            }
            // Add rs2 to rd
            emit_bin_op(translator, rs2_loc, rd_loc, |src, dst| {
                X86Instruction::Add { src, dst }
            })?;

            // Sign-extend the 32-bit result to 64 bits using MOVSXD
            translator
                .emitter_mut()
                .emit_movsx_32_to_64(&rd_op, &rd_op)?;
        }

        // SUBW rd, rs1, rs2 → X[rd] = (X[rs1][31:0] - X[rs2][31:0])[31:0] sign-extended
        Subw(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);
            let rd_op = mapping.location_to_operand(rd_loc)?;

            // Move rs1 to rd if different
            if r.rd != r.rs1 {
                emit_bin_op(translator, rs1_loc, rd_loc, |src, dst| {
                    X86Instruction::Mov { src, dst }
                })?;
            }
            // Subtract rs2 from rd
            emit_bin_op(translator, rs2_loc, rd_loc, |src, dst| {
                X86Instruction::Sub { src, dst }
            })?;

            // Sign-extend the 32-bit result to 64 bits using MOVSXD
            translator
                .emitter_mut()
                .emit_movsx_32_to_64(&rd_op, &rd_op)?;
        }

        // ADDIW rd, rs1, imm → X[rd] = (X[rs1][31:0] + imm)[31:0] sign-extended
        Addiw(i) => {
            let rd_loc = mapping.get_register_location(i.rd);
            let rs1_loc = mapping.get_register_location(i.rs1);
            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rd_op = mapping.location_to_operand(rd_loc)?;

            // Move rs1 to rd if different
            if i.rd != i.rs1 {
                translator.emit_instruction(&X86Instruction::Mov {
                    src: rs1_op,
                    dst: rd_op.clone(),
                })?;
            }

            // Add immediate to rd (only if imm is non-zero)
            if i.imm != 0 {
                let imm_op = Operand::Immediate(i.imm as i64);
                translator.emit_instruction(&X86Instruction::Add {
                    src: imm_op,
                    dst: rd_op.clone(),
                })?;
            }

            // Sign-extend the 32-bit result to 64 bits using MOVSXD
            translator
                .emitter_mut()
                .emit_movsx_32_to_64(&rd_op, &rd_op)?;
        }

        // === Store Instructions (S-type) ===
        // SD rs2, imm(rs1) → M[X[rs1] + imm] = X[rs2][63:0]
        Sd(s) => {
            let rs1_loc = mapping.get_register_location(s.rs1);
            let rs2_loc = mapping.get_register_location(s.rs2);

            let mut value_op = mapping.location_to_operand(rs2_loc)?;
            let base_op = mapping.location_to_operand(rs1_loc)?;

            // Create memory operand for the store destination
            let mem_op = match base_op {
                Operand::Register(base_reg) => Operand::Memory {
                    base: base_reg,
                    offset: s.imm as i32,
                },
                Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                    // Base is in memory, load it into temp R11 first
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: base_op,
                        dst: temp_reg.clone(),
                    })?;
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: s.imm as i32,
                    }
                }
                _ => {
                    return Err("SD: rs1 must resolve to a register or memory location".to_string())
                }
            };

            // If the value to store is in memory, load it into temp R12 first
            // x86-64 doesn't support memory-to-memory moves
            if let Operand::Memory { .. } | Operand::AbsoluteAddress(_) = value_op {
                let temp_reg = Operand::Register(X86Register::R12);
                translator.emit_instruction(&X86Instruction::Mov {
                    src: value_op,
                    dst: temp_reg.clone(),
                })?;
                value_op = temp_reg;
            }

            // Emit: MOV [rs1 + imm], rs2
            translator.emit_instruction(&X86Instruction::Mov {
                src: value_op,
                dst: mem_op,
            })?;
        }

        // Store other sizes not yet implemented
        // SB rs2, imm(rs1) → M[X[rs1] + imm] = X[rs2][7:0]
        Sb(s) => {
            let rs1_loc = mapping.get_register_location(s.rs1);
            let rs2_loc = mapping.get_register_location(s.rs2);

            let mut value_op = mapping.location_to_operand(rs2_loc)?;

            // Get base address from rs1 - if it's in memory, we need a temp register
            let base_op = mapping.location_to_operand(rs1_loc)?;

            let mem_op =
                match base_op {
                    Operand::Register(base_reg) => {
                        // Base is already in a GPR, use it directly
                        Operand::Memory {
                            base: base_reg,
                            offset: s.imm as i32,
                        }
                    }
                    Operand::Memory { base: _, offset: _ } => {
                        // Base is in memory (x16-x31), load it into temp R11 first
                        let temp_reg = Operand::Register(X86Register::R11);
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: base_op,
                            dst: temp_reg.clone(),
                        })?;

                        // Now use temp register as base
                        Operand::Memory {
                            base: X86Register::R11,
                            offset: s.imm as i32,
                        }
                    }
                    Operand::AbsoluteAddress(_addr) => {
                        // Base is an absolute address (e.g., BSS location for spilled registers)
                        // Load the address into a temp register first
                        let temp_reg = Operand::Register(X86Register::R11);
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: base_op.clone(),
                            dst: temp_reg.clone(),
                        })?;

                        // Now use temp register as base
                        Operand::Memory {
                            base: X86Register::R11,
                            offset: s.imm as i32,
                        }
                    }
                    _ => return Err(
                        "SB: rs1 must resolve to a register, memory location, or absolute address"
                            .to_string(),
                    ),
                };

            // If the value to store is in memory, load it into a temp register first
            // x86-64 doesn't support memory-to-memory moves
            if let Operand::Memory { .. } | Operand::AbsoluteAddress(_) = value_op {
                let temp_reg = Operand::Register(X86Register::R12);
                translator.emit_instruction(&X86Instruction::Mov {
                    src: value_op,
                    dst: temp_reg.clone(),
                })?;
                value_op = temp_reg;
            }

            // Emit: MOV byte [address], rs2
            translator.emit_instruction(&X86Instruction::Mov {
                src: value_op,
                dst: mem_op,
            })?;
        }

        // SH rs2, imm(rs1) → M[X[rs1] + imm] = X[rs2][15:0]
        Sh(s) => {
            let rs1_loc = mapping.get_register_location(s.rs1);
            let rs2_loc = mapping.get_register_location(s.rs2);

            let mut value_op = mapping.location_to_operand(rs2_loc)?;

            let base_op = mapping.location_to_operand(rs1_loc)?;

            let mem_op = match base_op {
                Operand::Register(base_reg) => Operand::Memory {
                    base: base_reg,
                    offset: s.imm as i32,
                },
                Operand::Memory { base: _, offset: _ } => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: base_op,
                        dst: temp_reg.clone(),
                    })?;

                    Operand::Memory {
                        base: X86Register::R11,
                        offset: s.imm as i32,
                    }
                }
                _ => {
                    return Err("SH: rs1 must resolve to a register or memory location".to_string())
                }
            };

            // If the value to store is in memory, load it into a temp register first
            if let Operand::Memory { .. } | Operand::AbsoluteAddress(_) = value_op {
                let temp_reg = Operand::Register(X86Register::R12);
                translator.emit_instruction(&X86Instruction::Mov {
                    src: value_op,
                    dst: temp_reg.clone(),
                })?;
                value_op = temp_reg;
            }

            // Emit: MOV word [address], rs2
            translator.emit_instruction(&X86Instruction::Mov {
                src: value_op,
                dst: mem_op,
            })?;
        }

        // SW rs2, imm(rs1) → M[X[rs1] + imm] = X[rs2][31:0]
        Sw(s) => {
            let rs1_loc = mapping.get_register_location(s.rs1);
            let rs2_loc = mapping.get_register_location(s.rs2);

            let mut value_op = mapping.location_to_operand(rs2_loc)?;

            let base_op = mapping.location_to_operand(rs1_loc)?;

            let mem_op = match base_op {
                Operand::Register(base_reg) => Operand::Memory {
                    base: base_reg,
                    offset: s.imm as i32,
                },
                Operand::Memory { base: _, offset: _ } => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: base_op,
                        dst: temp_reg.clone(),
                    })?;

                    Operand::Memory {
                        base: X86Register::R11,
                        offset: s.imm as i32,
                    }
                }
                _ => {
                    return Err("SW: rs1 must resolve to a register or memory location".to_string())
                }
            };

            // If the value to store is in memory, load it into a temp register first
            if let Operand::Memory { .. } | Operand::AbsoluteAddress(_) = value_op {
                let temp_reg = Operand::Register(X86Register::R12);
                translator.emit_instruction(&X86Instruction::Mov {
                    src: value_op,
                    dst: temp_reg.clone(),
                })?;
                value_op = temp_reg;
            }

            // Emit: MOV dword [address], rs2
            translator.emit_instruction(&X86Instruction::Mov {
                src: value_op,
                dst: mem_op,
            })?;
        }

        // === Upper Immediate Operations (U-type) ===
        // AUIPC rd, imm → X[rd] = PC + (imm << 12)
        Auipc(u) => {
            let rd_loc = mapping.get_register_location(u.rd);
            let rd_op = mapping.location_to_operand(rd_loc)?;

            let riscv_pc = translator.context().pc;
            let upper_imm = (u.imm as i64) << 12;
            let riscv_result = (riscv_pc as i64).wrapping_add(upper_imm) as u64;

            // If destination is in memory, use a temporary register first
            match rd_op {
                Operand::Register(_) => {
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::Immediate(riscv_result as i64),
                        dst: rd_op,
                    })?;
                }
                Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                    // Memory destination: load into temp R14, then store to memory
                    let temp_reg = Operand::Register(X86Register::R14);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::Immediate(riscv_result as i64),
                        dst: temp_reg.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: temp_reg,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("AUIPC: invalid destination operand".to_string()),
            }
        }

        // === Load Instructions (I-type) ===
        // LD rd, imm(rs1) → X[rd] = M[X[rs1] + imm][63:0] (load 64-bit/doubleword from memory)
        Ld(i) => {
            let rd_loc = mapping.get_register_location(i.rd);
            let rs1_loc = mapping.get_register_location(i.rs1);

            let rd_op = mapping.location_to_operand(rd_loc)?;
            let base_op = mapping.location_to_operand(rs1_loc)?;

            // Create memory operand for the load source
            let mem_op = match base_op {
                Operand::Register(base_reg) => Operand::Memory {
                    base: base_reg,
                    offset: i.imm as i32,
                },
                Operand::Memory {
                    base,
                    offset: base_offset,
                } => {
                    // Base is in memory at [base + base_offset]
                    // We need to load it into a temp register first
                    let temp_reg = Operand::Register(X86Register::R11);

                    // Load the pointer from [base + base_offset] into R11
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::Memory {
                            base,
                            offset: base_offset,
                        },
                        dst: temp_reg.clone(),
                    })?;

                    // Now use R11 as the base for the actual load
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                Operand::AbsoluteAddress(addr) => {
                    // Base is at an absolute address
                    // Load it into R11 first
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::AbsoluteAddress(addr),
                        dst: temp_reg.clone(),
                    })?;

                    // Now use R11 as the base for the actual load
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                _ => {
                    return Err("LD: rs1 must resolve to a register or memory location".to_string())
                }
            };

            // Emit: MOV rd, [rs1 + imm]
            // If rd is in memory, we need to load into a temp register first
            match rd_op {
                Operand::Register(_) => {
                    // Standard case: destination is a register, load directly
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: mem_op,
                        dst: rd_op,
                    })?;
                }
                Operand::AbsoluteAddress(_) | Operand::Memory { .. } => {
                    // Destination is in memory - load into temp R12 (reserved scratch) first, then move to destination
                    let temp_reg = Operand::Register(X86Register::R12);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: mem_op,
                        dst: temp_reg.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: temp_reg,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("LD: invalid destination operand".to_string()),
            }
        }

        // LB rd, imm(rs1) → X[rd] = sign_extend(M[X[rs1] + imm][7:0]) (load byte signed)
        Lb(i) => {
            let rd_loc = mapping.get_register_location(i.rd);
            let rs1_loc = mapping.get_register_location(i.rs1);

            let rd_op = mapping.location_to_operand(rd_loc)?;
            let base_op = mapping.location_to_operand(rs1_loc)?;

            // Create memory operand for the load source
            let mem_op = match base_op {
                Operand::Register(base_reg) => Operand::Memory {
                    base: base_reg,
                    offset: i.imm as i32,
                },
                Operand::Memory {
                    base,
                    offset: base_offset,
                } => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::Memory {
                            base,
                            offset: base_offset,
                        },
                        dst: temp_reg.clone(),
                    })?;
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                Operand::AbsoluteAddress(addr) => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::AbsoluteAddress(addr),
                        dst: temp_reg.clone(),
                    })?;
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                _ => {
                    return Err("LB: rs1 must resolve to a register or memory location".to_string())
                }
            };

            // Load byte with sign extension (MOVSX)
            match rd_op {
                Operand::Register(_) => {
                    translator.emit_instruction(&X86Instruction::Movsx {
                        src: mem_op,
                        dst: rd_op,
                    })?;
                }
                Operand::AbsoluteAddress(_) | Operand::Memory { .. } => {
                    let temp_reg = Operand::Register(X86Register::R12);
                    translator.emit_instruction(&X86Instruction::Movsx {
                        src: mem_op,
                        dst: temp_reg.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: temp_reg,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("LB: invalid destination operand".to_string()),
            }
        }

        // LBU rd, imm(rs1) → X[rd] = zero_extend(M[X[rs1] + imm][7:0]) (load byte unsigned)
        Lbu(i) => {
            let rd_loc = mapping.get_register_location(i.rd);
            let rs1_loc = mapping.get_register_location(i.rs1);

            let rd_op = mapping.location_to_operand(rd_loc)?;
            let base_op = mapping.location_to_operand(rs1_loc)?;

            // Create memory operand for the load source
            let mem_op = match base_op {
                Operand::Register(base_reg) => Operand::Memory {
                    base: base_reg,
                    offset: i.imm as i32,
                },
                Operand::Memory {
                    base,
                    offset: base_offset,
                } => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::Memory {
                            base,
                            offset: base_offset,
                        },
                        dst: temp_reg.clone(),
                    })?;
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                Operand::AbsoluteAddress(addr) => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::AbsoluteAddress(addr),
                        dst: temp_reg.clone(),
                    })?;
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                _ => {
                    return Err("LBU: rs1 must resolve to a register or memory location".to_string())
                }
            };

            // Load byte with zero extension (MOVZX)
            match rd_op {
                Operand::Register(_) => {
                    translator.emit_instruction(&X86Instruction::Movzx {
                        src: mem_op,
                        dst: rd_op,
                    })?;
                }
                Operand::AbsoluteAddress(_) | Operand::Memory { .. } => {
                    let temp_reg = Operand::Register(X86Register::R12);
                    translator.emit_instruction(&X86Instruction::Movzx {
                        src: mem_op,
                        dst: temp_reg.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: temp_reg,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("LBU: invalid destination operand".to_string()),
            }
        }

        // LH rd, imm(rs1) → X[rd] = sign_extend(M[X[rs1] + imm][15:0]) (load halfword signed)
        Lh(i) => {
            let rd_loc = mapping.get_register_location(i.rd);
            let rs1_loc = mapping.get_register_location(i.rs1);

            let rd_op = mapping.location_to_operand(rd_loc)?;
            let base_op = mapping.location_to_operand(rs1_loc)?;

            // Create memory operand for the load source
            let mem_op = match base_op {
                Operand::Register(base_reg) => Operand::Memory {
                    base: base_reg,
                    offset: i.imm as i32,
                },
                Operand::Memory {
                    base,
                    offset: base_offset,
                } => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::Memory {
                            base,
                            offset: base_offset,
                        },
                        dst: temp_reg.clone(),
                    })?;
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                Operand::AbsoluteAddress(addr) => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::AbsoluteAddress(addr),
                        dst: temp_reg.clone(),
                    })?;
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                _ => {
                    return Err("LH: rs1 must resolve to a register or memory location".to_string())
                }
            };

            // Load halfword with sign extension (MOVSX)
            match rd_op {
                Operand::Register(_) => {
                    translator.emit_instruction(&X86Instruction::Movsx {
                        src: mem_op,
                        dst: rd_op,
                    })?;
                }
                Operand::AbsoluteAddress(_) | Operand::Memory { .. } => {
                    let temp_reg = Operand::Register(X86Register::R12);
                    translator.emit_instruction(&X86Instruction::Movsx {
                        src: mem_op,
                        dst: temp_reg.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: temp_reg,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("LH: invalid destination operand".to_string()),
            }
        }

        // LHU rd, imm(rs1) → X[rd] = zero_extend(M[X[rs1] + imm][15:0]) (load halfword unsigned)
        Lhu(i) => {
            let rd_loc = mapping.get_register_location(i.rd);
            let rs1_loc = mapping.get_register_location(i.rs1);

            let rd_op = mapping.location_to_operand(rd_loc)?;
            let base_op = mapping.location_to_operand(rs1_loc)?;

            // Create memory operand for the load source
            let mem_op = match base_op {
                Operand::Register(base_reg) => Operand::Memory {
                    base: base_reg,
                    offset: i.imm as i32,
                },
                Operand::Memory {
                    base,
                    offset: base_offset,
                } => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::Memory {
                            base,
                            offset: base_offset,
                        },
                        dst: temp_reg.clone(),
                    })?;
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                Operand::AbsoluteAddress(addr) => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::AbsoluteAddress(addr),
                        dst: temp_reg.clone(),
                    })?;
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                _ => {
                    return Err("LHU: rs1 must resolve to a register or memory location".to_string())
                }
            };

            // Load halfword with zero extension (MOVZX)
            match rd_op {
                Operand::Register(_) => {
                    translator.emit_instruction(&X86Instruction::Movzx {
                        src: mem_op,
                        dst: rd_op,
                    })?;
                }
                Operand::AbsoluteAddress(_) | Operand::Memory { .. } => {
                    let temp_reg = Operand::Register(X86Register::R12);
                    translator.emit_instruction(&X86Instruction::Movzx {
                        src: mem_op,
                        dst: temp_reg.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: temp_reg,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("LHU: invalid destination operand".to_string()),
            }
        }

        // LW rd, imm(rs1) → X[rd] = sign_extend(M[X[rs1] + imm][31:0]) (load word signed)
        Lw(i) => {
            let rd_loc = mapping.get_register_location(i.rd);
            let rs1_loc = mapping.get_register_location(i.rs1);

            let rd_op = mapping.location_to_operand(rd_loc)?;
            let base_op = mapping.location_to_operand(rs1_loc)?;

            // Create memory operand for the load source
            let mem_op = match base_op {
                Operand::Register(base_reg) => Operand::Memory {
                    base: base_reg,
                    offset: i.imm as i32,
                },
                Operand::Memory {
                    base,
                    offset: base_offset,
                } => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::Memory {
                            base,
                            offset: base_offset,
                        },
                        dst: temp_reg.clone(),
                    })?;
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                Operand::AbsoluteAddress(addr) => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::AbsoluteAddress(addr),
                        dst: temp_reg.clone(),
                    })?;
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                _ => {
                    return Err("LW: rs1 must resolve to a register or memory location".to_string())
                }
            };

            // Load word with sign extension (MOVSX)
            match rd_op {
                Operand::Register(_) => {
                    translator.emit_instruction(&X86Instruction::Movsx {
                        src: mem_op,
                        dst: rd_op,
                    })?;
                }
                Operand::AbsoluteAddress(_) | Operand::Memory { .. } => {
                    let temp_reg = Operand::Register(X86Register::R12);
                    translator.emit_instruction(&X86Instruction::Movsx {
                        src: mem_op,
                        dst: temp_reg.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: temp_reg,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("LW: invalid destination operand".to_string()),
            }
        }

        // LWU rd, imm(rs1) → X[rd] = zero_extend(M[X[rs1] + imm][31:0]) (load word unsigned)
        Lwu(i) => {
            let rd_loc = mapping.get_register_location(i.rd);
            let rs1_loc = mapping.get_register_location(i.rs1);

            let rd_op = mapping.location_to_operand(rd_loc)?;
            let base_op = mapping.location_to_operand(rs1_loc)?;

            // Create memory operand for the load source
            let mem_op = match base_op {
                Operand::Register(base_reg) => Operand::Memory {
                    base: base_reg,
                    offset: i.imm as i32,
                },
                Operand::Memory {
                    base,
                    offset: base_offset,
                } => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::Memory {
                            base,
                            offset: base_offset,
                        },
                        dst: temp_reg.clone(),
                    })?;
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                Operand::AbsoluteAddress(addr) => {
                    let temp_reg = Operand::Register(X86Register::R11);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::AbsoluteAddress(addr),
                        dst: temp_reg.clone(),
                    })?;
                    Operand::Memory {
                        base: X86Register::R11,
                        offset: i.imm as i32,
                    }
                }
                _ => {
                    return Err("LWU: rs1 must resolve to a register or memory location".to_string())
                }
            };

            // Load word with zero extension (MOVZX)
            match rd_op {
                Operand::Register(_) => {
                    translator.emit_instruction(&X86Instruction::Movzx {
                        src: mem_op,
                        dst: rd_op,
                    })?;
                }
                Operand::AbsoluteAddress(_) | Operand::Memory { .. } => {
                    let temp_reg = Operand::Register(X86Register::R12);
                    translator.emit_instruction(&X86Instruction::Movzx {
                        src: mem_op,
                        dst: temp_reg.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: temp_reg,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("LWU: invalid destination operand".to_string()),
            }
        }

        // LUI rd, imm → X[rd] = (imm << 12) (Load Upper Immediate)
        Lui(u) => {
            let rd_loc = mapping.get_register_location(u.rd);
            let rd_op = mapping.location_to_operand(rd_loc)?;

            // LUI: Load immediate into upper 20 bits, zeros out lower 12 bits
            let lui_value = (u.imm as i64) << 12;

            match rd_op {
                Operand::Register(_) => {
                    // Direct register destination
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::Immediate(lui_value),
                        dst: rd_op,
                    })?;
                }
                Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                    // Memory destination: load into temp R14, then store to memory
                    let temp_reg = Operand::Register(X86Register::R14);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::Immediate(lui_value),
                        dst: temp_reg.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: temp_reg,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("LUI: invalid destination operand".to_string()),
            }
        }

        // === Shift Operations ===
        // In x86-64, shift amount must be in CL (lower byte of RCX) or a constant

        // SLLI rd, rs1, imm → X[rd] = X[rs1] << imm
        Slli(sh) => {
            let rd_loc = mapping.get_register_location(sh.rd);
            let rs1_loc = mapping.get_register_location(sh.rs1);
            let rd_op = mapping.location_to_operand(rd_loc)?;
            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let r11 = Operand::Register(X86Register::R11);

            // Move rs1 to rd if different
            if sh.rd != sh.rs1 {
                match rd_op {
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: r11.clone(),
                        })?;
                    }
                    Operand::Register(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: rd_op.clone(),
                        })?;
                    }
                    _ => return Err("SLLI: Invalid destination operand".to_string()),
                }
            }

            // Shift rd left by immediate amount
            let shift_amount = Operand::Immediate(sh.shamt as i64);
            match rd_op {
                Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                    translator.emit_instruction(&X86Instruction::Shl {
                        src: shift_amount,
                        dst: r11.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: r11,
                        dst: rd_op,
                    })?;
                }
                Operand::Register(_) => {
                    translator.emit_instruction(&X86Instruction::Shl {
                        src: shift_amount,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("SLLI: Invalid destination operand".to_string()),
            }
        }

        // SRLI rd, rs1, imm → X[rd] = X[rs1] >> imm (logical)
        Srli(sh) => {
            let rd_loc = mapping.get_register_location(sh.rd);
            let rs1_loc = mapping.get_register_location(sh.rs1);
            let rd_op = mapping.location_to_operand(rd_loc)?;
            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let r11 = Operand::Register(X86Register::R11);

            if sh.rd != sh.rs1 {
                match rd_op {
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: r11.clone(),
                        })?;
                    }
                    Operand::Register(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: rd_op.clone(),
                        })?;
                    }
                    _ => return Err("SRLI: Invalid destination operand".to_string()),
                }
            }

            let shift_amount = Operand::Immediate(sh.shamt as i64);
            match rd_op {
                Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                    translator.emit_instruction(&X86Instruction::Shr {
                        src: shift_amount,
                        dst: r11.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: r11,
                        dst: rd_op,
                    })?;
                }
                Operand::Register(_) => {
                    translator.emit_instruction(&X86Instruction::Shr {
                        src: shift_amount,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("SRLI: Invalid destination operand".to_string()),
            }
        }

        // SRAI rd, rs1, imm → X[rd] = X[rs1] >> imm (arithmetic)
        Srai(sh) => {
            let rd_loc = mapping.get_register_location(sh.rd);
            let rs1_loc = mapping.get_register_location(sh.rs1);
            let rd_op = mapping.location_to_operand(rd_loc)?;
            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let r11 = Operand::Register(X86Register::R11);

            if sh.rd != sh.rs1 {
                match rd_op {
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: r11.clone(),
                        })?;
                    }
                    Operand::Register(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: rd_op.clone(),
                        })?;
                    }
                    _ => return Err("SRAI: Invalid destination operand".to_string()),
                }
            }

            let shift_amount = Operand::Immediate(sh.shamt as i64);
            match rd_op {
                Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                    translator.emit_instruction(&X86Instruction::Sar {
                        src: shift_amount,
                        dst: r11.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: r11,
                        dst: rd_op,
                    })?;
                }
                Operand::Register(_) => {
                    translator.emit_instruction(&X86Instruction::Sar {
                        src: shift_amount,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("SRAI: Invalid destination operand".to_string()),
            }
        }

        // SLL rd, rs1, rs2 → X[rd] = X[rs1] << X[rs2][5:0]
        // In x86-64, the shift amount must be in CL or constant. For register shifts,
        // we need to move rs2 to RCX, shift, then restore RCX if needed
        Sll(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);
            let rd_op = mapping.location_to_operand(rd_loc)?;
            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rs2_op = mapping.location_to_operand(rs2_loc)?;

            let rcx = Operand::Register(X86Register::RCX);
            let r11 = Operand::Register(X86Register::R11);

            // Store rcx value
            translator.emit_instruction(&X86Instruction::Push { src: rcx.clone() })?;

            // Move shift amount to RCX (required for shift operations)
            translator.emit_instruction(&X86Instruction::Mov {
                src: rs2_op,
                dst: rcx.clone(),
            })?;

            // Handle shift based on rd location
            if r.rd != r.rs1 {
                // Different destination and source
                match rd_op {
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                        // Load rs1 into temp, shift, store to memory
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: r11.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Shl {
                            src: rcx.clone(),
                            dst: r11.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: r11,
                            dst: rd_op,
                        })?;
                    }
                    Operand::Register(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: rd_op.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Shl {
                            src: rcx.clone(),
                            dst: rd_op,
                        })?;
                    }
                    _ => return Err("SLL: Invalid destination operand".to_string()),
                }
            } else {
                // Same destination and source
                match rd_op {
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rd_op.clone(),
                            dst: r11.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Shl {
                            src: rcx.clone(),
                            dst: r11.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: r11,
                            dst: rd_op,
                        })?;
                    }
                    Operand::Register(_) => {
                        translator.emit_instruction(&X86Instruction::Shl {
                            src: rcx.clone(),
                            dst: rd_op,
                        })?;
                    }
                    _ => return Err("SLL: Invalid destination operand".to_string()),
                }
            }

            // Restore rcx value
            translator.emit_instruction(&X86Instruction::Pop { dst: rcx })?;
        }

        // SRL rd, rs1, rs2 → X[rd] = X[rs1] >> X[rs2][5:0] (logical)
        Srl(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);
            let rd_op = mapping.location_to_operand(rd_loc)?;
            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rs2_op = mapping.location_to_operand(rs2_loc)?;

            let rcx = Operand::Register(X86Register::RCX);
            let r11 = Operand::Register(X86Register::R11);

            // Store rcx
            translator.emit_instruction(&X86Instruction::Push { src: rcx.clone() })?;

            translator.emit_instruction(&X86Instruction::Mov {
                src: rs2_op,
                dst: rcx.clone(),
            })?;

            if r.rd != r.rs1 {
                match rd_op {
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: r11.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Shr {
                            src: rcx.clone(),
                            dst: r11.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: r11,
                            dst: rd_op,
                        })?;
                    }
                    Operand::Register(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: rd_op.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Shr {
                            src: rcx.clone(),
                            dst: rd_op,
                        })?;
                    }
                    _ => return Err("SRL: Invalid destination operand".to_string()),
                }
            } else {
                match rd_op {
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rd_op.clone(),
                            dst: r11.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Shr {
                            src: rcx.clone(),
                            dst: r11.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: r11,
                            dst: rd_op,
                        })?;
                    }
                    Operand::Register(_) => {
                        translator.emit_instruction(&X86Instruction::Shr {
                            src: rcx.clone(),
                            dst: rd_op,
                        })?;
                    }
                    _ => return Err("SRL: Invalid destination operand".to_string()),
                }
            }

            // Resstore rcx
            translator.emit_instruction(&X86Instruction::Pop { dst: rcx })?;
        }

        // SRA rd, rs1, rs2 → X[rd] = X[rs1] >> X[rs2][5:0] (arithmetic)
        Sra(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);
            let rd_op = mapping.location_to_operand(rd_loc)?;
            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rs2_op = mapping.location_to_operand(rs2_loc)?;

            let rcx = Operand::Register(X86Register::RCX);
            let r11 = Operand::Register(X86Register::R11);

            // Store rcx
            translator.emit_instruction(&X86Instruction::Push { src: rcx.clone() })?;

            translator.emit_instruction(&X86Instruction::Mov {
                src: rs2_op,
                dst: rcx.clone(),
            })?;

            if r.rd != r.rs1 {
                match rd_op {
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: r11.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Sar {
                            src: rcx.clone(),
                            dst: r11.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: r11,
                            dst: rd_op,
                        })?;
                    }
                    Operand::Register(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op,
                            dst: rd_op.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Sar {
                            src: rcx.clone(),
                            dst: rd_op,
                        })?;
                    }
                    _ => return Err("SRA: Invalid destination operand".to_string()),
                }
            } else {
                match rd_op {
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rd_op.clone(),
                            dst: r11.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Sar {
                            src: rcx.clone(),
                            dst: r11.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: r11,
                            dst: rd_op,
                        })?;
                    }
                    Operand::Register(_) => {
                        translator.emit_instruction(&X86Instruction::Sar {
                            src: rcx.clone(),
                            dst: rd_op,
                        })?;
                    }
                    _ => return Err("SRA: Invalid destination operand".to_string()),
                }
            }

            // Restore rcx
            translator.emit_instruction(&X86Instruction::Pop { dst: rcx })?;
        }

        // === Control Flow Instructions ===
        // JALR rd, imm(rs1) → X[rd] = PC + 4; PC = (X[rs1] + imm) & ~1
        Jalr(i) => {
            let rd_loc = mapping.get_register_location(i.rd);
            let rs1_loc = mapping.get_register_location(i.rs1);

            // Get operands
            let rd_op = mapping.location_to_operand(rd_loc)?;
            let rs1_op = mapping.location_to_operand(rs1_loc)?;

            // If rd == rs1, we need to save rs1 before overwriting it!
            // Determine target_reg BEFORE we clobber rd
            let target_reg = if rd_loc == rs1_loc {
                // rd and rs1 are the same register - we need a temp
                // Load rs1 into R12 BEFORE we overwrite it with the return address
                translator.emit_instruction(&X86Instruction::Mov {
                    src: rs1_op.clone(),
                    dst: Operand::Register(X86Register::R12),
                })?;
                Operand::Register(X86Register::R12)
            } else {
                // rd and rs1 are different, safe to overwrite rd
                match rs1_op {
                    Operand::Register(_) => rs1_op.clone(),
                    _ => {
                        // Load rs1 from memory into R12
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: rs1_op.clone(),
                            dst: Operand::Register(X86Register::R12),
                        })?;
                        Operand::Register(X86Register::R12)
                    }
                }
            };

            // Now save return address AFTER we've preserved rs1
            let next_riscv_pc = translator.context.pc + 4;
            translator.emit_instruction(&X86Instruction::Mov {
                src: Operand::Immediate(next_riscv_pc as i64),
                dst: rd_op,
            })?;

            // Now compute the jump target:
            // 1. Add immediate offset to target_reg (rs1 value or copy of it)
            // 2. Clear LSB (&~1)
            // 3. Look up in PC mapping table
            // 4. Jump to x86 address

            // Step 1: Add immediate offset
            if i.imm != 0 {
                translator.emit_instruction(&X86Instruction::Add {
                    src: Operand::Immediate(i.imm as i64),
                    dst: target_reg.clone(),
                })?;
            }

            // Step 2: Clear LSB (&~1)
            translator.emit_instruction(&X86Instruction::And {
                src: Operand::Immediate(-2i64),
                dst: target_reg.clone(),
            })?;

            // Now target_reg contains the RISC-V PC to jump to
            // Use PC mapping table to look it up
            let riscv_entry_pc = translator.context.entry_point;

            // Subtract entry point from PC
            translator.emit_instruction(&X86Instruction::Sub {
                src: Operand::Immediate(riscv_entry_pc as i64),
                dst: target_reg.clone(),
            })?;

            // Divide by 4 to get PC index, then multiply by 8 to get table offset
            translator.emit_instruction(&X86Instruction::Shr {
                src: Operand::Immediate(2),
                dst: target_reg.clone(),
            })?;

            translator.emit_instruction(&X86Instruction::Shl {
                src: Operand::Immediate(3),
                dst: target_reg.clone(),
            })?;

            // Add PC mapping table base address
            translator.emit_instruction(&X86Instruction::Add {
                src: Operand::Immediate(0x502000i64),
                dst: target_reg.clone(),
            })?;

            // Load the x86 offset from the PC mapping table
            // Use R11 as temp address register
            translator.emit_instruction(&X86Instruction::Mov {
                src: target_reg.clone(),
                dst: Operand::Register(X86Register::R11),
            })?;

            // // Read x86 offset from table
            translator.emit_instruction(&X86Instruction::Mov {
                src: Operand::Memory {
                    base: X86Register::R11,
                    offset: 0,
                },
                dst: Operand::Register(X86Register::R13),
            })?;

            // Add code base address (0x400000) to get absolute x86 address
            translator.emit_instruction(&X86Instruction::Add {
                src: Operand::Immediate(0x400000i64),
                dst: Operand::Register(X86Register::R13),
            })?;

            // Jump to computed address
            translator.emit_instruction(&X86Instruction::JmpReg {
                target: Operand::Register(X86Register::R13),
            })?;
        }

        // JAL rd, imm → X[rd] = PC + 4; PC = PC + imm
        // Unconditional jump with link: save return address and jump to offset
        Jal(j) => {
            let rd_loc = mapping.get_register_location(j.rd);
            let rd_op = mapping.location_to_operand(rd_loc)?;

            // Save return address: rd = PC + 4 (RISC-V PC value)
            let riscv_pc = translator.context().pc;
            let next_riscv_pc = riscv_pc + 4;

            translator.emit_instruction(&X86Instruction::Mov {
                src: Operand::Immediate(next_riscv_pc as i64),
                dst: rd_op,
            })?;

            // Calculate target PC = PC + imm (also a RISC-V PC)
            let target_pc = riscv_pc + (j.imm as u64);

            // Use PC mapping table to jump to the target
            // Load target RISC-V PC into R11 for lookup (R11 is our address register)
            let riscv_entry_pc = translator.context.entry_point;

            translator.emit_instruction(&X86Instruction::Mov {
                src: Operand::Immediate(target_pc as i64),
                dst: Operand::Register(X86Register::R11),
            })?;

            // Subtract entry point to get offset from start
            translator.emit_instruction(&X86Instruction::Sub {
                src: Operand::Immediate(riscv_entry_pc as i64),
                dst: Operand::Register(X86Register::R11),
            })?;

            // Divide by 4 to get PC index, then multiply by 8 to get table offset
            translator.emit_instruction(&X86Instruction::Shr {
                src: Operand::Immediate(2),
                dst: Operand::Register(X86Register::R11),
            })?;

            translator.emit_instruction(&X86Instruction::Shl {
                src: Operand::Immediate(3),
                dst: Operand::Register(X86Register::R11),
            })?;

            // Add PC mapping table base address
            translator.emit_instruction(&X86Instruction::Add {
                src: Operand::Immediate(0x502000i64),
                dst: Operand::Register(X86Register::R11),
            })?;

            // Load the x86 offset from the PC mapping table
            translator.emit_instruction(&X86Instruction::Mov {
                src: Operand::Memory {
                    base: X86Register::R11,
                    offset: 0,
                },
                dst: Operand::Register(X86Register::R12),
            })?;

            // Add code base address (0x400000) to get absolute x86 address
            translator.emit_instruction(&X86Instruction::Add {
                src: Operand::Immediate(0x400000i64),
                dst: Operand::Register(X86Register::R12),
            })?;

            // Jump to computed address
            translator.emit_instruction(&X86Instruction::JmpReg {
                target: Operand::Register(X86Register::R12),
            })?;
        }

        // === Comparison Operations ===
        // Branch instructions work with the riscv pc
        // so we generate labels for the pc and do symbol relocation during linking
        // BEQ - Branch if Equal
        // if rs1 == rs2: PC = PC + imm
        Beq(b) => {
            let rs1_loc = mapping.get_register_location(b.rs1);
            let rs2_loc = mapping.get_register_location(b.rs2);

            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rs2_op = mapping.location_to_operand(rs2_loc)?;

            let r12 = Operand::Register(X86Register::R12);
            let r13 = Operand::Register(X86Register::R13);

            // Handle memory operands: x86-64 CMP doesn't support mem-mem comparisons
            let (cmp_src, cmp_dst) = match (&rs1_op, &rs2_op) {
                (
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                ) => {
                    // Both in memory: load rs1 into R12, rs2 into R13
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r13.clone(),
                    })?;
                    (r13, r12)
                }
                (Operand::Memory { .. } | Operand::AbsoluteAddress(_), _) => {
                    // rs1 in memory: load into R12
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (rs2_op, r12)
                }
                (_, Operand::Memory { .. } | Operand::AbsoluteAddress(_)) => {
                    // rs2 in memory: load into r12
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (r12, rs1_op)
                }
                _ => (rs2_op, rs1_op),
            };

            // Compare rs1 with rs2
            translator.emit_instruction(&X86Instruction::Cmp {
                src: cmp_src,
                dst: cmp_dst,
            })?;

            // Calculate target PC
            let target_pc = translator.context().pc as i64 + b.imm as i64;
            let label = format!("L_{:x}", target_pc);

            // Emit conditional jump: je label
            translator.emit_instruction(&X86Instruction::Je { target: label })?;
        }

        // BNE - Branch if Not Equal
        // if rs1 != rs2: PC = PC + imm
        Bne(b) => {
            let rs1_loc = mapping.get_register_location(b.rs1);
            let rs2_loc = mapping.get_register_location(b.rs2);

            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rs2_op = mapping.location_to_operand(rs2_loc)?;

            let r12 = Operand::Register(X86Register::R12);
            let r13 = Operand::Register(X86Register::R13);

            // Handle memory operands: x86-64 CMP doesn't support mem-mem comparisons
            let (cmp_src, cmp_dst) = match (&rs1_op, &rs2_op) {
                (
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                ) => {
                    // Both in memory: load rs1 into R12, rs2 into R13
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r13.clone(),
                    })?;
                    (r13, r12)
                }
                (Operand::Memory { .. } | Operand::AbsoluteAddress(_), _) => {
                    // rs1 in memory: load into R12
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (rs2_op, r12)
                }
                (_, Operand::Memory { .. } | Operand::AbsoluteAddress(_)) => {
                    // rs2 in memory: load into R12
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (r12, rs1_op)
                }
                _ => (rs2_op, rs1_op),
            };

            // Compare rs1 with rs2
            translator.emit_instruction(&X86Instruction::Cmp {
                src: cmp_src,
                dst: cmp_dst,
            })?;

            // Calculate target PC
            let target_pc = translator.context().pc as i64 + b.imm as i64;
            let label = format!("L_{:x}", target_pc);

            // Emit conditional jump: jne label
            translator.emit_instruction(&X86Instruction::Jne { target: label })?;
        }

        // BLT - Branch if Less Than (signed)
        // if rs1 < rs2 (signed): PC = PC + imm
        Blt(b) => {
            let rs1_loc = mapping.get_register_location(b.rs1);
            let rs2_loc = mapping.get_register_location(b.rs2);

            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rs2_op = mapping.location_to_operand(rs2_loc)?;

            let r12 = Operand::Register(X86Register::R12);
            let r13 = Operand::Register(X86Register::R13);

            // Handle memory operands: x86-64 CMP doesn't support mem-mem comparisons
            let (cmp_src, cmp_dst) = match (&rs1_op, &rs2_op) {
                (
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                ) => {
                    // Both in memory: load rs1 into R12, rs2 into R13
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r13.clone(),
                    })?;
                    (r13, r12)
                }
                (Operand::Memory { .. } | Operand::AbsoluteAddress(_), _) => {
                    // rs1 in memory: load into R12
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (rs2_op, r12)
                }
                (_, Operand::Memory { .. } | Operand::AbsoluteAddress(_)) => {
                    // rs2 in memory: load into R12
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (r12, rs1_op)
                }
                _ => (rs2_op, rs1_op),
            };

            // Compare rs1 with rs2
            translator.emit_instruction(&X86Instruction::Cmp {
                src: cmp_src,
                dst: cmp_dst,
            })?;

            // Calculate target PC
            let target_pc = translator.context().pc as i64 + b.imm as i64;
            let label = format!("L_{:x}", target_pc);

            // Emit conditional jump: jl label (signed less)
            translator.emit_instruction(&X86Instruction::Jl { target: label })?;
        }

        // BGE - Branch if Greater or Equal (signed)
        // if rs1 >= rs2 (signed): PC = PC + imm
        Bge(b) => {
            let rs1_loc = mapping.get_register_location(b.rs1);
            let rs2_loc = mapping.get_register_location(b.rs2);

            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rs2_op = mapping.location_to_operand(rs2_loc)?;

            let r12 = Operand::Register(X86Register::R12);
            let r13 = Operand::Register(X86Register::R13);

            // Handle memory operands: x86-64 CMP doesn't support mem-mem comparisons
            let (cmp_src, cmp_dst) = match (&rs1_op, &rs2_op) {
                (
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                ) => {
                    // Both in memory: load rs1 into R12, rs2 into R13
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r13.clone(),
                    })?;
                    (r13, r12)
                }
                (Operand::Memory { .. } | Operand::AbsoluteAddress(_), _) => {
                    // rs1 in memory: load into R12
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (rs2_op, r12)
                }
                (_, Operand::Memory { .. } | Operand::AbsoluteAddress(_)) => {
                    // rs2 in memory: load into R12
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (r12, rs1_op)
                }
                _ => (rs2_op, rs1_op),
            };

            // Compare rs1 with rs2
            translator.emit_instruction(&X86Instruction::Cmp {
                src: cmp_src,
                dst: cmp_dst,
            })?;

            // Calculate target PC
            let target_pc = translator.context().pc as i64 + b.imm as i64;
            let label = format!("L_{:x}", target_pc);

            // Emit conditional jump: jge label (signed greater or equal)
            translator.emit_instruction(&X86Instruction::Jge { target: label })?;
        }

        // BLTU - Branch if Less Than Unsigned
        // if rs1 < rs2 (unsigned): PC = PC + imm
        Bltu(b) => {
            let rs1_loc = mapping.get_register_location(b.rs1);
            let rs2_loc = mapping.get_register_location(b.rs2);

            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rs2_op = mapping.location_to_operand(rs2_loc)?;

            let r12 = Operand::Register(X86Register::R12);
            let r13 = Operand::Register(X86Register::R13);

            // Handle memory operands: x86-64 CMP doesn't support mem-mem comparisons
            let (cmp_src, cmp_dst) = match (&rs1_op, &rs2_op) {
                (
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                ) => {
                    // Both in memory: load rs1 into R12, rs2 into r13
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r13.clone(),
                    })?;
                    (r13, r12)
                }
                (Operand::Memory { .. } | Operand::AbsoluteAddress(_), _) => {
                    // rs1 in memory: load into R12
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (rs2_op, r12)
                }
                (_, Operand::Memory { .. } | Operand::AbsoluteAddress(_)) => {
                    // rs2 in memory: load into R12
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (r12, rs1_op)
                }
                _ => (rs2_op, rs1_op),
            };

            // Compare rs1 with rs2
            translator.emit_instruction(&X86Instruction::Cmp {
                src: cmp_src,
                dst: cmp_dst,
            })?;

            // Calculate target PC
            let target_pc = translator.context().pc as i64 + b.imm as i64;
            let label = format!("L_{:x}", target_pc);

            // Emit conditional jump: jb label (unsigned below)
            translator.emit_instruction(&X86Instruction::Jb { target: label })?;
        }

        // BGEU - Branch if Greater or Equal Unsigned
        // if rs1 >= rs2 (unsigned): PC = PC + imm
        Bgeu(b) => {
            let rs1_loc = mapping.get_register_location(b.rs1);
            let rs2_loc = mapping.get_register_location(b.rs2);

            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rs2_op = mapping.location_to_operand(rs2_loc)?;

            let r12 = Operand::Register(X86Register::R12);
            let r13 = Operand::Register(X86Register::R13);

            // Handle memory operands: x86-64 CMP doesn't support mem-mem comparisons
            let (cmp_src, cmp_dst) = match (&rs1_op, &rs2_op) {
                (
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                ) => {
                    // Both in memory: load rs1 into R12, rs2 into R13
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r13.clone(),
                    })?;
                    (r13, r12)
                }
                (Operand::Memory { .. } | Operand::AbsoluteAddress(_), _) => {
                    // rs1 in memory: load into R12
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (rs2_op, r12)
                }
                (_, Operand::Memory { .. } | Operand::AbsoluteAddress(_)) => {
                    // rs2 in memory: load into R12
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (r12, rs1_op)
                }
                _ => (rs2_op, rs1_op),
            };

            // Compare rs1 with rs2
            translator.emit_instruction(&X86Instruction::Cmp {
                src: cmp_src,
                dst: cmp_dst,
            })?;

            // Calculate target PC
            let target_pc = translator.context().pc as i64 + b.imm as i64;
            let label = format!("L_{:x}", target_pc);

            // Emit conditional jump: jae label (unsigned above or equal)
            translator.emit_instruction(&X86Instruction::Jae { target: label })?;
        }

        // SLT rd, rs1, rs2 → X[rd] = (X[rs1] < X[rs2]) ? 1 : 0 (signed comparison)
        Slt(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);

            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rs2_op = mapping.location_to_operand(rs2_loc)?;
            let rd_op = mapping.location_to_operand(rd_loc)?;

            // Determine which registers to use for comparison
            let (cmp_src, cmp_dst) = match (&rs1_op, &rs2_op) {
                (
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                ) => {
                    // Both in memory: load both into temp registers
                    let r12 = Operand::Register(X86Register::R12);
                    let r13 = Operand::Register(X86Register::R13);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r13.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (r12, r13)
                }
                (Operand::Memory { .. } | Operand::AbsoluteAddress(_), _) => {
                    // rs1 in memory: load into R12
                    let r12 = Operand::Register(X86Register::R12);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (rs2_op, r12)
                }
                (_, Operand::Memory { .. } | Operand::AbsoluteAddress(_)) => {
                    // rs2 in memory: load into R12
                    let r12 = Operand::Register(X86Register::R12);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (r12, rs1_op)
                }
                _ => (rs2_op, rs1_op),
            };

            // Compare rs1 with rs2
            translator.emit_instruction(&X86Instruction::Cmp {
                src: cmp_src,
                dst: cmp_dst,
            })?;

            // Set AL to 1 if rs1 < rs2 (signed)
            let al = Operand::Register(X86Register::RAX);
            translator.emit_instruction(&X86Instruction::Setl { dst: al.clone() })?;

            // Zero-extend AL to RAX
            translator.emit_instruction(&X86Instruction::Movzx {
                src: Operand::Register(X86Register::RAX), // This will only use AL
                dst: al.clone(),
            })?;

            // Move result to destination
            match rd_op {
                Operand::Register(_) => {
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: al,
                        dst: rd_op,
                    })?;
                }
                Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                    // Need temp register for memory destination
                    let r12 = Operand::Register(X86Register::R12);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: al,
                        dst: r12.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: r12,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("SLT: invalid destination operand".to_string()),
            }
        }

        // SLTU rd, rs1, rs2 → X[rd] = (X[rs1] < X[rs2]) ? 1 : 0 (unsigned comparison)
        Sltu(r) => {
            let rd_loc = mapping.get_register_location(r.rd);
            let rs1_loc = mapping.get_register_location(r.rs1);
            let rs2_loc = mapping.get_register_location(r.rs2);

            let rs1_op = mapping.location_to_operand(rs1_loc)?;
            let rs2_op = mapping.location_to_operand(rs2_loc)?;
            let rd_op = mapping.location_to_operand(rd_loc)?;

            // Determine which registers to use for comparison
            let (cmp_src, cmp_dst) = match (&rs1_op, &rs2_op) {
                (
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                    Operand::Memory { .. } | Operand::AbsoluteAddress(_),
                ) => {
                    // Both in memory: load both into temp registers
                    let r12 = Operand::Register(X86Register::R12);
                    let r13 = Operand::Register(X86Register::R13);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r13.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (r12, r13)
                }
                (Operand::Memory { .. } | Operand::AbsoluteAddress(_), _) => {
                    // rs1 in memory: load into R12
                    let r12 = Operand::Register(X86Register::R12);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs1_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (rs2_op, r12)
                }
                (_, Operand::Memory { .. } | Operand::AbsoluteAddress(_)) => {
                    // rs2 in memory: load into R12
                    let r12 = Operand::Register(X86Register::R12);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: rs2_op.clone(),
                        dst: r12.clone(),
                    })?;
                    (r12, rs1_op)
                }
                _ => (rs2_op, rs1_op),
            };

            // Compare rs1 with rs2
            translator.emit_instruction(&X86Instruction::Cmp {
                src: cmp_src,
                dst: cmp_dst,
            })?;

            // Set AL to 1 if rs1 < rs2 (unsigned)
            let al = Operand::Register(X86Register::RAX);
            translator.emit_instruction(&X86Instruction::Setb { dst: al.clone() })?;

            // Zero-extend AL to RAX
            translator.emit_instruction(&X86Instruction::Movzx {
                src: Operand::Register(X86Register::RAX), // This will only use AL
                dst: al.clone(),
            })?;

            // Move result to destination
            match rd_op {
                Operand::Register(_) => {
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: al,
                        dst: rd_op,
                    })?;
                }
                Operand::Memory { .. } | Operand::AbsoluteAddress(_) => {
                    // Need temp register for memory destination
                    let r12 = Operand::Register(X86Register::R12);
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: al,
                        dst: r12.clone(),
                    })?;
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: r12,
                        dst: rd_op,
                    })?;
                }
                _ => return Err("SLTU: invalid destination operand".to_string()),
            }
        }

        Slti(_i) => return Err("SLTI: Refactoring in progress".to_string()),
        Sltiu(_i) => return Err("SLTIU: Refactoring in progress".to_string()),

        // === Control and Status Registers (Zicsr) ===
        // For AOT translation, CSR state is inaccessible and can't be accurately modeled.
        // We emit NOP for all CSR instructions to preserve program flow without corrupting
        // register state. The actual CSR values and side effects are lost, but this allows
        // the translated code to proceed without crashes.

        // CSRRW - Control and Status Register Read/Write
        Csrrw(_i) => {
            translator.emit_instruction(&X86Instruction::Nop)?;
        }

        // CSRRS - Control and Status Register Read and Set Bits
        Csrrs(_i) => {
            translator.emit_instruction(&X86Instruction::Nop)?;
        }

        // CSRRC - Control and Status Register Read and Clear Bits
        Csrrc(_i) => {
            translator.emit_instruction(&X86Instruction::Nop)?;
        }

        // CSRRWI - Control and Status Register Read/Write Immediate
        Csrrwi(_i) => {
            translator.emit_instruction(&X86Instruction::Nop)?;
        }

        // CSRRSI - Control and Status Register Read and Set Bits Immediate
        Csrrsi(_i) => {
            translator.emit_instruction(&X86Instruction::Nop)?;
        }

        // CSRRCI - Control and Status Register Read and Clear Bits Immediate
        Csrrci(_i) => {
            translator.emit_instruction(&X86Instruction::Nop)?;
        }

        // === System Calls ===
        // ECALL - Environment call (system call)
        // RISC-V calling convention:
        //   a7 (x17) = syscall number
        //   a0-a5 (x10-x15) = arguments
        // x86-64 syscall convention:
        //   RAX = syscall number
        //   RDI, RSI, RDX, R10, R8, R9 = arguments (in order)
        // Note: x86-64 syscall clobbers RCX, R11, and caller-saved registers.
        // We need to preserve caller-saved registers and RAX before the syscall.
        Ecall => {
            // Get all register locations and operands before mutable borrow
            let syscall_num_loc = mapping.get_register_location(17);
            let a0_loc = mapping.get_register_location(10);
            let a1_loc = mapping.get_register_location(11);
            let a2_loc = mapping.get_register_location(12);
            let a3_loc = mapping.get_register_location(13);
            let a4_loc = mapping.get_register_location(14);
            let a5_loc = mapping.get_register_location(15);

            let syscall_num_op = mapping.location_to_operand(syscall_num_loc)?;
            let a0_op = mapping.location_to_operand(a0_loc)?;
            let a1_op = mapping.location_to_operand(a1_loc)?;
            let a2_op = mapping.location_to_operand(a2_loc)?;
            let a3_op = mapping.location_to_operand(a3_loc)?;
            let a4_op = mapping.location_to_operand(a4_loc)?;
            let a5_op = mapping.location_to_operand(a5_loc)?;

            // Helper to push an operand that might be in memory
            // PUSH instruction only supports register operands, so if operand is in memory,
            // we need to load it into a temporary register (R12) first
            let push_operand = |translator: &mut RiscvToX86Translator<M>,
                                op: &Operand|
             -> Result<(), String> {
                match op {
                    Operand::Register(_) => {
                        translator.emit_instruction(&X86Instruction::Push { src: op.clone() })?;
                        Ok(())
                    }
                    Operand::AbsoluteAddress(_) => {
                        // Load from memory into R12, then push R12
                        let r12 = Operand::Register(X86Register::R12);
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: op.clone(),
                            dst: r12.clone(),
                        })?;
                        translator.emit_instruction(&X86Instruction::Push { src: r12 })?;
                        Ok(())
                    }
                    _ => Err(format!("Cannot push operand: {}", op)),
                }
            };

            // We need to save:
            // - a1-a5: the argument registers
            // - RAX: contains x1(ra), will be overwritten with syscall number
            // Push order: a5, a4, a3, a2, a1, RAX
            // a0(rdi) is not saved because its value will be clobbered(the syscall return value will be written there)
            push_operand(translator, &a5_op)?;
            push_operand(translator, &a4_op)?;
            push_operand(translator, &a3_op)?;
            push_operand(translator, &a2_op)?;
            push_operand(translator, &a1_op)?;
            translator.emit_instruction(&X86Instruction::Push {
                src: Operand::Register(X86Register::RAX),
            })?; // Save RAX (contains x1/ra)

            // Move syscall number (x17) to RAX first (before mapping)
            translator.emit_instruction(&X86Instruction::Mov {
                src: syscall_num_op.clone(),
                dst: Operand::Register(X86Register::RAX),
            })?;

            // Move arguments to their x86-64 syscall positions
            // a0 (x10) -> RDI
            if a0_op != Operand::Register(X86Register::RDI) {
                translator.emit_instruction(&X86Instruction::Mov {
                    src: a0_op.clone(),
                    dst: Operand::Register(X86Register::RDI),
                })?;
            }

            // a1 (x11) -> RSI
            if a1_op != Operand::Register(X86Register::RSI) {
                translator.emit_instruction(&X86Instruction::Mov {
                    src: a1_op.clone(),
                    dst: Operand::Register(X86Register::RSI),
                })?;
            }

            // a2 (x12) -> RDX
            if a2_op != Operand::Register(X86Register::RDX) {
                translator.emit_instruction(&X86Instruction::Mov {
                    src: a2_op.clone(),
                    dst: Operand::Register(X86Register::RDX),
                })?;
            }

            // a3 (x13) -> R10
            if a3_op != Operand::Register(X86Register::R10) {
                translator.emit_instruction(&X86Instruction::Mov {
                    src: a3_op.clone(),
                    dst: Operand::Register(X86Register::R10),
                })?;
            }

            // a4 (x14) -> R8
            if a4_op != Operand::Register(X86Register::R8) {
                translator.emit_instruction(&X86Instruction::Mov {
                    src: a4_op.clone(),
                    dst: Operand::Register(X86Register::R8),
                })?;
            }

            // a5 (x15) -> R9
            if a5_op != Operand::Register(X86Register::R9) {
                translator.emit_instruction(&X86Instruction::Mov {
                    src: a5_op.clone(),
                    dst: Operand::Register(X86Register::R9),
                })?;
            }

            // Map RISC-V syscall numbers to x86-64 syscall numbers using data segment lookup table
            // Syscall mapping table is stored in the data segment at address 0x501000
            // (0x500000 data_base + 0x1000 syscall table offset)
            // Table entries: [0]=invalid, [63]=0 (read), [64]=1 (write), [93]=60 (exit)

            // RAX contains the RISC-V syscall number
            // Mask RAX to get only the low byte (syscall numbers are < 256)
            translator.emit_instruction(&X86Instruction::And {
                src: Operand::Immediate(0xFF),
                dst: Operand::Register(X86Register::RAX),
            })?;

            // Load base address of syscall mapping table into R11
            translator.emit_instruction(&X86Instruction::Mov {
                src: Operand::Immediate(0x501000i64),
                dst: Operand::Register(X86Register::R11),
            })?;

            // Add RAX (syscall number) to R11 to get the address of the mapped syscall
            translator.emit_instruction(&X86Instruction::Add {
                src: Operand::Register(X86Register::RAX),
                dst: Operand::Register(X86Register::R11),
            })?;

            // Load the mapped syscall number from [R11]
            // Use movzx to zero-extend the byte value to 64-bit
            translator.emit_instruction(&X86Instruction::Movzx {
                src: Operand::Memory {
                    base: X86Register::R11,
                    offset: 0,
                },
                dst: Operand::Register(X86Register::RAX),
            })?;

            // Emit the syscall instruction with the mapped syscall number in RAX
            translator.emit_instruction(&X86Instruction::Syscall)?;

            // After syscall, RAX contains the return value
            // Store it in a0 (x10) - the return value location
            match a0_op {
                Operand::Register(X86Register::RDI) => {
                    // a0 is in RDI, but RAX has the return value, so move RAX to RDI
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::Register(X86Register::RAX),
                        dst: Operand::Register(X86Register::RDI),
                    })?;
                }
                _ => {
                    // a0 is somewhere else (register or memory), move RAX there
                    translator.emit_instruction(&X86Instruction::Mov {
                        src: Operand::Register(X86Register::RAX),
                        dst: a0_op.clone(),
                    })?;
                }
            }

            // Restore a1-a5 from stack (pop in reverse order: a1, a2, a3, a4, a5)
            // Helper to pop into an operand that might be in memory
            let pop_operand = |translator: &mut RiscvToX86Translator<M>,
                               op: &Operand|
             -> Result<(), String> {
                match op {
                    Operand::Register(_) => {
                        translator.emit_instruction(&X86Instruction::Pop { dst: op.clone() })?;
                        Ok(())
                    }
                    Operand::AbsoluteAddress(_) => {
                        // Pop from stack into R11, then store R11 to memory
                        let r11 = Operand::Register(X86Register::R11);
                        translator.emit_instruction(&X86Instruction::Pop { dst: r11.clone() })?;
                        translator.emit_instruction(&X86Instruction::Mov {
                            src: r11,
                            dst: op.clone(),
                        })?;
                        Ok(())
                    }
                    _ => Err(format!("Cannot pop into operand: {}", op)),
                }
            };

            // Restore from stack (pop in reverse order: RAX, a1, a2, a3, a4, a5)
            // First restore RAX (contains x1/ra)
            translator.emit_instruction(&X86Instruction::Pop {
                dst: Operand::Register(X86Register::RAX),
            })?;

            // Restore a1-a5 from stack (pop in reverse order: a1, a2, a3, a4, a5)
            pop_operand(translator, &a1_op)?;
            pop_operand(translator, &a2_op)?;
            pop_operand(translator, &a3_op)?;
            pop_operand(translator, &a4_op)?;
            pop_operand(translator, &a5_op)?;
        }

        _ => {
            return Err(format!("Instruction not yet translated: {:?}", riscv_insn));
        }
    }

    Ok(())
}
