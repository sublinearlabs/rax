use crate::ir2::{ConstVal, IrBuilder, IrType, PureOp, ValueId};

impl IrBuilder {
    pub fn const_i1(&mut self, value: bool) -> ValueId {
        self.emit_pure(PureOp::Const(ConstVal::I1(value)), IrType::I1)
    }

    pub fn const_i8(&mut self, value: i8) -> ValueId {
        self.emit_pure(PureOp::Const(ConstVal::I8(value)), IrType::I8)
    }

    pub fn const_i16(&mut self, value: i16) -> ValueId {
        self.emit_pure(PureOp::Const(ConstVal::I16(value)), IrType::I16)
    }

    pub fn const_i32(&mut self, value: i32) -> ValueId {
        self.emit_pure(PureOp::Const(ConstVal::I32(value)), IrType::I32)
    }

    pub fn const_i64(&mut self, value: i64) -> ValueId {
        self.emit_pure(PureOp::Const(ConstVal::I64(value)), IrType::I64)
    }

    pub fn add(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.expect_type(a, ty);
        self.expect_type(b, ty);
        self.emit_pure(PureOp::Add(a, b), ty)
    }

    pub fn sub(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.expect_type(a, ty);
        self.expect_type(b, ty);
        self.emit_pure(PureOp::Sub(a, b), ty)
    }

    pub fn mul(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.expect_type(a, ty);
        self.expect_type(b, ty);
        self.emit_pure(PureOp::Mul(a, b), ty)
    }

    pub fn div(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.expect_type(a, ty);
        self.expect_type(b, ty);
        self.emit_pure(PureOp::Div(a, b), ty)
    }

    pub fn rem(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.expect_type(a, ty);
        self.expect_type(b, ty);
        self.emit_pure(PureOp::Rem(a, b), ty)
    }

    pub fn and(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.expect_type(a, ty);
        self.expect_type(b, ty);
        self.emit_pure(PureOp::And(a, b), ty)
    }

    pub fn or(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.expect_type(a, ty);
        self.expect_type(b, ty);
        self.emit_pure(PureOp::Or(a, b), ty)
    }

    pub fn xor(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.expect_type(a, ty);
        self.expect_type(b, ty);
        self.emit_pure(PureOp::Xor(a, b), ty)
    }

    pub fn shl(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.expect_type(a, ty);
        self.expect_type(b, ty);
        self.emit_pure(PureOp::Shl(a, b), ty)
    }

    pub fn shr(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.expect_type(a, ty);
        self.expect_type(b, ty);
        self.emit_pure(PureOp::Shr(a, b), ty)
    }

    pub fn sar(&mut self, a: ValueId, b: ValueId, ty: IrType) -> ValueId {
        self.expect_type(a, ty);
        self.expect_type(b, ty);
        self.emit_pure(PureOp::Sar(a, b), ty)
    }

    pub fn eq(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.expect_same_type(a, b);
        self.emit_pure(PureOp::Eq(a, b), IrType::I1)
    }

    pub fn ne(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.expect_same_type(a, b);
        self.emit_pure(PureOp::Ne(a, b), IrType::I1)
    }

    pub fn lt(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.expect_same_type(a, b);
        self.emit_pure(PureOp::Lt(a, b), IrType::I1)
    }

    pub fn ltu(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.expect_same_type(a, b);
        self.emit_pure(PureOp::Ltu(a, b), IrType::I1)
    }

    pub fn ge(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.expect_same_type(a, b);
        self.emit_pure(PureOp::Ge(a, b), IrType::I1)
    }

    pub fn geu(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.expect_same_type(a, b);
        self.emit_pure(PureOp::Geu(a, b), IrType::I1)
    }

    pub fn sext(&mut self, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.expect_type(v, from);
        self.emit_pure(PureOp::Sext { v, from, to }, to)
    }

    pub fn zext(&mut self, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.expect_type(v, from);
        self.emit_pure(PureOp::Zext { v, from, to }, to)
    }

    pub fn trunc(&mut self, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.expect_type(v, from);
        self.emit_pure(PureOp::Trunc { v, from, to }, to)
    }

    pub fn select(&mut self, cond: ValueId, t: ValueId, f: ValueId, ty: IrType) -> ValueId {
        self.expect_type(cond, IrType::I1);
        self.expect_type(t, ty);
        self.expect_type(f, ty);
        self.emit_pure(PureOp::Select { cond, t, f }, ty)
    }
}
