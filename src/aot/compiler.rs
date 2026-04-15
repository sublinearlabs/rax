use crate::{
    aot::register_mapping::{RegisterLocation, RegisterMapping, RiscvRegister},
    decode::{Instruction, R},
};
use dynasmrt::{dynasm, x86::Assembler, DynasmApi};

/// Converts a slice of RISCV Instruction to their corresponding
/// x86 instructions
fn translate_insns(insns: &[Instruction], register_mapping: &RegisterMapping) {
    let mut ops = Assembler::new().unwrap();

    for insn in insns {
        match insn {
            Instruction::Add(R { rd, rs1, rs2 }) => {
                // I need a way to make this clean
                // I want to be able to know the location

                // let us assume just GPR for now
                let rd = get_register(rd, register_mapping);
                let rs1 = get_register(rs1, register_mapping);
                let rs2 = get_register(rs2, register_mapping);

                if rd != rs1 {
                    dynasm!(ops
                        ; mov Rq(rd), Rq(rs1)
                    );
                }

                dynasm!(ops
                    ; add Rq(rd), Rq(rs2)
                );
            }
            _ => todo!(),
        }
    }
}

/// Returns x86 register associated with any given riscv register
fn get_register(register_id: &u8, mapping: &RegisterMapping) -> u8 {
    match mapping[RiscvRegister::new(*register_id)] {
        RegisterLocation::GPR(loc_index) => loc_index,
        _ => unimplemented!(),
    }
}
