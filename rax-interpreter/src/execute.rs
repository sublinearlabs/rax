use rax_core::decode::Instruction;
use crate::ecall::handle_ecall;
#[cfg(feature = "ext_a")]
use crate::instr_execute::a_instr::*;
use crate::instr_execute::csr_instr::*;
#[cfg(feature = "ext_d")]
use crate::instr_execute::d_instr::*;
#[cfg(feature = "ext_f")]
use crate::instr_execute::f_instr::*;
use crate::instr_execute::i_instr::*;
#[cfg(feature = "ext_m")]
use crate::instr_execute::m_instr::*;
use crate::{HostIO, VM};

impl VM {
    pub fn execute_instruction(
        &mut self,
        insn: &Instruction,
        is_compressed: bool,
        current_pc: u64,
        io: &mut HostIO,
    ) {
        match insn {
            // Register Opcodes
            Instruction::Add(insn) => execute_add(self, insn),

            Instruction::Sub(insn) => execute_sub(self, insn),

            Instruction::Xor(insn) => execute_xor(self, insn),

            Instruction::Or(insn) => execute_or(self, insn),

            Instruction::And(insn) => execute_and(self, insn),

            Instruction::Sll(insn) => execute_sll(self, insn),

            Instruction::Srl(insn) => execute_srl(self, insn),

            Instruction::Sra(insn) => execute_sra(self, insn),

            Instruction::Slt(insn) => execute_slt(self, insn),

            Instruction::Sltu(insn) => execute_sltu(self, insn),

            // Immediate Opcodes
            Instruction::Addi(insn) => execute_addi(self, insn),

            Instruction::Xori(insn) => execute_xori(self, insn),

            Instruction::Ori(insn) => execute_ori(self, insn),

            Instruction::Andi(insn) => execute_andi(self, insn),

            Instruction::Slli(insn) => execute_slli(self, insn),

            Instruction::Srli(insn) => execute_srli(self, insn),

            Instruction::Srai(insn) => execute_srai(self, insn),

            Instruction::Slti(insn) => execute_slti(self, insn),

            Instruction::Sltiu(insn) => execute_sltiu(self, insn),

            // Load Opcodes
            Instruction::Lb(insn) => execute_lb(self, insn),

            Instruction::Lbu(insn) => execute_lbu(self, insn),

            Instruction::Lh(insn) => execute_lh(self, insn),

            Instruction::Lhu(insn) => execute_lhu(self, insn),

            Instruction::Lw(insn) => execute_lw(self, insn),

            Instruction::Lwu(insn) => execute_lwu(self, insn),

            Instruction::Ld(insn) => execute_ld(self, insn),

            // Store Opcodes
            Instruction::Sb(insn) => execute_sb(self, insn),

            Instruction::Sh(insn) => execute_sh(self, insn),

            Instruction::Sw(insn) => execute_sw(self, insn),

            Instruction::Sd(insn) => execute_sd(self, insn),

            // Branch Opcodes
            Instruction::Beq(insn) => execute_beq(self, insn, current_pc),

            Instruction::Bne(insn) => execute_bne(self, insn, current_pc),

            Instruction::Blt(insn) => execute_blt(self, insn, current_pc),

            Instruction::Bltu(insn) => execute_bltu(self, insn, current_pc),

            Instruction::Bge(insn) => execute_bge(self, insn, current_pc),

            Instruction::Bgeu(insn) => execute_bgeu(self, insn, current_pc),

            // Jump opcodes
            Instruction::Jal(insn) => execute_jal(self, insn, current_pc, is_compressed),

            Instruction::Jalr(insn) => execute_jalr(self, insn, current_pc, is_compressed),

            // Lui and Auipc
            Instruction::Lui(insn) => execute_lui(self, insn),

            Instruction::Auipc(insn) => execute_auipc(self, insn, current_pc),

            // RV64I Instructions
            Instruction::Addiw(insn) => execute_addiw(self, insn),

            Instruction::Slliw(insn) => execute_slliw(self, insn),

            Instruction::Srliw(insn) => execute_srliw(self, insn),

            Instruction::Sraiw(insn) => execute_sraiw(self, insn),

            Instruction::Addw(insn) => execute_addw(self, insn),

            Instruction::Subw(insn) => execute_subw(self, insn),

            Instruction::Sllw(insn) => execute_sllw(self, insn),

            Instruction::Srlw(insn) => execute_srlw(self, insn),

            Instruction::Sraw(insn) => execute_sraw(self, insn),

            // M Extension - Multiplication
            #[cfg(feature = "ext_m")]
            Instruction::Mul(insn) => execute_mul(self, insn),

            #[cfg(feature = "ext_m")]
            Instruction::Mulh(insn) => execute_mulh(self, insn),

            #[cfg(feature = "ext_m")]
            Instruction::Mulhsu(insn) => execute_mulhsu(self, insn),

            #[cfg(feature = "ext_m")]
            Instruction::Mulhu(insn) => execute_mulhu(self, insn),

            #[cfg(feature = "ext_m")]
            Instruction::Mulw(insn) => execute_mulw(self, insn),

            // M Extension - Division
            #[cfg(feature = "ext_m")]
            Instruction::Div(insn) => execute_div(self, insn),

            #[cfg(feature = "ext_m")]
            Instruction::Divu(insn) => execute_divu(self, insn),

            #[cfg(feature = "ext_m")]
            Instruction::Rem(insn) => execute_rem(self, insn),

            #[cfg(feature = "ext_m")]
            Instruction::Remu(insn) => execute_remu(self, insn),

            #[cfg(feature = "ext_m")]
            Instruction::Divw(insn) => execute_divw(self, insn),

            #[cfg(feature = "ext_m")]
            Instruction::Divuw(insn) => execute_divuw(self, insn),

            #[cfg(feature = "ext_m")]
            Instruction::Remw(insn) => execute_remw(self, insn),

            #[cfg(feature = "ext_m")]
            Instruction::Remuw(insn) => execute_remuw(self, insn),

            // A Extension - Load Reserved / Store Conditional
            #[cfg(feature = "ext_a")]
            Instruction::LrW(insn) => execute_lr_w(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::LrD(insn) => execute_lr_d(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::ScW(insn) => execute_sc_w(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::ScD(insn) => execute_sc_d(self, insn),

            // A Extension - Atomic Memory Operations (Word)
            #[cfg(feature = "ext_a")]
            Instruction::AmoSwapW(insn) => execute_amo_swap_w(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoAddW(insn) => execute_amo_add_w(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoXorW(insn) => execute_amo_xor_w(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoAndW(insn) => execute_amo_and_w(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoOrW(insn) => execute_amo_or_w(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoMinW(insn) => execute_amo_min_w(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoMaxW(insn) => execute_amo_max_w(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoMinuW(insn) => execute_amo_minu_w(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoMaxuW(insn) => execute_amo_maxu_w(self, insn),

            // A Extension - Atomic Memory Operations (Double)
            #[cfg(feature = "ext_a")]
            Instruction::AmoSwapD(insn) => execute_amo_swap_d(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoAddD(insn) => execute_amo_add_d(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoXorD(insn) => execute_amo_xor_d(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoAndD(insn) => execute_amo_and_d(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoOrD(insn) => execute_amo_or_d(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoMinD(insn) => execute_amo_min_d(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoMaxD(insn) => execute_amo_max_d(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoMinuD(insn) => execute_amo_minu_d(self, insn),

            #[cfg(feature = "ext_a")]
            Instruction::AmoMaxuD(insn) => execute_amo_maxu_d(self, insn),

            // F instructions
            #[cfg(feature = "ext_f")]
            Instruction::FmaddS(insn) => execute_fmadd_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FmsubS(insn) => execute_fmsub_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FnmsubS(insn) => execute_fnmsub_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FnmaddS(insn) => execute_fnmadd_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FaddS(insn) => execute_fadd_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FsubS(insn) => execute_fsub_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FmulS(insn) => execute_fmul_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FdivS(insn) => execute_fdiv_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FsqrtS(insn) => execute_fsqrt_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FsgnjS(insn) => execute_fsgnj_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FsgnjnS(insn) => execute_fsgnjn_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FsgnjxS(insn) => execute_fsgnjx_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FminS(insn) => execute_fmin_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FmaxS(insn) => execute_fmax_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FcvtWS(insn) => execute_fcvt_ws(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FcvtWuS(insn) => execute_fcvt_wu_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FmvXW(insn) => execute_fmv_xw(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FeqS(insn) => execute_feq_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FltS(insn) => execute_flt_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FleS(insn) => execute_fle_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FclassS(insn) => execute_fclass_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FcvtSW(insn) => execute_fcvt_sw(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FcvtSWu(insn) => execute_fcvt_swu(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FmvWX(insn) => execute_fmv_wx(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FmaddD(insn) => execute_fmadd_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FmsubD(insn) => execute_fmsub_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FnmsubD(insn) => execute_fnmsub_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FnmaddD(insn) => execute_fnmadd_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FaddD(insn) => execute_fadd_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FsubD(insn) => execute_fsub_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FmulD(insn) => execute_fmul_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FdivD(insn) => execute_fdiv_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FsqrtD(insn) => execute_fsqrt_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FsgnjD(insn) => execute_fsgnj_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FsgnjnD(insn) => execute_fsgnjn_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FsgnjxD(insn) => execute_fsgnjx_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FminD(insn) => execute_fmin_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FmaxD(insn) => execute_fmax_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FcvtSD(insn) => execute_fcvt_sd(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FcvtDS(insn) => execute_fcvt_ds(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FeqD(insn) => execute_feq_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FltD(insn) => execute_flt_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FleD(insn) => execute_fle_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FclassD(insn) => execute_fclass_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FcvtWD(insn) => execute_fcvt_wd(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FcvtWuD(insn) => execute_fcvt_wu_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FcvtDW(insn) => execute_fcvt_dw(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FcvtDWu(insn) => execute_fcvt_dwu(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::Flw(insn) => execute_flw(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::Fsw(insn) => execute_fsw(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::Fld(insn) => execute_fld(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::Fsd(insn) => execute_fsd(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FcvtLS(insn) => execute_fcvt_ls(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FcvtLuS(insn) => execute_fcvt_lu_s(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FcvtSL(insn) => execute_fcvt_sl(self, insn),

            #[cfg(feature = "ext_f")]
            Instruction::FcvtSLu(insn) => execute_fcvt_slu(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FcvtLD(insn) => execute_fcvt_ld(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FcvtLuD(insn) => execute_fcvt_lu_d(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FmvXD(insn) => execute_fmv_xd(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FcvtDL(insn) => execute_fcvt_dl(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FcvtDLu(insn) => execute_fcvt_dlu(self, insn),

            #[cfg(feature = "ext_d")]
            Instruction::FmvDX(insn) => execute_fmv_dx(self, insn),

            // CSR instructions
            Instruction::Csrrw(insn) => execute_csrrw(self, insn),

            Instruction::Csrrs(insn) => execute_csrrs(self, insn),

            Instruction::Csrrc(insn) => execute_csrrc(self, insn),

            Instruction::Csrrwi(insn) => execute_csrrwi(self, insn),

            Instruction::Csrrsi(insn) => execute_csrrsi(self, insn),

            Instruction::Csrrci(insn) => execute_csrrci(self, insn),

            // System Opcodes
            Instruction::Ecall => {
                handle_ecall(self, io);
            }
            Instruction::Ebreak => {
                handle_ecall(self, io);
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use rax_core::decode::decode;
    use crate::ecall::constants;
    use crate::{HostIO, VM};

    fn run_insn(vm: &mut VM, io: &mut HostIO, insn: u32, is_compressed: bool) {
        let current_pc = vm.pc();
        let next_pc = current_pc.wrapping_add(if is_compressed { 2 } else { 4 });
        vm.set_pc(next_pc);
        let decoded = decode(insn);
        vm.execute_instruction(&decoded, is_compressed, current_pc, io);
    }

    #[test]
    fn test_add_instruction() {
        let mut vm = VM::init();
        let mut io = HostIO::new();
        vm.reg_mut(3, 12);
        vm.reg_mut(5, 32);
        // r8 = r3 + r5
        // 0x518433 = Instruction::Add(R { rd: 8, rs1: 3, rs2: 5 });
        let insn = 0x518433;
        run_insn(&mut vm, &mut io, insn, false);
        assert_eq!(vm.reg(8), 12 + 32);
    }

    #[test]
    fn test_store_byte() {
        let mut vm = VM::init();
        let mut io = HostIO::new();
        vm.reg_mut(3, 12);
        vm.reg_mut(2, 5);
        // 0x310123 = Instruction::Sb(S {rs1: 2, rs2: 3, imm: 2});
        let insn = 0x310123;
        run_insn(&mut vm, &mut io, insn, false);
        assert_eq!(vm.load_u64(7), 12);
    }

    #[test]
    fn test_store_half_word() {
        let mut vm = VM::init();
        let mut io = HostIO::new();
        vm.reg_mut(3, 64008);
        vm.reg_mut(2, 5);
        // 0x311123 = Instruction::Sh(S {rs1: 2, rs2: 3, imm: 2});
        let insn = 0x311123;
        run_insn(&mut vm, &mut io, insn, false);
        assert_eq!(vm.load_u64(7), 64008);
        assert_eq!(vm.load_u64(8), 250);
    }

    #[test]
    fn test_store_word() {
        let mut vm = VM::init();
        let mut io = HostIO::new();
        vm.reg_mut(3, 2299561908);
        vm.reg_mut(2, 5);
        // 0x312123 = Instruction::Sw(S { rs1: 2, rs2: 3, imm: 2 });
        let insn = 0x312123;
        run_insn(&mut vm, &mut io, insn, false);
        assert_eq!(vm.load_u64(7), 2299561908);
        assert_eq!(vm.load_u64(8), 8982663);
        assert_eq!(vm.load_u64(9), 35088);
    }

    #[test]
    fn test_store_double_word() {
        let mut vm = VM::init();
        let mut io = HostIO::new();
        vm.reg_mut(3, 1234567898765432123);
        vm.reg_mut(2, 5);
        // 0x313123 = Instruction::Sd(S { rs1: 2, rs2: 3, imm: 2 });
        let insn = 0x313123;
        run_insn(&mut vm, &mut io, insn, false);
        assert_eq!(vm.load_u64(7), 1234567898765432123);
        assert_eq!(vm.load_u64(8), 4822530854552469);
        assert_eq!(vm.load_u64(9), 18838011150595);
        assert_eq!(vm.load_u64(10), 73585981057);
        assert_eq!(vm.load_u64(11), 287445238);
        assert_eq!(vm.load_u64(12), 1122832);
    }

    #[test]
    fn test_jal_opcode() {
        let mut vm = VM::init();
        let mut io = HostIO::new();
        vm.set_pc(8);
        // 0xC001EF = Instruction::Jal(J { rd: 3, imm: 12 });
        let insn = 0xC001EF;
        run_insn(&mut vm, &mut io, insn, false);
        assert_eq!(vm.reg(3), 12);
        assert_eq!(vm.pc(), 20);
    }

    #[test]
    fn test_jalr_opcode() {
        let mut vm = VM::init();
        let mut io = HostIO::new();
        vm.set_pc(8);
        vm.reg_mut(5, 6);
        // 0x9281E7 = Instruction::Jalr(I {rs1: 5, rd: 3, imm: 9});
        let insn = 0x9281E7;
        run_insn(&mut vm, &mut io, insn, false);
        assert_eq!(vm.reg(3), 12);
        assert_eq!(vm.pc(), 15);
    }

    #[test]
    fn test_ecall_stdin() {
        let mut vm = VM::init();
        let mut io = HostIO::new();

        // Prepare an input stream "hello"
        io.set_input_stream(b"hello".to_vec());

        // a0 = fd (stdin), a1 = guest ptr, a2 = len
        vm.reg_mut(10, constants::STDIN_FILENO); // x10 = a0
        vm.reg_mut(11, 0); // x11 = a1 -> memory addr 0
        vm.reg_mut(12, 3); // x12 = a2 -> read 3 bytes

        // place ecall function (ECALL_STD_INPUT) in x17 (a7)
        vm.reg_mut(17, constants::ECALL_STD_INPUT as u64);

        // execute ecall (standard encoding 0x0000_0073)
        let insn = 0x0000_0073;
        run_insn(&mut vm, &mut io, insn, false);

        // check bytes written to guest memory and return value in a0
        assert_eq!(vm.read_bytes(0, 3), b"hel".to_vec());
        assert_eq!(vm.reg(10), 3);
    }

    #[test]
    fn test_ecall_stdout() {
        let mut vm = VM::init();
        let mut io = HostIO::new();

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
        run_insn(&mut vm, &mut io, insn, false);

        // stdout handler returns length read in a0
        assert_eq!(vm.reg(10), 5);
    }
}
