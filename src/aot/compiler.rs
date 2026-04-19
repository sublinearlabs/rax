use crate::{
    aot::register_mapping::{RegisterLocation, RegisterMapping, RiscvRegister, XmmLane},
    decode::{Instruction, Sh, B, I, R, S, U},
};
use alloy_primitives::map::foldhash::HashMap;
use dynasmrt::{dynasm, x64::Assembler, DynamicLabel, DynasmApi, DynasmLabelApi};

const RAX: u8 = 0;
const RDX: u8 = 2;

struct AllocatedReg {
    source: RegisterLocation,
    // TODO: type GPR
    dest: u8,
}

enum AluRrOp {
    Add,
    Sub,
    Or,
    Subw,
    Mulhu,
}

enum AluRiOp {
    Addi,
    Andi,
}

enum ShiftRiOp {
    Slli,
}

enum StoreOp {
    Sb,
    Sd,
}

enum UpperOp {
    Lui,
    Auipc,
}

enum BranchOp {
    Beq,
    Bne,
    Bltu,
}

struct Compiler {
    ops: Assembler,
    register_mapping: RegisterMapping,
    current_temp: usize,
    current_riscv_pc: u64,
    pc_labels: HashMap<u64, DynamicLabel>,
}

impl Compiler {
    /// Converts a slice of RISCV Instruction to their corresponding
    /// x86 instructions
    fn translate_insns(&mut self, insns: &[Instruction]) {
        for insn in insns {
            self.translate_insn(insn);
            self.reset_temp();
        }

        // TODO: resolve the dynamic labels
    }

    /// Converts a single RISCV instruction to its corresponding x86 instruction
    fn translate_insn(&mut self, insn: &Instruction) {
        match insn {
            // ALU REGISTER REGISTER
            Instruction::Add(R { rd, rs1, rs2 }) => self.emit_alu_rr(rd, rs1, rs2, AluRrOp::Add),
            Instruction::Sub(R { rd, rs1, rs2 }) => self.emit_alu_rr(rd, rs1, rs2, AluRrOp::Sub),
            Instruction::Or(R { rd, rs1, rs2 }) => self.emit_alu_rr(rd, rs1, rs2, AluRrOp::Or),
            Instruction::Subw(R { rd, rs1, rs2 }) => self.emit_alu_rr(rd, rs1, rs2, AluRrOp::Subw),
            Instruction::Mulhu(R { rd, rs1, rs2 }) => {
                self.emit_alu_rr(rd, rs1, rs2, AluRrOp::Mulhu)
            }

            // ALU REGISTER IMMEDIATE
            Instruction::Addi(I { rd, rs1, imm }) => self.emit_alu_ri(rd, rs1, imm, AluRiOp::Addi),
            Instruction::Andi(I { rd, rs1, imm }) => self.emit_alu_ri(rd, rs1, imm, AluRiOp::Andi),

            // SHIFT REGISTER IMMEDIATE
            Instruction::Slli(Sh { rd, rs1, shamt }) => {
                self.emit_shift_ri(rd, rs1, shamt, ShiftRiOp::Slli)
            }

            // STORES
            Instruction::Sb(S { rs1, rs2, imm }) => self.emit_store(rs1, rs2, imm, StoreOp::Sb),
            Instruction::Sd(S { rs1, rs2, imm }) => self.emit_store(rs1, rs2, imm, StoreOp::Sd),

            // UPPER
            Instruction::Lui(U { rd, imm }) => self.emit_upper(rd, imm, UpperOp::Lui),
            Instruction::Auipc(U { rd, imm }) => self.emit_upper(rd, imm, UpperOp::Auipc),

            // CONTROL
            Instruction::Beq(B { rs1, rs2, imm }) => self.emit_branch(rs1, rs2, imm, BranchOp::Beq),

            // TODO:
            // control
            // -------
            // jalr
            //
            // system
            // ------
            // ecall
            _ => todo!(),
        }
    }

    /// Converts alu register register instructions to equivalent x86 assembly
    fn emit_alu_rr(&mut self, rd: &u8, rs1: &u8, rs2: &u8, alu_op: AluRrOp) {
        // the zero register is always zero
        if *rd == 0 {
            return;
        }

        let rs1 = self.prepare_input(*rs1);
        let rs2 = self.prepare_input(*rs2);
        let rd = self.prepare_output(*rd);

        if rd.dest != rs1.dest {
            dynasm!(self.ops ; mov Rq(rd.dest), Rq(rs1.dest));
        }

        match alu_op {
            AluRrOp::Add => dynasm!(self.ops ; add Rq(rd.dest), Rq(rs2.dest)),
            AluRrOp::Sub => dynasm!(self.ops ; sub Rq(rd.dest), Rq(rs2.dest)),
            AluRrOp::Or => dynasm!(self.ops ; or Rq(rd.dest), Rq(rs2.dest)),
            AluRrOp::Subw => {
                // subtract the lower 32 bits
                dynasm!(self.ops; sub Rd(rd.dest), Rd(rs2.dest));
                // sign extend the result to 64 bits
                dynasm!(self.ops; movsxd Rq(rd.dest), Rd(rd.dest));
            }
            AluRrOp::Mulhu => {
                // x86 mul r/m64 uses the rdx and rax as implicit registers
                // RDX:RAX = RAX * r/m64
                // so high XLEN bits of the multiplication are in RDX
                // and low XLEN bits of the multiplication are in RAX
                //
                // for Mulhu we want to store the high XLEN bits in rd

                // TODO: depending on the state of the temp regiters
                // we might have to spill the current values of rax and rdx
                // as mul will clobber their current values

                if rs1.dest != RAX {
                    dynasm!(self.ops ; mov rax, Rq(rs1.dest));
                }

                dynasm!(self.ops ; mul Rq(rs2.dest));

                if rd.dest != RDX {
                    dynasm!(self.ops ; mov Rq(rd.dest), rdx);
                }
            }
        }

        self.writeback_result(rd);
    }

    /// Converts alu register immediate instructions to equivalent x86 assembly
    fn emit_alu_ri(&mut self, rd: &u8, rs1: &u8, imm: &i32, alu_op: AluRiOp) {
        // the zero register is always zero
        if *rd == 0 {
            return;
        }

        let rs1 = self.prepare_input(*rs1);
        let rd = self.prepare_output(*rd);

        if rd.dest != rs1.dest {
            dynasm!(self.ops ; mov Rq(rd.dest), Rq(rs1.dest));
        }

        match alu_op {
            AluRiOp::Addi => dynasm!(self.ops ; add Rq(rd.dest), *imm),
            AluRiOp::Andi => dynasm!(self.ops ; and Rq(rd.dest), *imm),
        }

        self.writeback_result(rd);
    }

    /// Converts shift register immediate instructions to equivalent x86 assembly
    fn emit_shift_ri(&mut self, rd: &u8, rs1: &u8, shamt: &u8, shift_op: ShiftRiOp) {
        // the zero register is always zero
        if *rd == 0 {
            return;
        }

        let rs1 = self.prepare_input(*rs1);
        let rd = self.prepare_output(*rd);

        if rd.dest != rs1.dest {
            dynasm!(self.ops ; mov Rq(rd.dest), Rq(rs1.dest));
        }

        match shift_op {
            ShiftRiOp::Slli => dynasm!(self.ops ; shl Rq(rd.dest), *shamt as i8),
        }

        self.writeback_result(rd);
    }

    /// Converts store opreations to equivalent x86 assembly
    fn emit_store(&mut self, rs1: &u8, rs2: &u8, imm: &i32, store_op: StoreOp) {
        let rs1 = self.prepare_input(*rs1);

        // TODO: there are ways to move bytes from xmm
        // so one might not need to prepare input for rs2
        let rs2 = self.prepare_input(*rs2);

        match store_op {
            // store byte
            // mov r/m8, r8
            StoreOp::Sb => dynasm!(self.ops ; mov BYTE [Rq(rs1.dest) + *imm], Rb(rs2.dest)),
            // store double (double in riscv is 64 bits)
            // mov r/m64, r64
            StoreOp::Sd => dynasm!(self.ops ; mov QWORD [Rq(rs1.dest) + *imm], Rq(rs2.dest)),
        }
    }

    /// Converts immediate upper instructions to equivalent x86 assembly
    fn emit_upper(&mut self, rd: &u8, imm: &i32, upper_op: UpperOp) {
        // the zero register is always zero
        if *rd == 0 {
            return;
        }

        let rd = self.prepare_output(*rd);

        // note: the immediate has already been shifted by 12 on the decode layer
        match upper_op {
            UpperOp::Lui => dynasm!(self.ops ; mov Rq(rd.dest), *imm),
            UpperOp::Auipc => {
                let auipc_val = self.current_riscv_pc.wrapping_add(*imm as i64 as u64);
                dynasm!(self.ops ; mov Rq(rd.dest), QWORD auipc_val as i64);
            }
        }

        self.writeback_result(rd);
    }

    /// Converts branchs instructions with known targets to equivalent x86 assembly
    fn emit_branch(&mut self, rs1: &u8, rs2: &u8, imm: &i32, branch_op: BranchOp) {
        let rs1 = self.prepare_input(*rs1);
        let rs2 = self.prepare_input(*rs2);

        dynasm!(self.ops ; cmp Rq(rs1.dest), Rq(rs2.dest));

        // computes the target riscv pc
        // we'd need to convert this to the equivalent riscv label
        // problem is this pc might be references a location in the future
        // hence the jump table won't be populated for this pc value yet
        //
        // to solve this we make use of a dynamic label that we'd patch
        // after we have translated all the riscv instructions
        let branch_target = self.current_riscv_pc.wrapping_add(*imm as i64 as u64);

        // retrieve or create a new dynamic label for the target riscv pc
        let target_label = self
            .pc_labels
            .entry(branch_target)
            .or_insert_with(|| self.ops.new_dynamic_label());

        match branch_op {
            BranchOp::Beq => dynasm!(self.ops ; je =>*target_label),
            BranchOp::Bne => dynasm!(self.ops ; jne =>*target_label),
            BranchOp::Bltu => dynasm!(self.ops ; jb =>*target_label),
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
        // TODO: implement bound checks on temp
        self.current_temp += 1;
        self.register_mapping.temps[self.current_temp - 1]
    }

    /// Reset the temp counter to the first temp variable
    fn reset_temp(&mut self) {
        self.current_temp = 0;
    }
}
