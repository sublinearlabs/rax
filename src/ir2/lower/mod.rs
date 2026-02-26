pub mod i;

use crate::decode::Instruction;
use crate::ir2::IrBuilder;
use crate::ir2::lower::i::lower_i_into;

pub fn lower_instruction_into(
    insn: &Instruction,
    current_pc: u64,
    next_pc: u64,
    builder: &mut IrBuilder,
) {
    match insn {
        Instruction::Illegal(_) => {
            builder.halt(1);
            builder.ret();
        }
        Instruction::Add(_)
        | Instruction::Sub(_)
        | Instruction::Sll(_)
        | Instruction::Slt(_)
        | Instruction::Sltu(_)
        | Instruction::Xor(_)
        | Instruction::Srl(_)
        | Instruction::Sra(_)
        | Instruction::Or(_)
        | Instruction::And(_)
        | Instruction::Addi(_)
        | Instruction::Slti(_)
        | Instruction::Sltiu(_)
        | Instruction::Xori(_)
        | Instruction::Ori(_)
        | Instruction::Andi(_)
        | Instruction::Slli(_)
        | Instruction::Srli(_)
        | Instruction::Srai(_)
        | Instruction::Lb(_)
        | Instruction::Lh(_)
        | Instruction::Lw(_)
        | Instruction::Lbu(_)
        | Instruction::Lhu(_)
        | Instruction::Sb(_)
        | Instruction::Sh(_)
        | Instruction::Sw(_)
        | Instruction::Beq(_)
        | Instruction::Bne(_)
        | Instruction::Blt(_)
        | Instruction::Bge(_)
        | Instruction::Bltu(_)
        | Instruction::Bgeu(_)
        | Instruction::Jal(_)
        | Instruction::Jalr(_)
        | Instruction::Lui(_)
        | Instruction::Auipc(_)
        | Instruction::Addiw(_)
        | Instruction::Slliw(_)
        | Instruction::Srliw(_)
        | Instruction::Sraiw(_)
        | Instruction::Addw(_)
        | Instruction::Subw(_)
        | Instruction::Sllw(_)
        | Instruction::Srlw(_)
        | Instruction::Sraw(_)
        | Instruction::Ld(_)
        | Instruction::Lwu(_)
        | Instruction::Sd(_)
        | Instruction::Nop
        | Instruction::Ecall
        | Instruction::Ebreak => lower_i_into(insn, current_pc, next_pc, builder),
        _ => panic!("no lowering found for {:?}", insn),
    }
}
