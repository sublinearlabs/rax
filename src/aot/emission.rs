use crate::aot::{temp_alloc::TempAllocator, translator::Translator};
use crate::decode::{B, I, J, R, S, Sh, U, Instruction};

pub(super) fn emit_instruction(
    translator: &mut Translator,
    temps: &TempAllocator,
    insn: &Instruction,
) {
    match insn {
        Instruction::Add(R { rd, rs1, rs2 }) => emit_add(translator, temps, *rd, *rs1, *rs2),
        Instruction::Sub(R { rd, rs1, rs2 }) => emit_sub(translator, temps, *rd, *rs1, *rs2),
        Instruction::Or(R { rd, rs1, rs2 }) => emit_or(translator, temps, *rd, *rs1, *rs2),
        Instruction::Subw(R { rd, rs1, rs2 }) => emit_subw(translator, temps, *rd, *rs1, *rs2),
        Instruction::Mulhu(R { rd, rs1, rs2 }) => {
            emit_mulhu(translator, temps, *rd, *rs1, *rs2)
        }
        Instruction::Addi(I { rd, rs1, imm }) => emit_addi(translator, temps, *rd, *rs1, *imm),
        Instruction::Andi(I { rd, rs1, imm }) => emit_andi(translator, temps, *rd, *rs1, *imm),
        Instruction::Slli(Sh { rd, rs1, shamt }) => {
            emit_slli(translator, temps, *rd, *rs1, *shamt)
        }
        Instruction::Sll(R { rd, rs1, rs2 }) => emit_sll(translator, temps, *rd, *rs1, *rs2),
        Instruction::Sb(S { rs1, rs2, imm }) => emit_sb(translator, temps, *rs1, *rs2, *imm),
        Instruction::Sd(S { rs1, rs2, imm }) => emit_sd(translator, temps, *rs1, *rs2, *imm),
        Instruction::Lui(U { rd, imm }) => emit_lui(translator, temps, *rd, *imm),
        Instruction::Auipc(U { rd, imm }) => emit_auipc(translator, temps, *rd, *imm),
        Instruction::Beq(B { rs1, rs2, imm }) => emit_beq(translator, temps, *rs1, *rs2, *imm),
        Instruction::Bne(B { rs1, rs2, imm }) => emit_bne(translator, temps, *rs1, *rs2, *imm),
        Instruction::Bltu(B { rs1, rs2, imm }) => {
            emit_bltu(translator, temps, *rs1, *rs2, *imm)
        }
        Instruction::Bgeu(B { rs1, rs2, imm }) => {
            emit_bgeu(translator, temps, *rs1, *rs2, *imm)
        }
        Instruction::Jal(J { rd, imm }) => emit_jal(translator, temps, *rd, *imm),
        Instruction::Jalr(I { rd, rs1, imm }) => emit_jalr(translator, temps, *rd, *rs1, *imm),
        Instruction::Ecall => emit_ecall(translator, temps),
        Instruction::Csrrw(_) => {}
        _ => panic!("unknown opcode: {:?}", insn),
    }
}

fn emit_add(translator: &mut Translator, temps: &TempAllocator, rd: u8, rs1: u8, rs2: u8) {
    let _ = (translator, temps, rd, rs1, rs2);
    todo!("emit_add")
}

fn emit_sub(translator: &mut Translator, temps: &TempAllocator, rd: u8, rs1: u8, rs2: u8) {
    let _ = (translator, temps, rd, rs1, rs2);
    todo!("emit_sub")
}

fn emit_or(translator: &mut Translator, temps: &TempAllocator, rd: u8, rs1: u8, rs2: u8) {
    let _ = (translator, temps, rd, rs1, rs2);
    todo!("emit_or")
}

fn emit_subw(translator: &mut Translator, temps: &TempAllocator, rd: u8, rs1: u8, rs2: u8) {
    let _ = (translator, temps, rd, rs1, rs2);
    todo!("emit_subw")
}

fn emit_mulhu(translator: &mut Translator, temps: &TempAllocator, rd: u8, rs1: u8, rs2: u8) {
    let _ = (translator, temps, rd, rs1, rs2);
    todo!("emit_mulhu")
}

fn emit_addi(translator: &mut Translator, temps: &TempAllocator, rd: u8, rs1: u8, imm: i32) {
    let _ = (translator, temps, rd, rs1, imm);
    todo!("emit_addi")
}

fn emit_andi(translator: &mut Translator, temps: &TempAllocator, rd: u8, rs1: u8, imm: i32) {
    let _ = (translator, temps, rd, rs1, imm);
    todo!("emit_andi")
}

fn emit_slli(translator: &mut Translator, temps: &TempAllocator, rd: u8, rs1: u8, shamt: u8) {
    let _ = (translator, temps, rd, rs1, shamt);
    todo!("emit_slli")
}

fn emit_sll(translator: &mut Translator, temps: &TempAllocator, rd: u8, rs1: u8, rs2: u8) {
    let _ = (translator, temps, rd, rs1, rs2);
    todo!("emit_sll")
}

fn emit_sb(translator: &mut Translator, temps: &TempAllocator, rs1: u8, rs2: u8, imm: i32) {
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_sb")
}

fn emit_sd(translator: &mut Translator, temps: &TempAllocator, rs1: u8, rs2: u8, imm: i32) {
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_sd")
}

fn emit_lui(translator: &mut Translator, temps: &TempAllocator, rd: u8, imm: i32) {
    let _ = (translator, temps, rd, imm);
    todo!("emit_lui")
}

fn emit_auipc(translator: &mut Translator, temps: &TempAllocator, rd: u8, imm: i32) {
    let _ = (translator, temps, rd, imm);
    todo!("emit_auipc")
}

fn emit_beq(translator: &mut Translator, temps: &TempAllocator, rs1: u8, rs2: u8, imm: i32) {
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_beq")
}

fn emit_bne(translator: &mut Translator, temps: &TempAllocator, rs1: u8, rs2: u8, imm: i32) {
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_bne")
}

fn emit_bltu(translator: &mut Translator, temps: &TempAllocator, rs1: u8, rs2: u8, imm: i32) {
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_bltu")
}

fn emit_bgeu(translator: &mut Translator, temps: &TempAllocator, rs1: u8, rs2: u8, imm: i32) {
    let _ = (translator, temps, rs1, rs2, imm);
    todo!("emit_bgeu")
}

fn emit_jal(translator: &mut Translator, temps: &TempAllocator, rd: u8, imm: i32) {
    let _ = (translator, temps, rd, imm);
    todo!("emit_jal")
}

fn emit_jalr(translator: &mut Translator, temps: &TempAllocator, rd: u8, rs1: u8, imm: i32) {
    let _ = (translator, temps, rd, rs1, imm);
    todo!("emit_jalr")
}

fn emit_ecall(translator: &mut Translator, temps: &TempAllocator) {
    let _ = (translator, temps);
    todo!("emit_ecall")
}
