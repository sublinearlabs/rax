use crate::ir2::{IrType, Reg, ValueId};

#[derive(Clone, Debug)]
pub enum PureOp {
    Const(ConstVal),
    Add(ValueId, ValueId),
    Sub(ValueId, ValueId),
    Mul(ValueId, ValueId),
    Div(ValueId, ValueId),
    Rem(ValueId, ValueId),
    And(ValueId, ValueId),
    Or(ValueId, ValueId),
    Xor(ValueId, ValueId),
    Shl(ValueId, ValueId),
    Shr(ValueId, ValueId),
    Sar(ValueId, ValueId),
    Eq(ValueId, ValueId),
    Ne(ValueId, ValueId),
    Lt(ValueId, ValueId),
    Ltu(ValueId, ValueId),
    Ge(ValueId, ValueId),
    Geu(ValueId, ValueId),
    Sext {
        v: ValueId,
        from: IrType,
        to: IrType,
    },
    Zext {
        v: ValueId,
        from: IrType,
        to: IrType,
    },
    Trunc {
        v: ValueId,
        from: IrType,
        to: IrType,
    },
    Select {
        cond: ValueId,
        t: ValueId,
        f: ValueId,
    },
}

#[derive(Clone, Debug)]
pub enum ConstVal {
    I1(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
}

#[derive(Clone, Debug)]
pub enum EffectOp {
    GetReg {
        dst: ValueId,
        reg: Reg,
    },
    SetReg {
        reg: Reg,
        val: ValueId,
    },
    GetCsr {
        dst: ValueId,
        csr: u32,
    },
    SetCsr {
        csr: u32,
        val: ValueId,
    },
    GetPc {
        dst: ValueId,
    },
    SetPc {
        val: ValueId,
    },
    Load {
        dst: ValueId,
        addr: ValueId,
        width: MemWidth,
        signed: LoadSign,
    },
    Store {
        addr: ValueId,
        val: ValueId,
        width: MemWidth,
    },
    LoadReserved {
        dst: ValueId,
        addr: ValueId,
        width: AtomicWidth,
    },
    StoreConditional {
        dst: ValueId,
        addr: ValueId,
        val: ValueId,
        width: AtomicWidth,
    },
    AtomicRmw {
        dst: ValueId,
        addr: ValueId,
        val: ValueId,
        op: AtomicRmwOp,
        width: AtomicWidth,
    },
    Ecall,
    Ebreak,
    Halt {
        code: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomicRmwOp {
    Xchg,
    Add,
    And,
    Or,
    Xor,
    Min,
    Max,
    Umin,
    Umax,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomicWidth {
    W,
    D,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemWidth {
    W8,
    W16,
    W32,
    W64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadSign {
    Signed,
    Unsigned,
}
