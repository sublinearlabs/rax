use crate::{
    aot::register_mapping::{RegisterLocation, RegisterMapping, RiscvRegister, XmmLane},
    decode::{Instruction, R},
};
use dynasmrt::{dynasm, relocations::SimpleRelocation, x64::Assembler, DynasmApi};

/// Converts a slice of RISCV Instruction to their corresponding
/// x86 instructions
fn translate_insns(insns: &[Instruction], register_mapping: &RegisterMapping) {
    let mut ops = Assembler::new().unwrap();

    // the next assumption is that spilled registers will be moved
    // to some of the temp registers first
    // then, instructions will work with the finalized temp registers
    // hence the register mapping needs some notion of temporary
    // and we can assume we need three temps
    // so some kind of prepare register function that returns the needed registers
    // but also the write back logic
    //
    // now I have mov_to_gpr which can take a register and some temp
    // I can return whether it was moved or not

    for insn in insns {
        match insn {
            Instruction::Add(R { rd, rs1, rs2 }) => {
                // let us assume just GPR for now
                // note: also need to handle the zero register
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
        RegisterLocation::Gpr(loc_index) => loc_index,
        _ => unimplemented!(),
    }
}

/// Emits assembly instruction to move RegisterContents in non-gpr locations
/// to some target_gpr.
/// If a movement occurs it returns target_gpr + 1
/// If no movement returns target_gpr
// TODO: make the target gpr typed
// TODO: rather than returning u8, return next temp gpr
fn mov_to_gpr(location: &RegisterLocation, target_gpr: u8, ops: &mut Assembler) -> u8 {
    match location {
        RegisterLocation::Gpr(_) => {
            // do nothing, already gpr
            target_gpr
        }
        RegisterLocation::Xmm(xmm) | RegisterLocation::XmmShared(xmm, XmmLane::LOWER) => {
            dynasm!(ops
                ; movq Rq(target_gpr), Rx(*xmm)
            );
            target_gpr + 1
        }
        RegisterLocation::XmmShared(xmm, XmmLane::UPPER) => {
            dynasm!(ops
                ; pextrq Rq(target_gpr), Rx(*xmm), 1
            );
            target_gpr + 1
        }
    }
}
