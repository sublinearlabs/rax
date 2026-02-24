use crate::ir::{
    Block, BlockId, EffectOp, IrFunction, IrType, Op, PureOp, Reg, Terminator, ValueId,
};

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

    pub fn const_i64(&mut self, v: i64) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Pure {
            dst,
            op: PureOp::ConstI64(v),
        });
        dst
    }

    pub fn add(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Add, a, b)
    }

    pub fn sub(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Sub, a, b)
    }

    pub fn and(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::And, a, b)
    }

    pub fn or(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Or, a, b)
    }

    pub fn xor(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Xor, a, b)
    }

    pub fn shl(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Shl, a, b)
    }

    pub fn shr(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Shr, a, b)
    }

    pub fn sar(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Sar, a, b)
    }

    pub fn eq(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_cmp(PureOp::Eq, a, b)
    }

    pub fn ne(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_cmp(PureOp::Ne, a, b)
    }

    pub fn lt(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_cmp(PureOp::Lt, a, b)
    }

    pub fn ltu(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_cmp(PureOp::Ltu, a, b)
    }

    pub fn ge(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_cmp(PureOp::Ge, a, b)
    }

    pub fn geu(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_cmp(PureOp::Geu, a, b)
    }

    pub fn sext(&mut self, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.un_op_typed(PureOp::Sext { v, from, to }, v, from, to)
    }

    pub fn zext(&mut self, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.un_op_typed(PureOp::Zext { v, from, to }, v, from, to)
    }

    pub fn trunc(&mut self, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.un_op_typed(PureOp::Trunc { v, from, to }, v, from, to)
    }

    pub fn select(&mut self, cond: ValueId, t: ValueId, f: ValueId) -> ValueId {
        self.expect_type(cond, IrType::I1);
        let ty = self.value_type(t);
        self.expect_type(f, ty);
        let dst = self.new_value(ty);
        self.push_op(Op::Pure {
            dst,
            op: PureOp::Select { cond, t, f },
        });
        dst
    }

    pub fn get_reg(&mut self, reg: Reg) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::GetReg { dst, reg }));
        dst
    }

    pub fn set_reg(&mut self, reg: Reg, val: ValueId) {
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::SetReg { reg, val }));
    }

    pub fn get_csr(&mut self, csr: u32) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::GetCsr { dst, csr }));
        dst
    }

    pub fn set_csr(&mut self, csr: u32, val: ValueId) {
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::SetCsr { csr, val }));
    }

    pub fn get_pc(&mut self) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::GetPc { dst }));
        dst
    }

    pub fn set_pc(&mut self, val: ValueId) {
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::SetPc { val }));
    }

    pub fn load8s(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load8s { dst, addr }));
        dst
    }

    pub fn load8u(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load8u { dst, addr }));
        dst
    }

    pub fn load16s(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load16s { dst, addr }));
        dst
    }

    pub fn load16u(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load16u { dst, addr }));
        dst
    }

    pub fn load32s(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load32s { dst, addr }));
        dst
    }

    pub fn load32u(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load32u { dst, addr }));
        dst
    }

    pub fn load64(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load64 { dst, addr }));
        dst
    }

    pub fn store8(&mut self, addr: ValueId, val: ValueId) {
        self.expect_type(addr, IrType::I64);
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::Store8 { addr, val }));
    }

    pub fn store16(&mut self, addr: ValueId, val: ValueId) {
        self.expect_type(addr, IrType::I64);
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::Store16 { addr, val }));
    }

    pub fn store32(&mut self, addr: ValueId, val: ValueId) {
        self.expect_type(addr, IrType::I64);
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::Store32 { addr, val }));
    }

    pub fn store64(&mut self, addr: ValueId, val: ValueId) {
        self.expect_type(addr, IrType::I64);
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::Store64 { addr, val }));
    }

    pub fn ecall(&mut self) {
        self.push_op(Op::Effect(EffectOp::Ecall));
    }

    pub fn ebreak(&mut self) {
        self.push_op(Op::Effect(EffectOp::Ebreak));
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

    pub fn value_type(&self, v: ValueId) -> IrType {
        self.func.value_type(v)
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

    fn expect_type(&self, v: ValueId, ty: IrType) {
        let actual = self.value_type(v);
        if actual != ty {
            panic!("type mismatch: expected {:?}, got {:?}", ty, actual);
        }
    }

    fn bin_i64(
        &mut self,
        make_op: fn(ValueId, ValueId) -> PureOp,
        a: ValueId,
        b: ValueId,
    ) -> ValueId {
        self.expect_type(a, IrType::I64);
        self.expect_type(b, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Pure {
            dst,
            op: make_op(a, b),
        });
        dst
    }

    fn bin_cmp(
        &mut self,
        make_op: fn(ValueId, ValueId) -> PureOp,
        a: ValueId,
        b: ValueId,
    ) -> ValueId {
        self.expect_type(a, IrType::I64);
        self.expect_type(b, IrType::I64);
        let dst = self.new_value(IrType::I1);
        self.push_op(Op::Pure {
            dst,
            op: make_op(a, b),
        });
        dst
    }

    fn un_op_typed(&mut self, op: PureOp, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.expect_type(v, from);
        let dst = self.new_value(to);
        self.push_op(Op::Pure { dst, op });
        dst
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
