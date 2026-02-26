use cranelift_codegen::ir::{
    BlockArg, FuncRef, Function, InstBuilder, Value, condcodes::IntCC, types,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};

use crate::ir::{IrFunction, Op, Terminator};

mod effect;
mod pure;

use effect::lower_effect;
use pure::{lower_pure, mask_value};

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
    let entry_param_count = entry_params.len();
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
        let param_offset = if block_id == 0 { entry_param_count } else { 0 };
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
