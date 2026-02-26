use cranelift_codegen::ir::{
    condcodes::IntCC, types, BlockArg, FuncRef, Function, InstBuilder, Value,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};

use crate::ir::{
    AtomicRmwOp, AtomicWidth, ConstVal, EffectOp, IrFunction, IrType, MemWidth, Op, PureOp,
    Terminator,
};

#[derive(Clone, Copy)]
pub struct HelperFuncRefs {
    pub get_reg: FuncRef,
    pub set_reg: FuncRef,
    pub get_pc: FuncRef,
    pub set_pc: FuncRef,
    pub get_csr: FuncRef,
    pub set_csr: FuncRef,
    pub load_u8: FuncRef,
    pub load_u16: FuncRef,
    pub load_u32: FuncRef,
    pub load_u64: FuncRef,
    pub store_u8: FuncRef,
    pub store_u16: FuncRef,
    pub store_u32: FuncRef,
    pub store_u64: FuncRef,
    pub load_reserved_w: FuncRef,
    pub load_reserved_d: FuncRef,
    pub store_conditional_w: FuncRef,
    pub store_conditional_d: FuncRef,
    pub atomic_rmw_w: FuncRef,
    pub atomic_rmw_d: FuncRef,
    pub ecall: FuncRef,
    pub ebreak: FuncRef,
    pub halt: FuncRef,
    pub div_s: FuncRef,
    pub div_u: FuncRef,
    pub rem_s: FuncRef,
    pub rem_u: FuncRef,
}

pub fn lower_ir_function(
    ir: &IrFunction,
    func: &mut Function,
    builder_ctx: &mut FunctionBuilderContext,
    helpers: &HelperFuncRefs,
) {
    let mut builder = FunctionBuilder::new(func, builder_ctx);
    let mut block_map = Vec::with_capacity(ir.blocks.len());
    for _ in &ir.blocks {
        block_map.push(builder.create_block());
    }

    let entry_block = block_map[0];
    builder.append_block_params_for_function_params(entry_block);
    let entry_params = builder.block_params(entry_block);
    let vm_value = entry_params.get(0).copied().expect("missing vm param");
    let io_value = entry_params.get(1).copied().expect("missing io param");

    for (block_id, block) in ir.blocks.iter().enumerate() {
        let clif_block = block_map[block_id];
        for _ in &block.args {
            builder.append_block_param(clif_block, types::I64);
        }
    }

    let mut value_map = vec![None; ir.value_types.len()];

    for (block_id, block) in ir.blocks.iter().enumerate() {
        let clif_block = block_map[block_id];
        builder.switch_to_block(clif_block);

        let params = builder.block_params(clif_block);
        let param_offset = if block_id == 0 { 2 } else { 0 };
        for (arg_idx, arg) in block.args.iter().enumerate() {
            value_map[arg.0 as usize] = Some(params[param_offset + arg_idx]);
        }

        for op in &block.ops {
            match op {
                Op::Pure { dst, op } => {
                    let value = lower_pure(&mut builder, op, &value_map, &ir.value_types, helpers);
                    let ty = ir.value_type(*dst);
                    let value = mask_value(&mut builder, value, ty);
                    value_map[dst.0 as usize] = Some(value);
                }
                Op::Effect(effect) => {
                    if let Some((dst, value)) = lower_effect(
                        &mut builder,
                        effect,
                        &value_map,
                        &ir.value_types,
                        helpers,
                        vm_value,
                        io_value,
                    ) {
                        let ty = ir.value_type(dst);
                        let value = mask_value(&mut builder, value, ty);
                        value_map[dst.0 as usize] = Some(value);
                    }
                }
            }
        }

        match block.term.as_ref().expect("missing terminator") {
            Terminator::Br { target, args } => {
                let mut params = Vec::with_capacity(args.len());
                for arg in args {
                    let value = value_map[arg.0 as usize].expect("missing arg value");
                    params.push(BlockArg::Value(value));
                }
                builder
                    .ins()
                    .jump(block_map[target.0 as usize], params.iter());
            }
            Terminator::Cbr {
                cond,
                t,
                f,
                t_args,
                f_args,
            } => {
                let cond_val = value_map[cond.0 as usize].expect("missing cond value");
                let cond_is_true = builder.ins().icmp_imm(IntCC::NotEqual, cond_val, 0);

                let mut t_params = Vec::with_capacity(t_args.len());
                for arg in t_args {
                    let value = value_map[arg.0 as usize].expect("missing arg value");
                    t_params.push(BlockArg::Value(value));
                }
                let mut f_params = Vec::with_capacity(f_args.len());
                for arg in f_args {
                    let value = value_map[arg.0 as usize].expect("missing arg value");
                    f_params.push(BlockArg::Value(value));
                }

                builder.ins().brif(
                    cond_is_true,
                    block_map[t.0 as usize],
                    t_params.iter(),
                    block_map[f.0 as usize],
                    f_params.iter(),
                );
            }
            Terminator::Ret => {
                builder.ins().return_(&[]);
            }
        }
    }

    for block in &block_map {
        builder.seal_block(*block);
    }

    builder.finalize();
}

fn lower_pure(
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

fn lower_effect(
    builder: &mut FunctionBuilder,
    op: &EffectOp,
    values: &[Option<Value>],
    _types: &[IrType],
    helpers: &HelperFuncRefs,
    vm_value: Value,
    io_value: Value,
) -> Option<(crate::ir::ValueId, Value)> {
    match op {
        EffectOp::GetReg { dst, reg } => {
            let reg_val = builder.ins().iconst(types::I8, reg_index(*reg) as i64);
            let call = builder.ins().call(helpers.get_reg, &[vm_value, reg_val]);
            let value = builder.inst_results(call)[0];
            Some((*dst, value))
        }
        EffectOp::SetReg { reg, val } => {
            let reg_val = builder.ins().iconst(types::I8, reg_index(*reg) as i64);
            let value = value_for(values, *val);
            builder
                .ins()
                .call(helpers.set_reg, &[vm_value, reg_val, value]);
            None
        }
        EffectOp::GetCsr { dst, csr } => {
            let csr_val = builder.ins().iconst(types::I32, *csr as i64);
            let call = builder.ins().call(helpers.get_csr, &[vm_value, csr_val]);
            let value = builder.inst_results(call)[0];
            Some((*dst, value))
        }
        EffectOp::SetCsr { csr, val } => {
            let csr_val = builder.ins().iconst(types::I32, *csr as i64);
            let value = value_for(values, *val);
            builder
                .ins()
                .call(helpers.set_csr, &[vm_value, csr_val, value]);
            None
        }
        EffectOp::GetPc { dst } => {
            let call = builder.ins().call(helpers.get_pc, &[vm_value]);
            let value = builder.inst_results(call)[0];
            Some((*dst, value))
        }
        EffectOp::SetPc { val } => {
            let value = value_for(values, *val);
            builder.ins().call(helpers.set_pc, &[vm_value, value]);
            None
        }
        EffectOp::Load { dst, addr, width } => {
            let addr_val = value_for(values, *addr);
            let helper = match width {
                MemWidth::W8 => helpers.load_u8,
                MemWidth::W16 => helpers.load_u16,
                MemWidth::W32 => helpers.load_u32,
                MemWidth::W64 => helpers.load_u64,
            };
            let call = builder.ins().call(helper, &[vm_value, addr_val]);
            let value = builder.inst_results(call)[0];
            Some((*dst, value))
        }
        EffectOp::Store { addr, val, width } => {
            let addr_val = value_for(values, *addr);
            let val = value_for(values, *val);
            let helper = match width {
                MemWidth::W8 => helpers.store_u8,
                MemWidth::W16 => helpers.store_u16,
                MemWidth::W32 => helpers.store_u32,
                MemWidth::W64 => helpers.store_u64,
            };
            builder.ins().call(helper, &[vm_value, addr_val, val]);
            None
        }
        EffectOp::LoadReserved { dst, addr, width } => {
            let addr_val = value_for(values, *addr);
            let helper = match width {
                AtomicWidth::W => helpers.load_reserved_w,
                AtomicWidth::D => helpers.load_reserved_d,
            };
            let call = builder.ins().call(helper, &[vm_value, addr_val]);
            let value = builder.inst_results(call)[0];
            Some((*dst, value))
        }
        EffectOp::StoreConditional {
            dst,
            addr,
            val,
            width,
        } => {
            let addr_val = value_for(values, *addr);
            let val = value_for(values, *val);
            let helper = match width {
                AtomicWidth::W => helpers.store_conditional_w,
                AtomicWidth::D => helpers.store_conditional_d,
            };
            let call = builder.ins().call(helper, &[vm_value, addr_val, val]);
            let value = builder.inst_results(call)[0];
            Some((*dst, value))
        }
        EffectOp::AtomicRmw {
            dst,
            addr,
            val,
            op,
            width,
        } => {
            let addr_val = value_for(values, *addr);
            let val = value_for(values, *val);
            let op_val = builder.ins().iconst(types::I32, atomic_rmw_tag(*op) as i64);
            let helper = match width {
                AtomicWidth::W => helpers.atomic_rmw_w,
                AtomicWidth::D => helpers.atomic_rmw_d,
            };
            let call = builder
                .ins()
                .call(helper, &[vm_value, addr_val, val, op_val]);
            let value = builder.inst_results(call)[0];
            Some((*dst, value))
        }
        EffectOp::Ecall => {
            builder.ins().call(helpers.ecall, &[vm_value, io_value]);
            None
        }
        EffectOp::Ebreak => {
            builder.ins().call(helpers.ebreak, &[vm_value, io_value]);
            None
        }
        EffectOp::Halt { code } => {
            let code_val = builder.ins().iconst(types::I64, *code as i64);
            builder.ins().call(helpers.halt, &[vm_value, code_val]);
            None
        }
    }
}

fn value_for(values: &[Option<Value>], id: crate::ir::ValueId) -> Value {
    values[id.0 as usize].expect("missing value")
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

fn mask_value(builder: &mut FunctionBuilder, value: Value, ty: IrType) -> Value {
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

fn reg_index(reg: crate::ir::Reg) -> u8 {
    match reg {
        crate::ir::Reg::X0 => 0,
        crate::ir::Reg::X1 => 1,
        crate::ir::Reg::X2 => 2,
        crate::ir::Reg::X3 => 3,
        crate::ir::Reg::X4 => 4,
        crate::ir::Reg::X5 => 5,
        crate::ir::Reg::X6 => 6,
        crate::ir::Reg::X7 => 7,
        crate::ir::Reg::X8 => 8,
        crate::ir::Reg::X9 => 9,
        crate::ir::Reg::X10 => 10,
        crate::ir::Reg::X11 => 11,
        crate::ir::Reg::X12 => 12,
        crate::ir::Reg::X13 => 13,
        crate::ir::Reg::X14 => 14,
        crate::ir::Reg::X15 => 15,
        crate::ir::Reg::X16 => 16,
        crate::ir::Reg::X17 => 17,
        crate::ir::Reg::X18 => 18,
        crate::ir::Reg::X19 => 19,
        crate::ir::Reg::X20 => 20,
        crate::ir::Reg::X21 => 21,
        crate::ir::Reg::X22 => 22,
        crate::ir::Reg::X23 => 23,
        crate::ir::Reg::X24 => 24,
        crate::ir::Reg::X25 => 25,
        crate::ir::Reg::X26 => 26,
        crate::ir::Reg::X27 => 27,
        crate::ir::Reg::X28 => 28,
        crate::ir::Reg::X29 => 29,
        crate::ir::Reg::X30 => 30,
        crate::ir::Reg::X31 => 31,
    }
}
