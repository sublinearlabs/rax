use crate::{
    aot::register_mapping::{RegisterLocation, RegisterMapping, RiscvRegister, XmmLane},
    decode::{Instruction, I, R},
};
use dynasmrt::{dynasm, x64::Assembler, DynasmApi};

struct AllocatedReg {
    source: RegisterLocation,
    // TODO: type GPR
    dest: u8,
}

struct Compiler {
    ops: Assembler,
    register_mapping: RegisterMapping,
    current_temp: usize,
}

impl Compiler {
    /// Converts a slice of RISCV Instruction to their corresponding
    /// x86 instructions
    fn translate_insns(&mut self, insns: &[Instruction]) {
        for insn in insns {
            self.translate_insn(insn);
            self.reset_temp();
        }
    }

    /// Converts a single RISCV instruction to its corresponding x86 instruction
    fn translate_insn(&mut self, insn: &Instruction) {
        // TODO: if rd == 0 we probably should not emit assembly
        match insn {
            Instruction::Add(R { rd, rs1, rs2 }) => {
                let rs1 = self.prepare_input(*rs1);
                let rs2 = self.prepare_input(*rs2);
                let rd = self.prepare_output(*rd);

                if rd.dest != rs1.dest {
                    dynasm!(self.ops
                        ; mov Rq(rd.dest), Rq(rs1.dest)
                    );
                }

                dynasm!(self.ops
                    ; add Rq(rd.dest), Rq(rs2.dest)
                );

                self.writeback_result(rd);
            }

            Instruction::Addi(I { rd, rs1, imm }) => {
                let rs1 = self.prepare_input(*rs1);
                let rd = self.prepare_output(*rd);

                if rd.dest != rs1.dest {
                    dynasm!(self.ops; mov Rq(rd.dest), Rq(rs1.dest));
                }

                dynasm!(self.ops; add Rq(rd.dest), *imm);

                self.writeback_result(rd);
            }

            // TODO:
            // addi
            // add
            // sub
            // subw
            // andi
            // slli
            // or
            // mulhu
            // lui
            // auipc
            // jalr
            // beq
            // bne
            // bgeu
            // bltu
            // sd
            // sb
            // ecal
            _ => todo!(),
        }
    }

    /// Finds a GPR register for a given riscv register
    /// if the riscv register has already been mapped to a gpr register nothing is done
    /// if mapped to xmm, then it will be moved to a temp register first
    fn prepare_input(&mut self, reg: u8) -> AllocatedReg {
        let reg_location = self.register_mapping[RiscvRegister::new(reg)];
        let dest;
        match reg_location {
            RegisterLocation::Gpr(idx) => {
                // we don't do anything, already gpr
                dest = idx;
            }
            RegisterLocation::Xmm(xmm) | RegisterLocation::XmmShared(xmm, XmmLane::LOWER) => {
                dest = self.temp();
                dynasm!(self.ops
                    ; movq Rq(dest), Rx(xmm)
                );
            }
            RegisterLocation::XmmShared(xmm, XmmLane::UPPER) => {
                dest = self.temp();
                dynasm!(self.ops
                    ; pextrq Rq(dest), Rx(xmm), 1
                );
            }
        }

        AllocatedReg {
            source: reg_location,
            dest,
        }
    }

    /// Determines which GPR register a riscv register will be mapped to
    /// it does not emit any x86 instruction
    fn prepare_output(&mut self, reg: u8) -> AllocatedReg {
        let reg_location = self.register_mapping[RiscvRegister::new(reg)];
        let dest;
        match reg_location {
            RegisterLocation::Gpr(idx) => dest = idx,
            RegisterLocation::Xmm(_) | RegisterLocation::XmmShared(_, _) => dest = self.temp(),
        }
        AllocatedReg {
            source: reg_location,
            dest,
        }
    }

    /// Writes the value stored in a temp gpr register to its target location
    fn writeback_result(&mut self, reg_info: AllocatedReg) {
        match reg_info.source {
            RegisterLocation::Gpr(_) => {
                // already GPR no need for writeback
            }
            RegisterLocation::Xmm(xmm) => {
                dynasm!(self.ops
                    ; movq Rx(xmm), Rq(reg_info.dest)
                );
            }
            RegisterLocation::XmmShared(xmm, XmmLane::LOWER) => {
                dynasm!(self.ops
                    ; pinsrq Rx(xmm), Rq(reg_info.dest), 0
                );
            }
            RegisterLocation::XmmShared(xmm, XmmLane::UPPER) => {
                dynasm!(self.ops
                    ; pinsrq Rx(xmm), Rq(reg_info.dest), 1
                );
            }
        }
    }

    /// Returns a temporary GPR register
    /// advaances the currrent temp register also
    fn temp(&mut self) -> u8 {
        self.current_temp += 1;
        self.register_mapping.temps[self.current_temp - 1]
    }

    /// Reset the temp counter to the first temp variable
    fn reset_temp(&mut self) {
        self.current_temp = 0;
    }
}
