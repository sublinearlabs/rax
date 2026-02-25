#[cfg(feature = "ext_a")]
pub mod a;
pub mod csr;
pub mod i;
#[cfg(feature = "ext_m")]
pub mod m;

use crate::decode::Instruction;
#[cfg(feature = "ext_a")]
use crate::ir::lower::a::lower_a_into;
use crate::ir::lower::i::lower_i_into;
#[cfg(feature = "ext_m")]
use crate::ir::lower::m::lower_m_into;
use crate::ir::IrBuilder;

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
        // I instructions
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
        | Instruction::Ecall
        | Instruction::Ebreak => lower_i_into(insn, current_pc, next_pc, builder),

        // M instructions
        #[cfg(feature = "ext_m")]
        Instruction::Mul(_)
        | Instruction::Mulh(_)
        | Instruction::Mulhsu(_)
        | Instruction::Mulhu(_)
        | Instruction::Mulw(_)
        | Instruction::Div(_)
        | Instruction::Divu(_)
        | Instruction::Rem(_)
        | Instruction::Remu(_)
        | Instruction::Divw(_)
        | Instruction::Divuw(_)
        | Instruction::Remw(_)
        | Instruction::Remuw(_) => lower_m_into(insn, current_pc, next_pc, builder),

        // A instructions
        #[cfg(feature = "ext_a")]
        Instruction::LrW(_)
        | Instruction::ScW(_)
        | Instruction::AmoSwapW(_)
        | Instruction::AmoAddW(_)
        | Instruction::AmoXorW(_)
        | Instruction::AmoAndW(_)
        | Instruction::AmoOrW(_)
        | Instruction::AmoMinW(_)
        | Instruction::AmoMaxW(_)
        | Instruction::AmoMinuW(_)
        | Instruction::AmoMaxuW(_)
        | Instruction::LrD(_)
        | Instruction::ScD(_)
        | Instruction::AmoSwapD(_)
        | Instruction::AmoAddD(_)
        | Instruction::AmoXorD(_)
        | Instruction::AmoAndD(_)
        | Instruction::AmoOrD(_)
        | Instruction::AmoMinD(_)
        | Instruction::AmoMaxD(_)
        | Instruction::AmoMinuD(_)
        | Instruction::AmoMaxuD(_) => lower_a_into(insn, current_pc, next_pc, builder),

        // Other instructions
        _ => panic!("no lowering found for {:?}", insn),
    }
}
