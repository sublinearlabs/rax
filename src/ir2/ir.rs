use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrType {
    I1,
    I8,
    I16,
    I32,
    I64,
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
}

#[derive(Clone, Debug)]
pub enum PureOp {
    ConstI64(i64),
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
    LoadReservedW {
        dst: ValueId,
        addr: ValueId,
    },
    LoadReservedD {
        dst: ValueId,
        addr: ValueId,
    },
    StoreConditionalW {
        dst: ValueId,
        addr: ValueId,
        val: ValueId,
    },
    StoreConditionalD {
        dst: ValueId,
        addr: ValueId,
        val: ValueId,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_formats_block_and_ops() {
        let mut func = IrFunction::new();
        func.value_types = vec![IrType::I64, IrType::I64, IrType::I1];

        let block = Block {
            args: vec![ValueId(0)],
            ops: vec![
                Op::Pure {
                    dst: ValueId(1),
                    op: PureOp::Add(ValueId(0), ValueId(0)),
                },
                Op::Effect(EffectOp::SetPc { val: ValueId(1) }),
            ],
            term: Some(Terminator::Ret),
        };

        func.blocks.push(block);

        let text = format!("{}", func);
        assert!(text.contains("bb0(v0: I64):"));
        assert!(text.contains("v1 = Add(ValueId(0), ValueId(0))"));
        assert!(text.contains("SetPc"));
        assert!(text.contains("Ret"));
    }
}
