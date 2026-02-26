use cranelift_codegen::ir::{condcodes::IntCC, types, FuncRef, InstBuilder, Value};
use cranelift_frontend::FunctionBuilder;

use crate::ir::{AtomicRmwOp, ConstVal, IrType, PureOp};

use super::HelperFuncRefs;

pub fn lower_pure(
    builder: &mut FunctionBuilder,
    op: &PureOp,
    values: &[Option<Value>],
    types: &[IrType],
    helpers: &HelperFuncRefs,
) -> Value {
    match op {
        PureOp::Const(c) => builder.ins().iconst(types::I64, const_value(c) as i64),
        PureOp::Add(a, b) => {
            let a = value_for(values, *a);
            let b = value_for(values, *b);
            builder.ins().iadd(a, b)
        }
        PureOp::Sub(a, b) => {
            let a = value_for(values, *a);
            let b = value_for(values, *b);
            builder.ins().isub(a, b)
        }
        PureOp::Mul(a, b) => {
            let a = value_for(values, *a);
            let b = value_for(values, *b);
            builder.ins().imul(a, b)
        }
        PureOp::Div(a, b) => call_divrem(builder, helpers.div_s, values, types, *a, *b),
        PureOp::Divu(a, b) => call_divrem(builder, helpers.div_u, values, types, *a, *b),
        PureOp::Rem(a, b) => call_divrem(builder, helpers.rem_s, values, types, *a, *b),
        PureOp::Remu(a, b) => call_divrem(builder, helpers.rem_u, values, types, *a, *b),
        PureOp::And(a, b) => {
            let a = value_for(values, *a);
            let b = value_for(values, *b);
            builder.ins().band(a, b)
        }
        PureOp::Or(a, b) => {
            let a = value_for(values, *a);
            let b = value_for(values, *b);
            builder.ins().bor(a, b)
        }
        PureOp::Xor(a, b) => {
            let a = value_for(values, *a);
            let b = value_for(values, *b);
            builder.ins().bxor(a, b)
        }
        PureOp::Shl(a, b) => {
            let a_val = value_for(values, *a);
            let ty = types[a.0 as usize];
            let sh = shift_amount(builder, value_for(values, *b), ty);
            builder.ins().ishl(a_val, sh)
        }
        PureOp::Shr(a, b) => {
            let a_val = value_for(values, *a);
            let ty = types[a.0 as usize];
            let sh = shift_amount(builder, value_for(values, *b), ty);
            builder.ins().ushr(a_val, sh)
        }
        PureOp::Sar(a, b) => {
            let ty = types[a.0 as usize];
            let a_val = sign_extend(builder, value_for(values, *a), ty);
            let sh = shift_amount(builder, value_for(values, *b), ty);
            builder.ins().sshr(a_val, sh)
        }
        PureOp::Eq(a, b) => cmp_to_i64(builder, IntCC::Equal, values, types, *a, *b, false),
        PureOp::Ne(a, b) => cmp_to_i64(builder, IntCC::NotEqual, values, types, *a, *b, false),
        PureOp::Lt(a, b) => cmp_to_i64(builder, IntCC::SignedLessThan, values, types, *a, *b, true),
        PureOp::Ltu(a, b) => cmp_to_i64(
            builder,
            IntCC::UnsignedLessThan,
            values,
            types,
            *a,
            *b,
            false,
        ),
        PureOp::Ge(a, b) => cmp_to_i64(
            builder,
            IntCC::SignedGreaterThanOrEqual,
            values,
            types,
            *a,
            *b,
            true,
        ),
        PureOp::Geu(a, b) => cmp_to_i64(
            builder,
            IntCC::UnsignedGreaterThanOrEqual,
            values,
            types,
            *a,
            *b,
            false,
        ),
        PureOp::Sext { v, from, to } => {
            let value = value_for(values, *v);
            let masked = mask_value(builder, value, *from);
            let extended = sign_extend(builder, masked, *from);
            mask_value(builder, extended, *to)
        }
        PureOp::Zext { v, from, to } => {
            let value = value_for(values, *v);
            let masked = mask_value(builder, value, *from);
            mask_value(builder, masked, *to)
        }
        PureOp::Trunc { v, from: _, to } => {
            let value = value_for(values, *v);
            mask_value(builder, value, *to)
        }
        PureOp::Select { cond, t, f } => {
            let cond_val = value_for(values, *cond);
            let cond_is_true = builder.ins().icmp_imm(IntCC::NotEqual, cond_val, 0);
            let t_val = value_for(values, *t);
            let f_val = value_for(values, *f);
            builder.ins().select(cond_is_true, t_val, f_val)
        }
    }
}

pub fn mask_value(builder: &mut FunctionBuilder, value: Value, ty: IrType) -> Value {
    let bits = ir_type_bits(ty);
    if bits >= 64 {
        return value;
    }
    let mask = ((1u64 << bits) - 1) as i64;
    let mask_val = builder.ins().iconst(types::I64, mask);
    builder.ins().band(value, mask_val)
}

fn sign_extend(builder: &mut FunctionBuilder, value: Value, ty: IrType) -> Value {
    let bits = ir_type_bits(ty);
    if bits >= 64 {
        return value;
    }
    let shift = (64 - bits) as i64;
    let shift_val = builder.ins().iconst(types::I64, shift);
    let shifted = builder.ins().ishl(value, shift_val);
    builder.ins().sshr(shifted, shift_val)
}

fn shift_amount(builder: &mut FunctionBuilder, value: Value, ty: IrType) -> Value {
    let bits = ir_type_bits(ty) as u64;
    if bits <= 1 {
        return builder.ins().iconst(types::I64, 0);
    }
    let mask = (bits - 1) as i64;
    let mask_val = builder.ins().iconst(types::I64, mask);
    builder.ins().band(value, mask_val)
}

fn ir_type_bits(ty: IrType) -> u8 {
    match ty {
        IrType::I1 => 1,
        IrType::I8 => 8,
        IrType::I16 => 16,
        IrType::I32 => 32,
        IrType::I64 => 64,
    }
}

fn cmp_to_i64(
    builder: &mut FunctionBuilder,
    cc: IntCC,
    values: &[Option<Value>],
    types: &[IrType],
    a: crate::ir::ValueId,
    b: crate::ir::ValueId,
    signed: bool,
) -> Value {
    let ty = types[a.0 as usize];
    let lhs = value_for(values, a);
    let rhs = value_for(values, b);
    let lhs = if signed {
        sign_extend(builder, lhs, ty)
    } else {
        mask_value(builder, lhs, ty)
    };
    let rhs = if signed {
        sign_extend(builder, rhs, ty)
    } else {
        mask_value(builder, rhs, ty)
    };
    let cond = builder.ins().icmp(cc, lhs, rhs);
    bool_to_i64(builder, cond)
}

fn bool_to_i64(builder: &mut FunctionBuilder, cond: Value) -> Value {
    let one = builder.ins().iconst(types::I64, 1);
    let zero = builder.ins().iconst(types::I64, 0);
    builder.ins().select(cond, one, zero)
}

fn call_divrem(
    builder: &mut FunctionBuilder,
    helper: FuncRef,
    values: &[Option<Value>],
    types: &[IrType],
    a: crate::ir::ValueId,
    b: crate::ir::ValueId,
) -> Value {
    let ty = types[a.0 as usize];
    let bits = ir_type_bits(ty);
    let bits_val = builder.ins().iconst(types::I8, bits as i64);
    let lhs = value_for(values, a);
    let rhs = value_for(values, b);
    let call = builder.ins().call(helper, &[bits_val, lhs, rhs]);
    builder.inst_results(call)[0]
}

fn const_value(c: &ConstVal) -> u64 {
    match *c {
        ConstVal::I1(v) => {
            if v {
                1
            } else {
                0
            }
        }
        ConstVal::I8(v) => (v as i64) as u64,
        ConstVal::I16(v) => (v as i64) as u64,
        ConstVal::I32(v) => (v as i64) as u64,
        ConstVal::I64(v) => v as u64,
    }
}

fn value_for(values: &[Option<Value>], id: crate::ir::ValueId) -> Value {
    values[id.0 as usize].expect("missing value")
}

fn atomic_rmw_tag(op: AtomicRmwOp) -> u32 {
    match op {
        AtomicRmwOp::Xchg => 0,
        AtomicRmwOp::Add => 1,
        AtomicRmwOp::And => 2,
        AtomicRmwOp::Or => 3,
        AtomicRmwOp::Xor => 4,
        AtomicRmwOp::Min => 5,
        AtomicRmwOp::Max => 6,
        AtomicRmwOp::Umin => 7,
        AtomicRmwOp::Umax => 8,
    }
}

pub fn atomic_rmw_tag_value(builder: &mut FunctionBuilder, op: AtomicRmwOp) -> Value {
    let tag = atomic_rmw_tag(op) as i64;
    builder.ins().iconst(types::I32, tag)
}
