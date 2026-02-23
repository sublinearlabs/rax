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
pub struct ValueId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockId(u32);

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
    GetReg { dst: ValueId, reg: Reg },
    SetReg { reg: Reg, val: ValueId },
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

    fn bin_i64(&mut self, ctor: fn(ValueId, ValueId) -> PureOp, a: ValueId, b: ValueId) -> ValueId {
        self.expect_type(a, IrType::I64);
        self.expect_type(b, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Pure {
            dst,
            op: ctor(a, b),
        });
        dst
    }

    fn bin_cmp(&mut self, ctor: fn(ValueId, ValueId) -> PureOp, a: ValueId, b: ValueId) -> ValueId {
        self.expect_type(a, IrType::I64);
        self.expect_type(b, IrType::I64);
        let dst = self.new_value(IrType::I1);
        self.push_op(Op::Pure {
            dst,
            op: ctor(a, b),
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
