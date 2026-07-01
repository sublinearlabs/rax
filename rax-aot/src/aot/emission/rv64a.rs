use dynasmrt::{dynasm, DynasmApi};

use crate::aot::{
    emission::rv64i::{emit_ld, emit_lw},
    instruction_context::InstructionContextBuilder,
    registers::RiscvRegister,
    temp_alloc::TempAllocator,
    translator::Translator,
};

#[derive(Clone, Copy)]
enum AmoWidth {
    Word,
    Double,
}

#[derive(Clone, Copy)]
enum AmoOp {
    Swap,
    Add,
    Xor,
    And,
    Or,
    Min,
    Max,
    Minu,
    Maxu,
}

/// RV64 `lr.w`: load-reserved word.
/// rd <- M[rs1][31:0]; reserve for later store-conditional
pub(super) fn emit_lrw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    _rs2: RiscvRegister,
) {
    // NOTE: this delegation is only safe for single core
    emit_lw(translator, temps, rd, rs1, 0);
}

/// RV64 `lr.d`: load-reserved doubleword.
/// rd <- M[rs1][63:0]; reserve for later store-conditional
pub(super) fn emit_lrd(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    _rs2: RiscvRegister,
) {
    // NOTE: this delegation is only safe for single core
    emit_ld(translator, temps, rd, rs1, 0);
}

/// RV64 `sc.w`: store-conditional word.
/// if reservation held then M[rs1] <- rs2[31:0], rd <- 0 else rd <- 1
pub(super) fn emit_scw(
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

    let addr_temp;
    let addr_id = if rs1.is_zero() {
        addr_temp = temps.allocate().unwrap();
        dynasm!(translator.emitter ; xor Rq(addr_temp.id()), Rq(addr_temp.id()));
        addr_temp.id()
    } else {
        rs1.id()
    };

    if rs2.is_zero() {
        dynasm!(translator.emitter ; mov DWORD [Rq(addr_id)], 0);
    } else {
        dynasm!(translator.emitter ; mov DWORD [Rq(addr_id)], Rd(rs2.id()));
    }

    if rd.is_zero() {
        ctx.discard_zero_output(translator);
    } else {
        dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
    }
}

/// RV64 `sc.d`: store-conditional doubleword.
/// if reservation held then M[rs1] <- rs2[63:0], rd <- 0 else rd <- 1
pub(super) fn emit_scd(
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

    let addr_temp;
    let addr_id = if rs1.is_zero() {
        addr_temp = temps.allocate().unwrap();
        dynasm!(translator.emitter ; xor Rq(addr_temp.id()), Rq(addr_temp.id()));
        addr_temp.id()
    } else {
        rs1.id()
    };

    if rs2.is_zero() {
        dynasm!(translator.emitter ; mov QWORD [Rq(addr_id)], 0);
    } else {
        dynasm!(translator.emitter ; mov QWORD [Rq(addr_id)], Rq(rs2.id()));
    }

    if rd.is_zero() {
        ctx.discard_zero_output(translator);
    } else {
        dynasm!(translator.emitter ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
    }
}

/// RV64 `amoadd.w`: atomic add word.
/// rd <- M[rs1]; M[rs1] <- M[rs1] + rs2[31:0]
pub(super) fn emit_amoaddw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, temps, rd, rs1, rs2, AmoWidth::Word, AmoOp::Add);
}

/// RV64 `amoswap.w`: atomic swap word.
/// rd <- M[rs1]; M[rs1] <- rs2[31:0]
pub(super) fn emit_amoswapw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, temps, rd, rs1, rs2, AmoWidth::Word, AmoOp::Swap);
}

/// RV64 `amoxor.w`: atomic XOR word.
/// rd <- M[rs1]; M[rs1] <- M[rs1] ^ rs2[31:0]
pub(super) fn emit_amoxorw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, temps, rd, rs1, rs2, AmoWidth::Word, AmoOp::Xor);
}

/// RV64 `amoand.w`: atomic AND word.
/// rd <- M[rs1]; M[rs1] <- M[rs1] & rs2[31:0]
pub(super) fn emit_amoandw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, temps, rd, rs1, rs2, AmoWidth::Word, AmoOp::And);
}

/// RV64 `amoadd.d`: atomic add doubleword.
/// rd <- M[rs1]; M[rs1] <- M[rs1] + rs2[63:0]
pub(super) fn emit_amoaddd(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(
        translator,
        temps,
        rd,
        rs1,
        rs2,
        AmoWidth::Double,
        AmoOp::Add,
    );
}

/// RV64 `amoswap.d`: atomic swap doubleword.
/// rd <- M[rs1]; M[rs1] <- rs2[63:0]
pub(super) fn emit_amoswapd(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(
        translator,
        temps,
        rd,
        rs1,
        rs2,
        AmoWidth::Double,
        AmoOp::Swap,
    );
}

/// RV64 `amoxor.d`: atomic XOR doubleword.
/// rd <- M[rs1]; M[rs1] <- M[rs1] ^ rs2[63:0]
pub(super) fn emit_amoxord(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(
        translator,
        temps,
        rd,
        rs1,
        rs2,
        AmoWidth::Double,
        AmoOp::Xor,
    );
}

/// RV64 `amoand.d`: atomic AND doubleword.
/// rd <- M[rs1]; M[rs1] <- M[rs1] & rs2[63:0]
pub(super) fn emit_amoandd(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(
        translator,
        temps,
        rd,
        rs1,
        rs2,
        AmoWidth::Double,
        AmoOp::And,
    );
}

/// RV64 `amoor.w`: atomic OR word.
/// rd <- M[rs1]; M[rs1] <- M[rs1] | rs2[31:0]
pub(super) fn emit_amoorw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, temps, rd, rs1, rs2, AmoWidth::Word, AmoOp::Or);
}

/// RV64 `amomin.w`: atomic signed min word.
/// rd <- M[rs1]; M[rs1] <- min_s(M[rs1], rs2[31:0])
pub(super) fn emit_amominw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, temps, rd, rs1, rs2, AmoWidth::Word, AmoOp::Min);
}

/// RV64 `amomax.w`: atomic signed max word.
/// rd <- M[rs1]; M[rs1] <- max_s(M[rs1], rs2[31:0])
pub(super) fn emit_amomaxw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, temps, rd, rs1, rs2, AmoWidth::Word, AmoOp::Max);
}

/// RV64 `amominu.w`: atomic unsigned min word.
/// rd <- M[rs1]; M[rs1] <- min_u(M[rs1], rs2[31:0])
pub(super) fn emit_amominuw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, temps, rd, rs1, rs2, AmoWidth::Word, AmoOp::Minu);
}

/// RV64 `amomaxu.w`: atomic unsigned max word.
/// rd <- M[rs1]; M[rs1] <- max_u(M[rs1], rs2[31:0])
pub(super) fn emit_amomaxuw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, temps, rd, rs1, rs2, AmoWidth::Word, AmoOp::Maxu);
}

/// RV64 `amoor.d`: atomic OR doubleword.
/// rd <- M[rs1]; M[rs1] <- M[rs1] | rs2[63:0]
pub(super) fn emit_amoodd(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, temps, rd, rs1, rs2, AmoWidth::Double, AmoOp::Or);
}

/// RV64 `amomin.d`: atomic signed min doubleword.
/// rd <- M[rs1]; M[rs1] <- min_s(M[rs1], rs2[63:0])
pub(super) fn emit_amomind(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(
        translator,
        temps,
        rd,
        rs1,
        rs2,
        AmoWidth::Double,
        AmoOp::Min,
    );
}

/// RV64 `amomax.d`: atomic signed max doubleword.
/// rd <- M[rs1]; M[rs1] <- max_s(M[rs1], rs2[63:0])
pub(super) fn emit_amomaxd(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(
        translator,
        temps,
        rd,
        rs1,
        rs2,
        AmoWidth::Double,
        AmoOp::Max,
    );
}

/// RV64 `amominu.d`: atomic unsigned min doubleword.
/// rd <- M[rs1]; M[rs1] <- min_u(M[rs1], rs2[63:0])
pub(super) fn emit_amominud(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(
        translator,
        temps,
        rd,
        rs1,
        rs2,
        AmoWidth::Double,
        AmoOp::Minu,
    );
}

/// RV64 `amomaxu.d`: atomic unsigned max doubleword.
/// rd <- M[rs1]; M[rs1] <- max_u(M[rs1], rs2[63:0])
pub(super) fn emit_amomaxud(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(
        translator,
        temps,
        rd,
        rs1,
        rs2,
        AmoWidth::Double,
        AmoOp::Maxu,
    );
}

fn emit_amo_rmw(
    translator: &mut Translator,
    temps: &TempAllocator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    width: AmoWidth,
    op: AmoOp,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator, temps);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    if rd.is_zero() && rs2.is_zero() {
        ctx.discard_zero_output(translator);
        return;
    }

    let addr_temp;
    let addr_id = if rs1.is_zero() {
        addr_temp = temps.allocate().unwrap();
        dynasm!(translator.emitter ; xor Rq(addr_temp.id()), Rq(addr_temp.id()));
        addr_temp.id()
    } else if !rd.is_zero() && rd.id() == rs1.id() {
        // Save the address before loading into rd, which aliases rs1 here.
        addr_temp = temps.allocate().unwrap();
        dynasm!(translator.emitter ; mov Rq(addr_temp.id()), Rq(rs1.id()));
        addr_temp.id()
    } else {
        rs1.id()
    };

    if rs2.is_zero() {
        match width {
            AmoWidth::Word => {
                dynasm!(translator.emitter ; movsxd Rq(rd.id()), DWORD [Rq(addr_id)]);
            }
            AmoWidth::Double => {
                dynasm!(translator.emitter ; mov Rq(rd.id()), QWORD [Rq(addr_id)]);
            }
        }
        ctx.write_back(translator);
        return;
    }

    let rs2_temp;
    let rs2_id = if !rd.is_zero() && rd.id() == rs2.id() {
        rs2_temp = temps.allocate().unwrap();
        dynasm!(translator.emitter ; mov Rq(rs2_temp.id()), Rq(rs2.id()));
        rs2_temp.id()
    } else {
        rs2.id()
    };

    let scratch = temps.allocate().unwrap();
    match (width, rd.is_zero()) {
        (AmoWidth::Word, true) => {
            dynasm!(translator.emitter ; movsxd Rq(scratch.id()), DWORD [Rq(addr_id)]);
        }
        (AmoWidth::Word, false) => {
            dynasm!(translator.emitter ; movsxd Rq(rd.id()), DWORD [Rq(addr_id)]);
            dynasm!(translator.emitter ; mov Rd(scratch.id()), Rd(rd.id()));
        }
        (AmoWidth::Double, true) => {
            dynasm!(translator.emitter ; mov Rq(scratch.id()), QWORD [Rq(addr_id)]);
        }
        (AmoWidth::Double, false) => {
            dynasm!(translator.emitter ; mov Rq(rd.id()), QWORD [Rq(addr_id)]);
            dynasm!(translator.emitter ; mov Rq(scratch.id()), Rq(rd.id()));
        }
    }

    match (width, op) {
        (AmoWidth::Word, AmoOp::Swap) => {
            dynasm!(translator.emitter ; mov Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Add) => {
            dynasm!(translator.emitter ; add Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Xor) => {
            dynasm!(translator.emitter ; xor Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::And) => {
            dynasm!(translator.emitter ; and Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Or) => {
            dynasm!(translator.emitter ; or Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Min) => {
            dynasm!(translator.emitter ; cmp Rd(scratch.id()), Rd(rs2_id));
            dynasm!(translator.emitter ; cmovg Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Max) => {
            dynasm!(translator.emitter ; cmp Rd(scratch.id()), Rd(rs2_id));
            dynasm!(translator.emitter ; cmovl Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Minu) => {
            dynasm!(translator.emitter ; cmp Rd(scratch.id()), Rd(rs2_id));
            dynasm!(translator.emitter ; cmova Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Maxu) => {
            dynasm!(translator.emitter ; cmp Rd(scratch.id()), Rd(rs2_id));
            dynasm!(translator.emitter ; cmovb Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Swap) => {
            dynasm!(translator.emitter ; mov Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Add) => {
            dynasm!(translator.emitter ; add Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Xor) => {
            dynasm!(translator.emitter ; xor Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::And) => {
            dynasm!(translator.emitter ; and Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Or) => {
            dynasm!(translator.emitter ; or Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Min) => {
            dynasm!(translator.emitter ; cmp Rq(scratch.id()), Rq(rs2_id));
            dynasm!(translator.emitter ; cmovg Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Max) => {
            dynasm!(translator.emitter ; cmp Rq(scratch.id()), Rq(rs2_id));
            dynasm!(translator.emitter ; cmovl Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Minu) => {
            dynasm!(translator.emitter ; cmp Rq(scratch.id()), Rq(rs2_id));
            dynasm!(translator.emitter ; cmova Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Maxu) => {
            dynasm!(translator.emitter ; cmp Rq(scratch.id()), Rq(rs2_id));
            dynasm!(translator.emitter ; cmovb Rq(scratch.id()), Rq(rs2_id));
        }
    }

    match width {
        AmoWidth::Word => {
            dynasm!(translator.emitter ; mov DWORD [Rq(addr_id)], Rd(scratch.id()));
        }
        AmoWidth::Double => {
            dynasm!(translator.emitter ; mov QWORD [Rq(addr_id)], Rq(scratch.id()));
        }
    }

    if rd.is_zero() {
        ctx.discard_zero_output(translator);
    } else {
        ctx.write_back(translator);
    }
}
