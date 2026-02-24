use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrType {
    I1,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ValueId(pub(crate) u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockId(pub(crate) u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reg {
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    X26,
    X27,
    X28,
    X29,
    X30,
    X31,
    F0,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
}

#[derive(Clone, Debug)]
pub enum PureOp {
    ConstI64(i64),
    ConstF32(u32),
    Add(ValueId, ValueId),
    Sub(ValueId, ValueId),
    Mul(ValueId, ValueId),
    Mulh(ValueId, ValueId),
    Mulhu(ValueId, ValueId),
    Mulhsu(ValueId, ValueId),
    Div(ValueId, ValueId),
    Divu(ValueId, ValueId),
    Rem(ValueId, ValueId),
    Remu(ValueId, ValueId),
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
    // Floating point operations
    Fadd(ValueId, ValueId),
    Fsub(ValueId, ValueId),
    Fmul(ValueId, ValueId),
    Fdiv(ValueId, ValueId),
    Fsqrt(ValueId),
    Fmin(ValueId, ValueId),
    Fmax(ValueId, ValueId),
    Feq(ValueId, ValueId),
    Flt(ValueId, ValueId),
    Fle(ValueId, ValueId),
    Fsgnj(ValueId, ValueId),
    Fsgnjn(ValueId, ValueId),
    Fsgnjx(ValueId, ValueId),
    FcvtF32I32(ValueId),
    FcvtF32I64(ValueId),
    FcvtF32U32(ValueId),
    FcvtF32U64(ValueId),
    FcvtI32F32(ValueId),
    FcvtI64F32(ValueId),
    FcvtU32F32(ValueId),
    FcvtU64F32(ValueId),
    FmvF32(ValueId),
    FmvI32(ValueId),
}

#[derive(Clone, Debug)]
pub enum EffectOp {
    GetReg { dst: ValueId, reg: Reg },
    SetReg { reg: Reg, val: ValueId },
    GetCsr { dst: ValueId, csr: u32 },
    SetCsr { csr: u32, val: ValueId },
    GetPc { dst: ValueId },
    SetPc { val: ValueId },
    Load8s { dst: ValueId, addr: ValueId },
    Load8u { dst: ValueId, addr: ValueId },
    Load16s { dst: ValueId, addr: ValueId },
    Load16u { dst: ValueId, addr: ValueId },
    Load32s { dst: ValueId, addr: ValueId },
    Load32u { dst: ValueId, addr: ValueId },
    Load64 { dst: ValueId, addr: ValueId },
    Store8 { addr: ValueId, val: ValueId },
    Store16 { addr: ValueId, val: ValueId },
    Store32 { addr: ValueId, val: ValueId },
    Store64 { addr: ValueId, val: ValueId },
    // Floating point load/store
    LoadF32 { dst: ValueId, addr: ValueId },
    StoreF32 { addr: ValueId, val: ValueId },
    Ecall,
    Ebreak,
}

#[derive(Clone, Debug)]
pub enum Op {
    Pure { dst: ValueId, op: PureOp },
    Effect(EffectOp),
}

#[derive(Clone, Debug)]
pub enum Terminator {
    Br {
        target: BlockId,
        args: Vec<ValueId>,
    },
    Cbr {
        cond: ValueId,
        t: BlockId,
        f: BlockId,
        t_args: Vec<ValueId>,
        f_args: Vec<ValueId>,
    },
    Ret,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub args: Vec<ValueId>,
    pub ops: Vec<Op>,
    pub term: Option<Terminator>,
}

#[derive(Clone, Debug)]
pub struct IrFunction {
    pub blocks: Vec<Block>,
    pub value_types: Vec<IrType>,
}

impl IrFunction {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            value_types: Vec::new(),
        }
    }

    pub fn value_type(&self, v: ValueId) -> IrType {
        self.value_types[v.0 as usize]
    }
}

impl fmt::Display for IrFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, block) in self.blocks.iter().enumerate() {
            write!(f, "bb{}(", i)?;
            for (idx, arg) in block.args.iter().enumerate() {
                if idx > 0 {
                    write!(f, ", ")?;
                }
                let ty = self.value_type(*arg);
                write!(f, "v{}: {:?}", arg.0, ty)?;
            }
            writeln!(f, "):")?;

            for op in &block.ops {
                match op {
                    Op::Pure { dst, op } => {
                        writeln!(f, "  v{} = {:?}", dst.0, op)?;
                    }
                    Op::Effect(effect) => {
                        writeln!(f, "  {:?}", effect)?;
                    }
                }
            }

            match &block.term {
                Some(term) => writeln!(f, "  {:?}", term)?,
                None => writeln!(f, "  <no terminator>")?,
            }
        }
        Ok(())
    }
}
