use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};

use crate::aot::{
    classification::{
        classify_shadow_case, classify_unary_shadow_case, classify_unary_zero_case,
        classify_zero_case, ShadowCase, UnaryShadowCase, UnaryZeroCase, ZeroCase,
    },
    instruction_context::InstructionContextBuilder,
    register_mapping::MapTarget,
    registers::{RiscvRegister, X86Gpr},
    temp_alloc::TempAllocator,
    translator::Translator,
};
use crate::decode::{Instruction, Sh, B, I, J, R, S, U};

/// RV64 `addw`: add low 32 bits, then sign-extend to 64 bits.
/// rd <- sext32((rs1[31:0] + rs2[31:0]) mod 2^32)
#[allow(unused_variables)]
pub(super) fn emit_addw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 addw emission.
}

/// RV64 `subw`: subtract low 32 bits, then sign-extend to 64 bits.
/// rd <- sext32((rs1[31:0] - rs2[31:0]) mod 2^32)
pub(super) fn emit_subw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator, temps);
    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    match classify_zero_case(&rd, &rs1, &rs2) {
        ZeroCase::RdZero => {
            // x0 is hardwired to zero, writes can be ignored
            ctx.discard_zero_output(translator);
            return;
        }

        ZeroCase::Rs1Rs2Zero => {
            // subw rd, 0, 0
            // -> rd = 0

            // zero out the rd register
            dynasm!(translator.emitter ; xor Rd(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs1Zero => {
            // subw rd, 0, rs2
            // -> rd = 0 - rs2
            // -> rd = low32(-rs2)

            if rd.id() != rs2.id() {
                dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs2.id()));
            }

            dynasm!(translator.emitter ; neg Rd(rd.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::Rs2Zero => {
            // subw rd, rs1, 0
            // -> rd = low32(rs1)

            if rd.id() != rs1.id() {
                dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs1.id()));
            }

            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ZeroCase::None => {}
    }

    match classify_shadow_case(&rd, &rs1, &rs2) {
        ShadowCase::AllEqual | ShadowCase::Rs1EqRs2 => {
            // subw rd, rd, rd
            // -> rd = 0

            dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs1 => {
            // subw rd, rd, rs2
            // -> rd -= rs2

            dynasm!(translator.emitter ; sub Rd(rd.id()), Rd(rs2.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::RdEqRs2 => {
            // subw rd, rs1, rd
            // -> rd = rs1 - rd
            // neg rd
            // add rs1

            dynasm!(translator.emitter ; neg Rd(rd.id()));
            dynasm!(translator.emitter ; add Rd(rd.id()), Rd(rs1.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }

        ShadowCase::AllDistinct => {
            // subw rd, rs1, rs2

            dynasm!(translator.emitter ; mov Rd(rd.id()), Rd(rs1.id()));
            dynasm!(translator.emitter ; sub Rd(rd.id()), Rd(rs2.id()));
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), Rd(rd.id()));
            ctx.write_back(translator);
            return;
        }
    }
}

/// RV64 `sllw`: logical left shift low word, then sign-extend to 64 bits.
/// rd <- sext32(rs1[31:0] << (rs2 & 0x1f))
#[allow(unused_variables)]
pub(super) fn emit_sllw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 sllw emission.
}

/// RV64 `srlw`: logical right shift low word, then sign-extend to 64 bits.
/// rd <- sext32(rs1[31:0] >> (rs2 & 0x1f))
#[allow(unused_variables)]
pub(super) fn emit_srlw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 srlw emission.
}

/// RV64 `sraw`: arithmetic right shift low word, then sign-extend to 64 bits.
/// rd <- sext32(signed(rs1[31:0]) >> (rs2 & 0x1f))
#[allow(unused_variables)]
pub(super) fn emit_sraw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 sraw emission.
}

/// RV64 `sltu`: set if less than, unsigned.
/// rd <- unsigned(rs1) < unsigned(rs2) ? 1 : 0
#[allow(unused_variables)]
pub(super) fn emit_sltu(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    // TODO: implement RV64 sltu emission.
}
