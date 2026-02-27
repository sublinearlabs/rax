use cranelift_codegen::ir::{condcodes::IntCC, types, BlockArg, InstBuilder, Value};
use cranelift_frontend::FunctionBuilder;

use crate::ir::{AtomicRmwOp, ConstVal, IrType, PureOp};

pub fn lower_pure(
    builder: &mut FunctionBuilder,
    op: &PureOp,
    values: &[Option<Value>],
    types: &[IrType],
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
        PureOp::Div(a, b) => inline_divrem(builder, values, types, *a, *b, true, false),
        PureOp::Divu(a, b) => inline_divrem(builder, values, types, *a, *b, false, false),
        PureOp::Rem(a, b) => inline_divrem(builder, values, types, *a, *b, true, true),
        PureOp::Remu(a, b) => inline_divrem(builder, values, types, *a, *b, false, true),
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

fn inline_divrem(
    builder: &mut FunctionBuilder,
    values: &[Option<Value>],
    types: &[IrType],
    a: crate::ir::ValueId,
    b: crate::ir::ValueId,
    signed: bool,
    is_rem: bool,
) -> Value {
    let ty = types[a.0 as usize];
    let bits = ir_type_bits(ty);
    let lhs = value_for(values, a);
    let rhs = value_for(values, b);
    if signed {
        inline_divrem_signed(builder, lhs, rhs, ty, bits, is_rem)
    } else {
        inline_divrem_unsigned(builder, lhs, rhs, ty, bits, is_rem)
    }
}

fn inline_divrem_signed(
    builder: &mut FunctionBuilder,
    lhs: Value,
    rhs: Value,
    ty: IrType,
    bits: u8,
    is_rem: bool,
) -> Value {
    let lhs_masked = mask_value(builder, lhs, ty);
    let rhs_masked = mask_value(builder, rhs, ty);
    let lhs = sign_extend(builder, lhs_masked, ty);
    let rhs = sign_extend(builder, rhs_masked, ty);
    let is_zero = builder.ins().icmp_imm(IntCC::Equal, rhs, 0);

    let min_val = signed_min(bits) as i64;
    let min_const = builder.ins().iconst(types::I64, min_val);
    let is_min = builder.ins().icmp(IntCC::Equal, lhs, min_const);
    let is_neg1 = builder.ins().icmp_imm(IntCC::Equal, rhs, -1);
    let is_overflow = builder.ins().band(is_min, is_neg1);

    let result_block = builder.create_block();
    builder.append_block_param(result_block, types::I64);
    let zero_block = builder.create_block();
    let check_block = builder.create_block();
    let overflow_block = builder.create_block();
    let normal_block = builder.create_block();

    builder
        .ins()
        .brif(is_zero, zero_block, &[], check_block, &[]);

    builder.switch_to_block(zero_block);
    let zero_val = if is_rem {
        mask_value(builder, lhs, ty)
    } else {
        builder.ins().iconst(types::I64, mask_bits(bits) as i64)
    };
    builder
        .ins()
        .jump(result_block, &[BlockArg::Value(zero_val)]);
    builder.seal_block(zero_block);

    builder.switch_to_block(check_block);
    builder
        .ins()
        .brif(is_overflow, overflow_block, &[], normal_block, &[]);
    builder.seal_block(check_block);

    builder.switch_to_block(overflow_block);
    let overflow_val = if is_rem {
        builder.ins().iconst(types::I64, 0)
    } else {
        mask_value(builder, min_const, ty)
    };
    builder
        .ins()
        .jump(result_block, &[BlockArg::Value(overflow_val)]);
    builder.seal_block(overflow_block);

    builder.switch_to_block(normal_block);
    let normal_val = if is_rem {
        let rem_raw = builder.ins().srem(lhs, rhs);
        mask_value(builder, rem_raw, ty)
    } else {
        let div_raw = builder.ins().sdiv(lhs, rhs);
        mask_value(builder, div_raw, ty)
    };
    builder
        .ins()
        .jump(result_block, &[BlockArg::Value(normal_val)]);
    builder.seal_block(normal_block);

    builder.switch_to_block(result_block);
    builder.seal_block(result_block);
    builder.block_params(result_block)[0]
}

fn inline_divrem_unsigned(
    builder: &mut FunctionBuilder,
    lhs: Value,
    rhs: Value,
    ty: IrType,
    bits: u8,
    is_rem: bool,
) -> Value {
    let lhs = mask_value(builder, lhs, ty);
    let rhs = mask_value(builder, rhs, ty);
    let is_zero = builder.ins().icmp_imm(IntCC::Equal, rhs, 0);
    let result_block = builder.create_block();
    builder.append_block_param(result_block, types::I64);
    let zero_block = builder.create_block();
    let normal_block = builder.create_block();

    builder
        .ins()
        .brif(is_zero, zero_block, &[], normal_block, &[]);

    builder.switch_to_block(zero_block);
    let zero_val = if is_rem {
        mask_value(builder, lhs, ty)
    } else {
        builder.ins().iconst(types::I64, mask_bits(bits) as i64)
    };
    builder
        .ins()
        .jump(result_block, &[BlockArg::Value(zero_val)]);
    builder.seal_block(zero_block);

    builder.switch_to_block(normal_block);
    let normal_val = if is_rem {
        let rem_raw = builder.ins().urem(lhs, rhs);
        mask_value(builder, rem_raw, ty)
    } else {
        let div_raw = builder.ins().udiv(lhs, rhs);
        mask_value(builder, div_raw, ty)
    };
    builder
        .ins()
        .jump(result_block, &[BlockArg::Value(normal_val)]);
    builder.seal_block(normal_block);

    builder.switch_to_block(result_block);
    builder.seal_block(result_block);
    builder.block_params(result_block)[0]
}

fn mask_bits(bits: u8) -> u64 {
    if bits >= 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    }
}

fn signed_min(bits: u8) -> i64 {
    if bits >= 64 {
        i64::MIN
    } else {
        -(1i64 << (bits - 1))
    }
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
