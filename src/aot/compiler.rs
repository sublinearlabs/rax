use crate::decode::Instruction;
use dynasmrt::x86::Assembler;

fn translate_insns(insns: &[Instruction], assembler: Assembler) {
    for insn in insns {
        match insn {
            Instruction::Add(r) => todo!(),
            _ => todo!(),
        }
    }
}
