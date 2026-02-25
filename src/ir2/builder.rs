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

    pub fn const_i64(&mut self, value: i64) -> ValueId {
        self.emit_pure(PureOp::ConstI64(value), IrType::I64)
    }

    pub fn add(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.emit_pure(PureOp::Add(a, b), ty)
    }

    pub fn sub(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.emit_pure(PureOp::Sub(a, b), ty)
    }

    pub fn mul(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.emit_pure(PureOp::Mul(a, b), ty)
    }

    pub fn div(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.emit_pure(PureOp::Div(a, b), ty)
    }

    pub fn rem(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.emit_pure(PureOp::Rem(a, b), ty)
    }

    pub fn and(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.emit_pure(PureOp::And(a, b), ty)
    }

    pub fn or(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.emit_pure(PureOp::Or(a, b), ty)
    }

    pub fn xor(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.emit_pure(PureOp::Xor(a, b), ty)
    }

    pub fn shl(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.emit_pure(PureOp::Shl(a, b), ty)
    }

    pub fn shr(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.emit_pure(PureOp::Shr(a, b), ty)
    }

    pub fn sar(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.emit_pure(PureOp::Sar(a, b), ty)
    }

    pub fn eq(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.emit_pure(PureOp::Eq(a, b), IrType::I1)
    }

    pub fn ne(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.emit_pure(PureOp::Ne(a, b), IrType::I1)
    }

    pub fn lt(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.emit_pure(PureOp::Lt(a, b), IrType::I1)
    }

    pub fn ltu(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.emit_pure(PureOp::Ltu(a, b), IrType::I1)
    }

    pub fn ge(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.emit_pure(PureOp::Ge(a, b), IrType::I1)
    }

    pub fn geu(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.emit_pure(PureOp::Geu(a, b), IrType::I1)
    }

    pub fn sext(&mut self, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.emit_pure(PureOp::Sext { v, from, to }, to)
    }

    pub fn zext(&mut self, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.emit_pure(PureOp::Zext { v, from, to }, to)
    }

    pub fn trunc(&mut self, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.emit_pure(PureOp::Trunc { v, from, to }, to)
    }

    pub fn select(&mut self, cond: ValueId, t: ValueId, f: ValueId, ty: IrType) -> ValueId {
        self.emit_pure(PureOp::Select { cond, t, f }, ty)
    }

    pub fn get_reg(&mut self, reg: crate::ir2::Reg) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.emit_effect(EffectOp::GetReg { dst, reg });
        dst
    }

    pub fn set_reg(&mut self, reg: crate::ir2::Reg, val: ValueId) {
        self.emit_effect(EffectOp::SetReg { reg, val });
    }

    pub fn get_pc(&mut self) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.emit_effect(EffectOp::GetPc { dst });
        dst
    }

    pub fn set_pc(&mut self, val: ValueId) {
        self.emit_effect(EffectOp::SetPc { val });
    }

    pub fn get_csr(&mut self, csr: u32) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.emit_effect(EffectOp::GetCsr { dst, csr });
        dst
    }

    pub fn set_csr(&mut self, csr: u32, val: ValueId) {
        self.emit_effect(EffectOp::SetCsr { csr, val });
    }

    pub fn load(
        &mut self,
        addr: ValueId,
        width: crate::ir2::MemWidth,
        signed: crate::ir2::LoadSign,
        ty: IrType,
    ) -> ValueId {
        let dst = self.new_value(ty);
        self.emit_effect(EffectOp::Load {
            dst,
            addr,
            width,
            signed,
        });
        dst
    }

    pub fn store(&mut self, addr: ValueId, val: ValueId, width: crate::ir2::MemWidth) {
        self.emit_effect(EffectOp::Store { addr, val, width });
    }

    pub fn load_reserved(
        &mut self,
        addr: ValueId,
        width: crate::ir2::AtomicWidth,
        ty: IrType,
    ) -> ValueId {
        let dst = self.new_value(ty);
        self.emit_effect(EffectOp::LoadReserved { dst, addr, width });
        dst
    }

    pub fn store_conditional(
        &mut self,
        addr: ValueId,
        val: ValueId,
        width: crate::ir2::AtomicWidth,
        ty: IrType,
    ) -> ValueId {
        let dst = self.new_value(ty);
        self.emit_effect(EffectOp::StoreConditional {
            dst,
            addr,
            val,
            width,
        });
        dst
    }

    pub fn atomic_rmw(
        &mut self,
        op: crate::ir2::AtomicRmwOp,
        width: crate::ir2::AtomicWidth,
        addr: ValueId,
        val: ValueId,
        ty: IrType,
    ) -> ValueId {
        let dst = self.new_value(ty);
        self.emit_effect(EffectOp::AtomicRmw {
            dst,
            addr,
            val,
            op,
            width,
        });
        dst
    }

    pub fn ecall(&mut self) {
        self.emit_effect(EffectOp::Ecall);
    }

    pub fn ebreak(&mut self) {
        self.emit_effect(EffectOp::Ebreak);
    }

    pub fn halt(&mut self, code: u64) {
        self.emit_effect(EffectOp::Halt { code });
    }

    pub fn br(&mut self, target: BlockId, args: Vec<ValueId>) {
        self.check_block_args(target, &args);
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
        self.expect_type(cond, IrType::I1);
        self.check_block_args(t, &t_args);
        self.check_block_args(f, &f_args);
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

    fn value_type(&self, v: ValueId) -> IrType {
        self.func.value_types[v.0 as usize]
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

    fn expect_type(&self, v: ValueId, ty: IrType) {
        let actual = self.value_type(v);
        if actual != ty {
            panic!("type mismatch: expected {:?}, got {:?}", ty, actual);
        }
    }

    fn check_block_args(&self, block: BlockId, args: &[ValueId]) {
        let block = &self.func.blocks[block.0 as usize];
        if block.args.len() != args.len() {
            panic!("block arg count mismatch");
        }
        for (arg, param) in args.iter().zip(block.args.iter()) {
            let expected = self.value_type(*param);
            let actual = self.value_type(*arg);
            if expected != actual {
                panic!(
                    "block arg type mismatch: expected {:?}, got {:?}",
                    expected, actual
                );
            }
        }
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
