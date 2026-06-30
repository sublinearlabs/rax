use crate::ir::{BlockId, EffectOp, IrType, PureOp, ValueId};

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
