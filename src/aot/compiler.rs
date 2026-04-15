use crate::{
    aot::register_mapping::{RegisterLocation, RegisterMapping, RiscvRegister, XmmLane},
    decode::{Instruction, R},
};
use dynasmrt::{dynasm, x64::Assembler, DynasmApi};

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
    // what I need now is a new register mapping structure

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

/// Normalize riscv registers to x86 general purpose registers
/// if the riscv register is already mapped to an x86 gpr nothing is done
/// if the riscv register is in xmm, it is move to one of the temp gprs
/// returns a list of the finalized gpr registers
// TODO: make the gpr register typed
fn prepare_registers<const N: usize>(
    registers: [u8; N],
    mapping: &RegisterMapping,
    ops: &mut Assembler,
) -> [u8; N] {
    let mut current_temp = mapping.temp_base;

    std::array::from_fn(|i| {
        let reg = registers[i];
        let loc = &mapping[RiscvRegister::new(reg)];
        let (dst, new_temp) = mov_to_gpr(loc, current_temp, ops);
        current_temp = new_temp;
        dst
    })
}

/// Emits assembly instruction to move RegisterContents in non-gpr locations
/// to some target_gpr.
/// If a movement occurs it returns target_gpr + 1
/// If no movement returns target_gpr
// TODO: make the target gpr typed
// TODO: rather than returning u8, return next temp gpr
fn mov_to_gpr(location: &RegisterLocation, target_gpr: u8, ops: &mut Assembler) -> (u8, u8) {
    match location {
        RegisterLocation::Gpr(reg) => {
            // do nothing, already gpr
            (*reg, target_gpr)
        }
        RegisterLocation::Xmm(xmm) | RegisterLocation::XmmShared(xmm, XmmLane::LOWER) => {
            dynasm!(ops
                ; movq Rq(target_gpr), Rx(*xmm)
            );
            (target_gpr, target_gpr + 1)
        }
        RegisterLocation::XmmShared(xmm, XmmLane::UPPER) => {
            dynasm!(ops
                ; pextrq Rq(target_gpr), Rx(*xmm), 1
            );
            (target_gpr, target_gpr + 1)
        }
    }
}
