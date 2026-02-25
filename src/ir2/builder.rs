use crate::ir2::{Block, BlockId, EffectOp, IrFunction, IrType, Op, PureOp, Terminator, ValueId};

pub struct IrBuilder {
    func: IrFunction,
    current_block: Option<BlockId>,
}

impl IrBuilder {
    pub fn new() -> Self {
        Self {
            func: IrFunction::new(),
            current_block: None,
        }
    }

    pub fn finish(self) -> IrFunction {
        self.func
    }

    pub fn block_with_args(&mut self, arg_types: &[IrType]) -> BlockId {
        let mut args = Vec::with_capacity(arg_types.len());
        for &ty in arg_types {
            args.push(self.new_value(ty));
        }
        let id = BlockId(self.func.blocks.len() as u32);
        self.func.blocks.push(Block {
            args,
            ops: Vec::new(),
            term: None,
        });
        id
    }

    pub fn block(&mut self) -> BlockId {
        self.block_with_args(&[])
    }

    pub fn block_arg(&self, block: BlockId, index: usize) -> ValueId {
        self.func.blocks[block.0 as usize].args[index]
    }

    pub fn switch_to(&mut self, block: BlockId) {
        self.current_block = Some(block);
    }

    pub fn emit_pure(&mut self, op: PureOp, ty: IrType) -> ValueId {
        let dst = self.new_value(ty);
        self.push_op(Op::Pure { dst, op });
        dst
    }

    pub fn emit_effect(&mut self, op: EffectOp) {
        self.push_op(Op::Effect(op));
    }

    pub fn br(&mut self, target: BlockId, args: Vec<ValueId>) {
        self.set_term(Terminator::Br { target, args });
    }

    pub fn cbr(
        &mut self,
        cond: ValueId,
        t: BlockId,
        f: BlockId,
        t_args: Vec<ValueId>,
        f_args: Vec<ValueId>,
    ) {
        self.set_term(Terminator::Cbr {
            cond,
            t,
            f,
            t_args,
            f_args,
        });
    }

    pub fn ret(&mut self) {
        self.set_term(Terminator::Ret);
    }

    fn new_value(&mut self, ty: IrType) -> ValueId {
        let id = ValueId(self.func.value_types.len() as u32);
        self.func.value_types.push(ty);
        id
    }

    fn push_op(&mut self, op: Op) {
        let block = self.current_block.expect("no current block");
        let block = &mut self.func.blocks[block.0 as usize];
        if block.term.is_some() {
            panic!("cannot add op after terminator");
        }
        block.ops.push(op);
    }

    fn set_term(&mut self, term: Terminator) {
        let block = self.current_block.expect("no current block");
        let block = &mut self.func.blocks[block.0 as usize];
        if block.term.is_some() {
            panic!("terminator already set");
        }
        block.term = Some(term);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir2::PureOp;

    #[test]
    fn build_single_block_with_ret() {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);
        builder.emit_pure(PureOp::ConstI64(7), IrType::I64);
        builder.ret();

        let func = builder.finish();
        assert_eq!(func.blocks.len(), 1);
        assert!(func.blocks[0].term.is_some());
    }
}
