use crate::decode::Instruction;
use crate::ecall::handle_ecall;
use crate::instr_execute::a_instr::{
    execute_AmoAddD, execute_AmoAddW, execute_AmoAndD, execute_AmoAndW, execute_AmoMaxD,
    execute_AmoMaxW, execute_AmoMaxuD, execute_AmoMaxuW, execute_AmoMinD, execute_AmoMinW,
    execute_AmoMinuD, execute_AmoMinuW, execute_AmoOrD, execute_AmoOrW, execute_AmoSwapD,
    execute_AmoSwapW, execute_AmoXorD, execute_AmoXorW, execute_LrD, execute_LrW, execute_ScD,
    execute_ScW,
};
use crate::instr_execute::csr_instr::{
    execute_Csrrc, execute_Csrrci, execute_Csrrs, execute_Csrrsi, execute_Csrrw, execute_Csrrwi,
};
use crate::instr_execute::d_instr::{
    execute_FaddD, execute_FclassD, execute_FcvtDL, execute_FcvtDLu, execute_FcvtDS,
    execute_FcvtDW, execute_FcvtDWu, execute_FcvtLD, execute_FcvtLS, execute_FcvtLuD,
    execute_FcvtLuS, execute_FcvtSD, execute_FcvtSL, execute_FcvtSLu, execute_FcvtWD,
    execute_FcvtWuD, execute_FdivD, execute_FeqD, execute_Fld, execute_FleD, execute_FltD,
    execute_Flw, execute_FmaddD, execute_FmaxD, execute_FminD, execute_FmsubD, execute_FmulD,
    execute_FmvDX, execute_FmvXD, execute_FnmaddD, execute_FnmsubD, execute_Fsd, execute_FsgnjD,
    execute_FsgnjnD, execute_FsgnjxD, execute_FsqrtD, execute_FsubD, execute_Fsw,
};
use crate::instr_execute::f_instr::{
    execute_FaddS, execute_FclassS, execute_FcvtSW, execute_FcvtSWu, execute_FcvtWS,
    execute_FcvtWuS, execute_FdivS, execute_FeqS, execute_FleS, execute_FltS, execute_FmaddS,
    execute_FmaxS, execute_FminS, execute_FmsubS, execute_FmulS, execute_FmvWX, execute_FmvXW,
    execute_FnmaddS, execute_FnmsubS, execute_FsgnjS, execute_FsgnjnS, execute_FsgnjxS,
    execute_FsqrtS, execute_FsubS,
};
use crate::instr_execute::i_instr::{
    execute_Add, execute_Addi, execute_Addiw, execute_Addw, execute_And, execute_Andi,
    execute_Auipc, execute_Beq, execute_Bge, execute_Bgeu, execute_Blt, execute_Bltu, execute_Bne,
    execute_Jal, execute_Jalr, execute_Lb, execute_Lbu, execute_Ld, execute_Lh, execute_Lhu,
    execute_Lui, execute_Lw, execute_Lwu, execute_Or, execute_Ori, execute_Sb, execute_Sd,
    execute_Sh, execute_Sll, execute_Slli, execute_Slliw, execute_Sllw, execute_Slt, execute_Slti,
    execute_Sltiu, execute_Sltu, execute_Sra, execute_Srai, execute_Sraiw, execute_Sraw,
    execute_Srl, execute_Srli, execute_Srliw, execute_Srlw, execute_Sub, execute_Subw, execute_Sw,
    execute_Xor, execute_Xori,
};
use crate::instr_execute::m_instr::{
    execute_Div, execute_Divu, execute_Divuw, execute_Divw, execute_Mul, execute_Mulh,
    execute_Mulhsu, execute_Mulhu, execute_Mulw, execute_Rem, execute_Remu, execute_Remuw,
    execute_Remw,
};
use crate::instr_execute::*;
use crate::trace::{MemOp, Tracer};
use crate::{
    VM, is_snan_f32, is_snan_f64,
    util::{classify32, classify64, mask, mask32, sext},
};

// TODO consider cleaning up sext logic
impl<T: Tracer> VM<T> {
    pub(crate) fn execute_instruction(&mut self, insn: Instruction, is_compressed: bool) {
        match insn {
            // Register Opcodes
            Instruction::Add(insn) => execute_Add(self, insn),

            Instruction::Sub(insn) => execute_Sub(self, insn),

            Instruction::Xor(insn) => execute_Xor(self, insn),

            Instruction::Or(insn) => execute_Or(self, insn),

            Instruction::And(insn) => execute_And(self, insn),

            Instruction::Sll(insn) => execute_Sll(self, insn),

            Instruction::Srl(insn) => execute_Srl(self, insn),

            Instruction::Sra(insn) => execute_Sra(self, insn),

            Instruction::Slt(insn) => execute_Slt(self, insn),

            Instruction::Sltu(insn) => execute_Sltu(self, insn),

            // Immediate Opcodes
            Instruction::Addi(insn) => execute_Addi(self, insn),

            Instruction::Xori(insn) => execute_Xori(self, insn),

            Instruction::Ori(insn) => execute_Ori(self, insn),

            Instruction::Andi(insn) => execute_Andi(self, insn),

            Instruction::Slli(insn) => execute_Slli(self, insn),

            Instruction::Srli(insn) => execute_Srli(self, insn),

            Instruction::Srai(insn) => execute_Srai(self, insn),

            Instruction::Slti(insn) => execute_Slti(self, insn),

            Instruction::Sltiu(insn) => execute_Sltiu(self, insn),

            // Load Opcodes
            Instruction::Lb(insn) => execute_Lb(self, insn),

            Instruction::Lbu(insn) => execute_Lbu(self, insn),

            Instruction::Lh(insn) => execute_Lh(self, insn),

            Instruction::Lhu(insn) => execute_Lhu(self, insn),

            Instruction::Lw(insn) => execute_Lw(self, insn),

            Instruction::Lwu(insn) => execute_Lwu(self, insn),

            Instruction::Ld(insn) => execute_Ld(self, insn),

            // Store Opcodes
            Instruction::Sb(insn) => execute_Sb(self, insn),

            Instruction::Sh(insn) => execute_Sh(self, insn),

            Instruction::Sw(insn) => execute_Sw(self, insn),

            Instruction::Sd(insn) => execute_Sd(self, insn),

            // Branch Opcodes
            Instruction::Beq(insn) => {
                if execute_Beq(self, insn) {
                    return;
                } else {
                }
            }

            Instruction::Bne(insn) => {
                if execute_Bne(self, insn) {
                    return;
                } else {
                }
            }

            Instruction::Blt(insn) => {
                if execute_Blt(self, insn) {
                    return;
                } else {
                }
            }

            Instruction::Bltu(insn) => {
                if execute_Bltu(self, insn) {
                    return;
                } else {
                }
            }

            Instruction::Bge(insn) => {
                if execute_Bge(self, insn) {
                    return;
                } else {
                }
            }

            Instruction::Bgeu(insn) => {
                if execute_Bgeu(self, insn) {
                    return;
                } else {
                }
            }

            // Jump opcodes
            Instruction::Jal(insn) => return execute_Jal(self, insn),

            Instruction::Jalr(insn) => return execute_Jalr(self, insn, is_compressed),

            // Lui and Auipc
            Instruction::Lui(insn) => execute_Lui(self, insn),

            Instruction::Auipc(insn) => execute_Auipc(self, insn),

            // RV64I Instructions
            Instruction::Addiw(insn) => execute_Addiw(self, insn),

            Instruction::Slliw(insn) => execute_Slliw(self, insn),

            Instruction::Srliw(insn) => execute_Srliw(self, insn),

            Instruction::Sraiw(insn) => execute_Sraiw(self, insn),

            Instruction::Addw(insn) => execute_Addw(self, insn),

            Instruction::Subw(insn) => execute_Subw(self, insn),

            Instruction::Sllw(insn) => execute_Sllw(self, insn),

            Instruction::Srlw(insn) => execute_Srlw(self, insn),

            Instruction::Sraw(insn) => execute_Sraw(self, insn),

            // M Extension - Multiplication
            Instruction::Mul(insn) => execute_Mul(self, insn),

            Instruction::Mulh(insn) => execute_Mulh(self, insn),

            Instruction::Mulhsu(insn) => execute_Mulhsu(self, insn),

            Instruction::Mulhu(insn) => execute_Mulhu(self, insn),

            Instruction::Mulw(insn) => execute_Mulw(self, insn),

            // M Extension - Division
            Instruction::Div(insn) => execute_Div(self, insn),

            Instruction::Divu(insn) => execute_Divu(self, insn),

            Instruction::Rem(insn) => execute_Rem(self, insn),

            Instruction::Remu(insn) => execute_Remu(self, insn),

            Instruction::Divw(insn) => execute_Divw(self, insn),

            Instruction::Divuw(insn) => execute_Divuw(self, insn),

            Instruction::Remw(insn) => execute_Remw(self, insn),

            Instruction::Remuw(insn) => execute_Remuw(self, insn),

            // A Extension - Load Reserved / Store Conditional
            Instruction::LrW(insn) => execute_LrW(self, insn),

            Instruction::LrD(insn) => execute_LrD(self, insn),

            Instruction::ScW(insn) => execute_ScW(self, insn),

            Instruction::ScD(insn) => execute_ScD(self, insn),

            // A Extension - Atomic Memory Operations (Word)
            Instruction::AmoSwapW(insn) => execute_AmoSwapW(self, insn),

            Instruction::AmoAddW(insn) => execute_AmoAddW(self, insn),

            Instruction::AmoXorW(insn) => execute_AmoXorW(self, insn),

            Instruction::AmoAndW(insn) => execute_AmoAndW(self, insn),

            Instruction::AmoOrW(insn) => execute_AmoOrW(self, insn),

            Instruction::AmoMinW(insn) => execute_AmoMinW(self, insn),

            Instruction::AmoMaxW(insn) => execute_AmoMaxW(self, insn),

            Instruction::AmoMinuW(insn) => execute_AmoMinuW(self, insn),

            Instruction::AmoMaxuW(insn) => execute_AmoMaxuW(self, insn),

            // A Extension - Atomic Memory Operations (Double)
            Instruction::AmoSwapD(insn) => execute_AmoSwapD(self, insn),

            Instruction::AmoAddD(insn) => execute_AmoAddD(self, insn),

            Instruction::AmoXorD(insn) => execute_AmoXorD(self, insn),

            Instruction::AmoAndD(insn) => execute_AmoAndD(self, insn),

            Instruction::AmoOrD(insn) => execute_AmoOrD(self, insn),

            Instruction::AmoMinD(insn) => execute_AmoMinD(self, insn),

            Instruction::AmoMaxD(insn) => execute_AmoMaxD(self, insn),

            Instruction::AmoMinuD(insn) => execute_AmoMinuD(self, insn),

            Instruction::AmoMaxuD(insn) => execute_AmoMaxuD(self, insn),

            // F instructions
            Instruction::FmaddS(insn) => execute_FmaddS(self, insn),

            Instruction::FmsubS(insn) => execute_FmsubS(self, insn),

            Instruction::FnmsubS(insn) => execute_FnmsubS(self, insn),

            Instruction::FnmaddS(insn) => execute_FnmaddS(self, insn),

            Instruction::FaddS(insn) => execute_FaddS(self, insn),

            Instruction::FsubS(insn) => execute_FsubS(self, insn),

            Instruction::FmulS(insn) => execute_FmulS(self, insn),

            Instruction::FdivS(insn) => execute_FdivS(self, insn),

            Instruction::FsqrtS(insn) => execute_FsqrtS(self, insn),

            Instruction::FsgnjS(insn) => execute_FsgnjS(self, insn),

            Instruction::FsgnjnS(insn) => execute_FsgnjnS(self, insn),

            Instruction::FsgnjxS(insn) => execute_FsgnjxS(self, insn),

            Instruction::FminS(insn) => execute_FminS(self, insn),

            Instruction::FmaxS(insn) => execute_FmaxS(self, insn),

            Instruction::FcvtWS(insn) => execute_FcvtWS(self, insn),

            Instruction::FcvtWuS(insn) => execute_FcvtWuS(self, insn),

            Instruction::FmvXW(insn) => execute_FmvXW(self, insn),

            Instruction::FeqS(insn) => execute_FeqS(self, insn),

            Instruction::FltS(insn) => execute_FltS(self, insn),

            Instruction::FleS(insn) => execute_FleS(self, insn),

            Instruction::FclassS(insn) => execute_FclassS(self, insn),

            Instruction::FcvtSW(insn) => execute_FcvtSW(self, insn),

            Instruction::FcvtSWu(insn) => execute_FcvtSWu(self, insn),

            Instruction::FmvWX(insn) => execute_FmvWX(self, insn),

            Instruction::FmaddD(insn) => execute_FmaddD(self, insn),

            Instruction::FmsubD(insn) => execute_FmsubD(self, insn),

            Instruction::FnmsubD(insn) => execute_FnmsubD(self, insn),

            Instruction::FnmaddD(insn) => execute_FnmaddD(self, insn),

            Instruction::FaddD(insn) => execute_FaddD(self, insn),

            Instruction::FsubD(insn) => execute_FsubD(self, insn),

            Instruction::FmulD(insn) => execute_FmulD(self, insn),

            Instruction::FdivD(insn) => execute_FdivD(self, insn),

            Instruction::FsqrtD(insn) => execute_FsqrtD(self, insn),

            Instruction::FsgnjD(insn) => execute_FsgnjD(self, insn),

            Instruction::FsgnjnD(insn) => execute_FsgnjnD(self, insn),

            Instruction::FsgnjxD(insn) => execute_FsgnjxD(self, insn),

            Instruction::FminD(insn) => execute_FminD(self, insn),

            Instruction::FmaxD(insn) => execute_FmaxD(self, insn),

            Instruction::FcvtSD(insn) => execute_FcvtSD(self, insn),

            Instruction::FcvtDS(insn) => execute_FcvtDS(self, insn),

            Instruction::FeqD(insn) => execute_FeqD(self, insn),

            Instruction::FltD(insn) => execute_FltD(self, insn),

            Instruction::FleD(insn) => execute_FleD(self, insn),

            Instruction::FclassD(insn) => execute_FclassD(self, insn),

            Instruction::FcvtWD(insn) => execute_FcvtWD(self, insn),

            Instruction::FcvtWuD(insn) => execute_FcvtWuD(self, insn),

            Instruction::FcvtDW(insn) => execute_FcvtDW(self, insn),

            Instruction::FcvtDWu(insn) => execute_FcvtDWu(self, insn),

            Instruction::Flw(insn) => execute_Flw(self, insn),

            Instruction::Fsw(insn) => execute_Fsw(self, insn),

            Instruction::Fld(insn) => execute_Fld(self, insn),

            Instruction::Fsd(insn) => execute_Fsd(self, insn),

            Instruction::FcvtLS(insn) => execute_FcvtLS(self, insn),

            Instruction::FcvtLuS(insn) => execute_FcvtLuS(self, insn),

            Instruction::FcvtSL(insn) => execute_FcvtSL(self, insn),

            Instruction::FcvtSLu(insn) => execute_FcvtSLu(self, insn),

            Instruction::FcvtLD(insn) => execute_FcvtLD(self, insn),

            Instruction::FcvtLuD(insn) => execute_FcvtLuD(self, insn),

            Instruction::FmvXD(insn) => execute_FmvXD(self, insn),

            Instruction::FcvtDL(insn) => execute_FcvtDL(self, insn),

            Instruction::FcvtDLu(insn) => execute_FcvtDLu(self, insn),

            Instruction::FmvDX(insn) => execute_FmvDX(self, insn),

            // CSR instructions
            Instruction::Csrrw(insn) => execute_Csrrw(self, insn),

            Instruction::Csrrs(insn) => execute_Csrrs(self, insn),

            Instruction::Csrrc(insn) => execute_Csrrc(self, insn),

            Instruction::Csrrwi(insn) => execute_Csrrwi(self, insn),

            Instruction::Csrrsi(insn) => execute_Csrrsi(self, insn),

            Instruction::Csrrci(insn) => execute_Csrrci(self, insn),

            // System Opcodes
            Instruction::Ecall => {
                handle_ecall(self);
            }

            // TODO remove the eager check once all opcodes have been implemented
            _ => {}
        }

        if is_compressed {
            self.pc += 2;
        } else {
            self.pc += 4;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ecall::constants;
    use crate::trace::NoopTracer;
    use crate::{VM, decode};

    #[test]
    fn test_add_instruction() {
        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(3, 12);
        vm.reg_mut(5, 32);
        // r8 = r3 + r5
        // 0x518433 = Instruction::Add(R { rd: 8, rs1: 3, rs2: 5 });
        let insn = 0x518433;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.reg(8), 12 + 32);
    }

    #[test]
    fn test_store_byte() {
        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(3, 12);
        vm.reg_mut(2, 5);
        // 0x310123 = Instruction::Sb(S {rs1: 2, rs2: 3, imm: 2});
        let insn = 0x310123;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.load_u64(7), 12);
    }

    #[test]
    fn test_store_half_word() {
        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(3, 64008);
        vm.reg_mut(2, 5);
        // 0x311123 = Instruction::Sh(S {rs1: 2, rs2: 3, imm: 2});
        let insn = 0x311123;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.load_u64(7), 64008);
        assert_eq!(vm.load_u64(8), 250);
    }

    #[test]
    fn test_store_word() {
        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(3, 2299561908);
        vm.reg_mut(2, 5);
        // 0x312123 = Instruction::Sw(S { rs1: 2, rs2: 3, imm: 2 });
        let insn = 0x312123;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.load_u64(7), 2299561908);
        assert_eq!(vm.load_u64(8), 8982663);
        assert_eq!(vm.load_u64(9), 35088);
    }

    #[test]
    fn test_store_double_word() {
        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(3, 1234567898765432123);
        vm.reg_mut(2, 5);
        // 0x313123 = Instruction::Sd(S { rs1: 2, rs2: 3, imm: 2 });
        let insn = 0x313123;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.load_u64(7), 1234567898765432123);
        assert_eq!(vm.load_u64(8), 4822530854552469);
        assert_eq!(vm.load_u64(9), 18838011150595);
        assert_eq!(vm.load_u64(10), 73585981057);
        assert_eq!(vm.load_u64(11), 287445238);
        assert_eq!(vm.load_u64(12), 1122832);
    }

    #[test]
    fn test_jal_opcode() {
        let mut vm = VM::<NoopTracer>::init();
        vm.pc = 8;
        // 0xC001EF = Instruction::Jal(J { rd: 3, imm: 12 });
        let insn = 0xC001EF;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.reg(3), 12);
        assert_eq!(vm.pc, 20);
    }

    #[test]
    fn test_jalr_opcode() {
        let mut vm = VM::<NoopTracer>::init();
        vm.pc = 8;
        vm.reg_mut(5, 6);
        // 0x9281E7 = Instruction::Jalr(I {rs1: 5, rd: 3, imm: 9});
        let insn = 0x9281E7;
        vm.execute_instruction(decode(insn), false);
        assert_eq!(vm.reg(3), 12);
        assert_eq!(vm.pc, 15);
    }

    #[test]
    fn test_ecall_stdin() {
        let mut vm = VM::<NoopTracer>::init();

        // Prepare an input stream "hello"
        vm.input_stream = b"hello".to_vec();
        vm.input_cursor = 0;

        // a0 = fd (stdin), a1 = guest ptr, a2 = len
        vm.reg_mut(10, constants::STDIN_FILENO); // x10 = a0
        vm.reg_mut(11, 0); // x11 = a1 -> memory addr 0
        vm.reg_mut(12, 3); // x12 = a2 -> read 3 bytes

        // place ecall function (ECALL_STD_INPUT) in x17 (a7)
        vm.reg_mut(17, constants::ECALL_STD_INPUT as u64);

        // execute ecall (standard encoding 0x0000_0073)
        let insn = 0x0000_0073;
        vm.execute_instruction(decode(insn), false);

        // check bytes written to guest memory and return value in a0
        assert_eq!(vm.read_bytes(0, 3), b"hel".to_vec());
        assert_eq!(vm.reg(10), 3);
    }

    #[test]
    fn test_ecall_stdout() {
        let mut vm = VM::<NoopTracer>::init();

        // Write "world" into guest memory at address 0
        vm.write_bytes(0, b"world");

        // a0 = fd (stdout), a1 = guest ptr, a2 = len
        vm.reg_mut(10, constants::STDOUT_FILENO); // x10 = a0
        vm.reg_mut(11, 0); // x11 = a1 -> memory addr 0
        vm.reg_mut(12, 5); // x12 = a2 -> length

        // place ecall function (ECALL_STD_OUTPUT) in x17 (a7)
        vm.reg_mut(17, constants::ECALL_STD_OUTPUT as u64);

        // execute ecall
        let insn = 0x0000_0073;
        vm.execute_instruction(decode(insn), false);

        // stdout handler returns length read in a0
        assert_eq!(vm.reg(10), 5);
    }
}
