use std::collections::HashMap;

use crate::{
    aot::register_mapping::{RegisterLocation, RegisterMapping, RiscvRegister, XmmLane},
    decode::{Instruction, Sh, B, I, J, R, S, U},
};
use dynasmrt::{dynasm, x64::Assembler, AssemblyOffset, DynamicLabel, DynasmApi, DynasmLabelApi};

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

enum ShiftRrOp {
    Sll,
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
    Bgeu,
}

pub(crate) struct Compiler {
    ops: Assembler,
    register_mapping: RegisterMapping,
    current_temp: usize,
    current_riscv_pc: u64,
    base_riscv_pc: u64,
    pc_labels: HashMap<u64, DynamicLabel>,
    // TODO: because of the current state of this structure
    // we can really only have this work for non-compressed
    // riscv elfs and a single 'read-execute' segment.
    // also, for correct functioning the compiler needs to be
    // passed the vaddr of the code section, so it can compute the
    // absolute jump address.
    jump_table: Vec<AssemblyOffset>,
    jt_label: DynamicLabel,
}

impl Compiler {
    /// Initializes a new compiler
    pub(crate) fn init(
        mut assembler: Assembler,
        register_mapping: RegisterMapping,
        base_pc: u64,
    ) -> Self {
        let jt_label = assembler.new_dynamic_label();
        Self {
            ops: assembler,
            register_mapping,
            current_temp: 0,
            current_riscv_pc: base_pc,
            base_riscv_pc: base_pc,
            pc_labels: HashMap::new(),
            // TODO: init with capacity
            jump_table: Vec::new(),
            jt_label,
        }
    }

    /// Converts a slice of RISCV Instruction to their corresponding
    /// x86 instructions
    pub(crate) fn translate_insns(&mut self, insns: &[Instruction]) {
        // TODO: remove this
        // clear out the zero register
        let temp = self.temp();
        dynasm!(self.ops ; xor Rq(temp), Rq(temp));
        dynasm!(self.ops ; pinsrq Rx(11), Rq(temp), 1);
        dynasm!(self.ops ; nop);
        self.reset_temp();

        for insn in insns {
            self.translate_insn(insn);
            self.reset_temp();
            self.current_riscv_pc += 4;
            dynasm!(self.ops ; nop);
        }

        // resolve dynamic labels
        for (index, label) in self.pc_labels.iter() {
            let jump_table_index = (index - self.base_riscv_pc) / 4;
            self.ops
                .labels_mut()
                .define_dynamic(*label, self.jump_table[jump_table_index as usize])
                .expect("failed to define dynamic label");
        }

        // compute the absolute addresses for each jump table entry
        // by adding the base_address of this segement
        // TODO: this just uses the base_riscv_pc for now
        // works for echo, but might not work for other binary
        // a more sophisticated approach will be needed
        let jump_table = self
            .jump_table
            .iter()
            .map(|offset| offset.0 + self.base_riscv_pc as usize)
            .collect::<Vec<_>>();

        // emit the jump table
        // TODO: consider moving this to rodata
        dynasm!(self.ops ; =>self.jt_label);
        for target_pc in jump_table {
            dynasm!(self.ops; .i64 target_pc as i64);
        }
    }

    /// Converts a single RISCV instruction to its corresponding x86 instruction
    fn translate_insn(&mut self, insn: &Instruction) {
        // populate the jump table for the current pc
        // assumes that the pc jump by 4 (uncompressed) and a single read execute segment
        self.jump_table.push(self.ops.offset());

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

            // SHIFT REGISTER REGISTER
            Instruction::Sll(R { rd, rs1, rs2 }) => {
                self.emit_shift_rr(rd, rs1, rs2, ShiftRrOp::Sll)
            }

            // STORES
            Instruction::Sb(S { rs1, rs2, imm }) => self.emit_store(rs1, rs2, imm, StoreOp::Sb),
            Instruction::Sd(S { rs1, rs2, imm }) => self.emit_store(rs1, rs2, imm, StoreOp::Sd),

            // UPPER
            Instruction::Lui(U { rd, imm }) => self.emit_upper(rd, imm, UpperOp::Lui),
            Instruction::Auipc(U { rd, imm }) => self.emit_upper(rd, imm, UpperOp::Auipc),

            // CONTROL
            Instruction::Beq(B { rs1, rs2, imm }) => self.emit_branch(rs1, rs2, imm, BranchOp::Beq),
            Instruction::Bne(B { rs1, rs2, imm }) => self.emit_branch(rs1, rs2, imm, BranchOp::Bne),
            Instruction::Bltu(B { rs1, rs2, imm }) => {
                self.emit_branch(rs1, rs2, imm, BranchOp::Bltu)
            }
            Instruction::Bgeu(B { rs1, rs2, imm }) => {
                self.emit_branch(rs1, rs2, imm, BranchOp::Bgeu)
            }
            Instruction::Jal(J { rd, imm }) => self.emit_jal(rd, imm),
            Instruction::Jalr(I { rd, rs1, imm }) => self.emit_jalr(rd, rs1, imm),

            // SYSTEM
            Instruction::Ecall => self.emit_ecall(),

            Instruction::Csrrw(_) => {}

            _ => panic!("unknown opcode"),
        }
    }

    /// Consume the compiler and return the generated assembly
    pub(crate) fn finalize(self) -> Vec<u8> {
        let buf = self.ops.finalize().unwrap();
        buf.to_vec()
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
                //
                // TODO: we need forced spill for now I believe

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

    /// Converts shift register resgier instructions to equivalent x86 assembly
    fn emit_shift_rr(&mut self, rd: &u8, rs1: &u8, rs2: &u8, shift_op: ShiftRrOp) {
        // the zero register is always zero
        if *rd == 0 {
            return;
        }

        let rs1 = self.prepare_input(*rs1);
        let rs2 = self.prepare_input(*rs2);
        let rd = self.prepare_output(*rd);

        // TODO: need a new capability here
        // need a way to be able to specify my exact temp register
        // shl when working with a shamt in the reg, requires that the
        // value be in rcx

        match shift_op {
            ShiftRrOp::Sll => {
                // move the shift amount
                // TODO: there is a constraint here that rcx must be temp
                // if it is not temp, then the content will be clobbered,
                // and we'd have to move it.
                // TODO: use temp approach for this rather than ecx directly
                dynasm!(self.ops ; mov ecx, Rd(rs2.dest));

                if rd.dest != rs1.dest {
                    dynasm!(self.ops ; mov Rq(rd.dest), Rq(rs1.dest));
                }

                dynasm!(self.ops ; shl Rq(rd.dest), cl);
            }
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
            BranchOp::Bgeu => dynasm!(self.ops ; jae =>*target_label),
        }
    }

    // TODO: write documentation
    fn emit_jal(&mut self, rd: &u8, imm: &i32) {
        if *rd != 0 {
            let rd = self.prepare_output(*rd);

            let return_pc = self.current_riscv_pc.wrapping_add(4);
            dynasm!(self.ops ; mov Rq(rd.dest), QWORD return_pc as i64);
            self.writeback_result(rd);
        }

        // TODO: remove duplication, very similar to emit_branch

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

        dynasm!(self.ops ; jmp =>*target_label);
    }

    /// Converts the jalr instruction to equivalent x86 assembly
    fn emit_jalr(&mut self, rd: &u8, rs1: &u8, imm: &i32) {
        let rs1 = self.prepare_input(*rs1);

        // BUG: there is a chance that rbx == rd
        // so use the current contents of rd before writing to it
        if *rd != 0 {
            let rd = self.prepare_output(*rd);

            // write the return address to rd
            // TODO: assumes we are always advancing the pc by 4
            let return_pc = self.current_riscv_pc.wrapping_add(4);
            dynasm!(self.ops ; mov Rq(rd.dest), QWORD return_pc as i64);

            self.writeback_result(rd);
        }

        let target = self.temp();
        let base_pc = self.temp();

        // we want to compute the following
        // target = (rs1 + imm) & !1
        // idx = (target - base_riscv_pc) >> 2 (assumes uncompressed)
        dynasm!(self.ops ; lea Rq(target), [Rq(rs1.dest) + *imm]);
        dynasm!(self.ops ; and Rq(target), -2 as i32);
        dynasm!(self.ops ; mov Rq(base_pc), QWORD self.base_riscv_pc as i64);
        dynasm!(self.ops ; sub Rq(target), Rq(base_pc));
        dynasm!(self.ops ; shr Rq(target), 2);

        // reuse the base_pc temp register
        let jump_table_base = base_pc;

        // now we need to get the value at that jump table index
        // and then jump to it
        dynasm!(self.ops ; lea Rq(jump_table_base), [=>self.jt_label]);
        dynasm!(self.ops ; jmp QWORD [Rq(jump_table_base) + Rq(target) * 8]);
    }

    // TODO: write documentation
    fn emit_ecall(&mut self) {
        // we only support 3 syscalls
        // read, write and halt
        // it is assumed that the syscall code and arguments have been mapped identically
        // i.e. the x86 syscall code and arguments are the same as riscv (via register mapping)
        // given this, the only work that needs to be done is translating the riscv
        // syscall code to x86
        // syscall | riscv_code | x86_code
        // read    |     63     |    0
        // write   |     64     |    1
        // halt    |     93     |   60
        //
        // we achieve this by evaluating a polynomial
        // f(x) = $(x^2 - 98x + 2205) / 29$
        // after simplification
        // f(x) = $((x - 49)^2 - 196) / 29$

        // TODO: enforce register mapping constraints here

        // TODO: rax and rdx will be clobbered
        // we might need to move them to temp first and then write back
        // we can skip this step if liveness says otherwise
        //
        // NOTE: rax contains the riscv syscall code, so we can unclobber
        // by just performing extra computations
        // i.e. f(y) = x
        // so no need for temp
        //
        // TODO: also rcx and r11 will be clobbered
        // syscall uses them as scratch values

        // rax = x - 49
        dynasm!(self.ops ; sub rax, 49);
        // rax = (x - 49)^2
        dynasm!(self.ops ; imul rax, rax);
        // rax = (x - 49)^2 - 196
        dynasm!(self.ops ; sub rax, 196);
        // set rdx to be the sign extension of rax
        // if rax is positive, rdx will be all zeros
        // if rax is negative, rdx will be all ones
        dynasm!(self.ops ; cqo);

        // compute ((x - 49)^2 - 196) / 29
        // store quotient in RAX
        // store remainder in RDX
        let divisor_reg = self.temp();
        dynasm!(self.ops ; mov Rq(divisor_reg), 29);
        dynasm!(self.ops ; idiv Rq(divisor_reg));

        // if the riscv a7 register contained
        // the read, write or halt syscall
        // then rax should now contain the correct
        // x86 syscall code

        dynasm!(self.ops ; syscall);
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
                    // needed to use pinsrq instead of movq
                    // as movq overwrite the other lanes
                    ; pinsrq Rx(xmm), Rq(reg_info.dest), 0
                );
            }
            RegisterLocation::XmmShared(xmm, XmmLane::UPPER) => {
                if xmm == 11 {
                    // this is the zero register
                    // so no write back
                } else {
                    dynasm!(self.ops
                        ; pinsrq Rx(xmm), Rq(reg_info.dest), 1
                    );
                }
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
