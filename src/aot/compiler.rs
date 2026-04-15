use crate::{
    aot::register_mapping::{RegisterMapping, RiscvRegister},
    decode::{Instruction, R},
};
use dynasmrt::x86::Assembler;

// need to figure out what is needed for this to be done
// one concern is register mapping

/// Converts a slice of RISCV Instruction to their corresponding
/// x86 instructions
fn translate_insns(insns: &[Instruction], assembler: Assembler, register_mapping: RegisterMapping) {
    let ops = Assembler::new().unwrap();

    for insn in insns {
        match insn {
            Instruction::Add(R { rd, rs1, rs2 }) => {
                // what instructions are needed here?
                // we are starting with
                // add rd, rs1, rs2
                // the expected result is rd = rs1 + rs2
                // we can simulate that with

                let rd = RiscvRegister::new(*rd);
                let rs1 = RiscvRegister::new(*rs1);
                let rs2 = RiscvRegister::new(*rs2);

                if rd != rs1 {
                    // mov R(rd), R(rs1)
                }

                // add R(rd), R(rs2)
            }
            _ => todo!(),
        }
    }
}
