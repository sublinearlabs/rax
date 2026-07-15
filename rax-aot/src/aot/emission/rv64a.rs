use crate::aot::emit_asm;

use crate::aot::{
    emission::rv64i::{emit_ld, emit_lw},
    instruction_context::InstructionContextBuilder,
    registers::RiscvRegister,
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
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    _rs2: RiscvRegister,
) {
    // NOTE: this delegation is only safe for single core
    emit_lw(translator, rd, rs1, 0);
}

/// RV64 `lr.d`: load-reserved doubleword.
/// rd <- M[rs1][63:0]; reserve for later store-conditional
pub(super) fn emit_lrd(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    _rs2: RiscvRegister,
) {
    // NOTE: this delegation is only safe for single core
    emit_ld(translator, rd, rs1, 0);
}

/// RV64 `sc.w`: store-conditional word.
/// if reservation held then M[rs1] <- rs2[31:0], rd <- 0 else rd <- 1
pub(super) fn emit_scw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let addr_id = if rs1.is_zero() {
        addr_temp = translator.temp_pool.allocate().unwrap();
        emit_asm!(translator ; xor Rq(addr_temp.id()), Rq(addr_temp.id()));
        addr_temp.id()
    } else {
        rs1.id()
    };

    if rs2.is_zero() {
        emit_asm!(translator ; mov DWORD [Rq(addr_id)], 0);
    } else {
        emit_asm!(translator ; mov DWORD [Rq(addr_id)], Rd(rs2.id()));
    }

    if rd.is_zero() {
        ctx.discard_zero_output(translator);
    } else {
        emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
    }
}

/// RV64 `sc.d`: store-conditional doubleword.
/// if reservation held then M[rs1] <- rs2[63:0], rd <- 0 else rd <- 1
pub(super) fn emit_scd(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    let addr_temp;
    let addr_id = if rs1.is_zero() {
        addr_temp = translator.temp_pool.allocate().unwrap();
        emit_asm!(translator ; xor Rq(addr_temp.id()), Rq(addr_temp.id()));
        addr_temp.id()
    } else {
        rs1.id()
    };

    if rs2.is_zero() {
        emit_asm!(translator ; mov QWORD [Rq(addr_id)], 0);
    } else {
        emit_asm!(translator ; mov QWORD [Rq(addr_id)], Rq(rs2.id()));
    }

    if rd.is_zero() {
        ctx.discard_zero_output(translator);
    } else {
        emit_asm!(translator ; xor Rq(rd.id()), Rq(rd.id()));
        ctx.write_back(translator);
    }
}

/// RV64 `amoadd.w`: atomic add word.
/// rd <- M[rs1]; M[rs1] <- M[rs1] + rs2[31:0]
pub(super) fn emit_amoaddw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Word, AmoOp::Add);
}

/// RV64 `amoswap.w`: atomic swap word.
/// rd <- M[rs1]; M[rs1] <- rs2[31:0]
pub(super) fn emit_amoswapw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Word, AmoOp::Swap);
}

/// RV64 `amoxor.w`: atomic XOR word.
/// rd <- M[rs1]; M[rs1] <- M[rs1] ^ rs2[31:0]
pub(super) fn emit_amoxorw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Word, AmoOp::Xor);
}

/// RV64 `amoand.w`: atomic AND word.
/// rd <- M[rs1]; M[rs1] <- M[rs1] & rs2[31:0]
pub(super) fn emit_amoandw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Word, AmoOp::And);
}

/// RV64 `amoadd.d`: atomic add doubleword.
/// rd <- M[rs1]; M[rs1] <- M[rs1] + rs2[63:0]
pub(super) fn emit_amoaddd(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Double, AmoOp::Add);
}

/// RV64 `amoswap.d`: atomic swap doubleword.
/// rd <- M[rs1]; M[rs1] <- rs2[63:0]
pub(super) fn emit_amoswapd(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Double, AmoOp::Swap);
}

/// RV64 `amoxor.d`: atomic XOR doubleword.
/// rd <- M[rs1]; M[rs1] <- M[rs1] ^ rs2[63:0]
pub(super) fn emit_amoxord(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Double, AmoOp::Xor);
}

/// RV64 `amoand.d`: atomic AND doubleword.
/// rd <- M[rs1]; M[rs1] <- M[rs1] & rs2[63:0]
pub(super) fn emit_amoandd(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Double, AmoOp::And);
}

/// RV64 `amoor.w`: atomic OR word.
/// rd <- M[rs1]; M[rs1] <- M[rs1] | rs2[31:0]
pub(super) fn emit_amoorw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Word, AmoOp::Or);
}

/// RV64 `amomin.w`: atomic signed min word.
/// rd <- M[rs1]; M[rs1] <- min_s(M[rs1], rs2[31:0])
pub(super) fn emit_amominw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Word, AmoOp::Min);
}

/// RV64 `amomax.w`: atomic signed max word.
/// rd <- M[rs1]; M[rs1] <- max_s(M[rs1], rs2[31:0])
pub(super) fn emit_amomaxw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Word, AmoOp::Max);
}

/// RV64 `amominu.w`: atomic unsigned min word.
/// rd <- M[rs1]; M[rs1] <- min_u(M[rs1], rs2[31:0])
pub(super) fn emit_amominuw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Word, AmoOp::Minu);
}

/// RV64 `amomaxu.w`: atomic unsigned max word.
/// rd <- M[rs1]; M[rs1] <- max_u(M[rs1], rs2[31:0])
pub(super) fn emit_amomaxuw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Word, AmoOp::Maxu);
}

/// RV64 `amoor.d`: atomic OR doubleword.
/// rd <- M[rs1]; M[rs1] <- M[rs1] | rs2[63:0]
pub(super) fn emit_amoodd(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Double, AmoOp::Or);
}

/// RV64 `amomin.d`: atomic signed min doubleword.
/// rd <- M[rs1]; M[rs1] <- min_s(M[rs1], rs2[63:0])
pub(super) fn emit_amomind(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Double, AmoOp::Min);
}

/// RV64 `amomax.d`: atomic signed max doubleword.
/// rd <- M[rs1]; M[rs1] <- max_s(M[rs1], rs2[63:0])
pub(super) fn emit_amomaxd(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Double, AmoOp::Max);
}

/// RV64 `amominu.d`: atomic unsigned min doubleword.
/// rd <- M[rs1]; M[rs1] <- min_u(M[rs1], rs2[63:0])
pub(super) fn emit_amominud(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Double, AmoOp::Minu);
}

/// RV64 `amomaxu.d`: atomic unsigned max doubleword.
/// rd <- M[rs1]; M[rs1] <- max_u(M[rs1], rs2[63:0])
pub(super) fn emit_amomaxud(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
) {
    emit_amo_rmw(translator, rd, rs1, rs2, AmoWidth::Double, AmoOp::Maxu);
}

fn emit_amo_rmw(
    translator: &Translator,
    rd: RiscvRegister,
    rs1: RiscvRegister,
    rs2: RiscvRegister,
    width: AmoWidth,
    op: AmoOp,
) {
    let ctx = InstructionContextBuilder::<2, 0>::new()
        .set_inputs([rs1, rs2])
        .set_output(rd)
        .build(translator);

    let [rs1, rs2] = ctx.inputs();
    let rd = ctx.output();

    if rd.is_zero() && rs2.is_zero() {
        ctx.discard_zero_output(translator);
        return;
    }

    let addr_temp;
    let addr_id = if rs1.is_zero() {
        addr_temp = translator.temp_pool.allocate().unwrap();
        emit_asm!(translator ; xor Rq(addr_temp.id()), Rq(addr_temp.id()));
        addr_temp.id()
    } else if !rd.is_zero() && rd.id() == rs1.id() {
        // Save the address before loading into rd, which aliases rs1 here.
        addr_temp = translator.temp_pool.allocate().unwrap();
        emit_asm!(translator ; mov Rq(addr_temp.id()), Rq(rs1.id()));
        addr_temp.id()
    } else {
        rs1.id()
    };

    if rs2.is_zero() {
        match width {
            AmoWidth::Word => {
                emit_asm!(translator ; movsxd Rq(rd.id()), DWORD [Rq(addr_id)]);
            }
            AmoWidth::Double => {
                emit_asm!(translator ; mov Rq(rd.id()), QWORD [Rq(addr_id)]);
            }
        }
        ctx.write_back(translator);
        return;
    }

    let rs2_temp;
    let rs2_id = if !rd.is_zero() && rd.id() == rs2.id() {
        rs2_temp = translator.temp_pool.allocate().unwrap();
        emit_asm!(translator ; mov Rq(rs2_temp.id()), Rq(rs2.id()));
        rs2_temp.id()
    } else {
        rs2.id()
    };

    let scratch = translator.temp_pool.allocate().unwrap();
    match (width, rd.is_zero()) {
        (AmoWidth::Word, true) => {
            emit_asm!(translator ; movsxd Rq(scratch.id()), DWORD [Rq(addr_id)]);
        }
        (AmoWidth::Word, false) => {
            emit_asm!(translator ; movsxd Rq(rd.id()), DWORD [Rq(addr_id)]);
            emit_asm!(translator ; mov Rd(scratch.id()), Rd(rd.id()));
        }
        (AmoWidth::Double, true) => {
            emit_asm!(translator ; mov Rq(scratch.id()), QWORD [Rq(addr_id)]);
        }
        (AmoWidth::Double, false) => {
            emit_asm!(translator ; mov Rq(rd.id()), QWORD [Rq(addr_id)]);
            emit_asm!(translator ; mov Rq(scratch.id()), Rq(rd.id()));
        }
    }

    match (width, op) {
        (AmoWidth::Word, AmoOp::Swap) => {
            emit_asm!(translator ; mov Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Add) => {
            emit_asm!(translator ; add Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Xor) => {
            emit_asm!(translator ; xor Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::And) => {
            emit_asm!(translator ; and Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Or) => {
            emit_asm!(translator ; or Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Min) => {
            emit_asm!(translator ; cmp Rd(scratch.id()), Rd(rs2_id));
            emit_asm!(translator ; cmovg Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Max) => {
            emit_asm!(translator ; cmp Rd(scratch.id()), Rd(rs2_id));
            emit_asm!(translator ; cmovl Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Minu) => {
            emit_asm!(translator ; cmp Rd(scratch.id()), Rd(rs2_id));
            emit_asm!(translator ; cmova Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Word, AmoOp::Maxu) => {
            emit_asm!(translator ; cmp Rd(scratch.id()), Rd(rs2_id));
            emit_asm!(translator ; cmovb Rd(scratch.id()), Rd(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Swap) => {
            emit_asm!(translator ; mov Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Add) => {
            emit_asm!(translator ; add Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Xor) => {
            emit_asm!(translator ; xor Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::And) => {
            emit_asm!(translator ; and Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Or) => {
            emit_asm!(translator ; or Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Min) => {
            emit_asm!(translator ; cmp Rq(scratch.id()), Rq(rs2_id));
            emit_asm!(translator ; cmovg Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Max) => {
            emit_asm!(translator ; cmp Rq(scratch.id()), Rq(rs2_id));
            emit_asm!(translator ; cmovl Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Minu) => {
            emit_asm!(translator ; cmp Rq(scratch.id()), Rq(rs2_id));
            emit_asm!(translator ; cmova Rq(scratch.id()), Rq(rs2_id));
        }
        (AmoWidth::Double, AmoOp::Maxu) => {
            emit_asm!(translator ; cmp Rq(scratch.id()), Rq(rs2_id));
            emit_asm!(translator ; cmovb Rq(scratch.id()), Rq(rs2_id));
        }
    }

    match width {
        AmoWidth::Word => {
            emit_asm!(translator ; mov DWORD [Rq(addr_id)], Rd(scratch.id()));
        }
        AmoWidth::Double => {
            emit_asm!(translator ; mov QWORD [Rq(addr_id)], Rq(scratch.id()));
        }
    }

    if rd.is_zero() {
        ctx.discard_zero_output(translator);
    } else {
        ctx.write_back(translator);
    }
}
