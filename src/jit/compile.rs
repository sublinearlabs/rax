use cranelift_codegen::ir::{AbiParam, Function};
use cranelift_frontend::FunctionBuilderContext;
use cranelift_jit::JITModule;
use cranelift_module::{Linkage, Module};

use crate::ir::IrFunction;
use crate::jit::jit_module::{declare_helpers, HelperFuncIds};
use crate::jit::lower::{lower_ir_function, HelperFuncRefs};
use crate::trace::NoopTracer;
use crate::{HostIO, VM};

pub type JitFn = unsafe extern "C" fn(*mut VM<NoopTracer>, *mut HostIO);

pub fn compile_ir_function(module: &mut JITModule, ir: &IrFunction) -> JitFn {
    let ptr_ty = module.isa().pointer_type();
    let mut ctx = module.make_context();
    ctx.func.signature.params.push(AbiParam::new(ptr_ty));
    ctx.func.signature.params.push(AbiParam::new(ptr_ty));
    ctx.func.signature.call_conv = module.isa().default_call_conv();

    let func_id = module
        .declare_function("ir_entry", Linkage::Local, &ctx.func.signature)
        .expect("declare function");

    let helper_ids = declare_helpers(module, ptr_ty);
    let helper_refs = build_helper_refs(module, &mut ctx.func, &helper_ids);

    let mut builder_ctx = FunctionBuilderContext::new();
    lower_ir_function(ir, &mut ctx.func, &mut builder_ctx, &helper_refs);

    module
        .define_function(func_id, &mut ctx)
        .expect("define function");
    module.clear_context(&mut ctx);
    module.finalize_definitions().expect("finalize");
    let code_ptr = module.get_finalized_function(func_id);
    unsafe { std::mem::transmute::<*const u8, JitFn>(code_ptr) }
}

fn build_helper_refs(
    module: &mut JITModule,
    func: &mut Function,
    ids: &HelperFuncIds,
) -> HelperFuncRefs {
    HelperFuncRefs {
        get_reg: module.declare_func_in_func(ids.get_reg, func),
        set_reg: module.declare_func_in_func(ids.set_reg, func),
        get_pc: module.declare_func_in_func(ids.get_pc, func),
        set_pc: module.declare_func_in_func(ids.set_pc, func),
        get_csr: module.declare_func_in_func(ids.get_csr, func),
        set_csr: module.declare_func_in_func(ids.set_csr, func),
        load_u8: module.declare_func_in_func(ids.load_u8, func),
        load_u16: module.declare_func_in_func(ids.load_u16, func),
        load_u32: module.declare_func_in_func(ids.load_u32, func),
        load_u64: module.declare_func_in_func(ids.load_u64, func),
        store_u8: module.declare_func_in_func(ids.store_u8, func),
        store_u16: module.declare_func_in_func(ids.store_u16, func),
        store_u32: module.declare_func_in_func(ids.store_u32, func),
        store_u64: module.declare_func_in_func(ids.store_u64, func),
        load_reserved_w: module.declare_func_in_func(ids.load_reserved_w, func),
        load_reserved_d: module.declare_func_in_func(ids.load_reserved_d, func),
        store_conditional_w: module.declare_func_in_func(ids.store_conditional_w, func),
        store_conditional_d: module.declare_func_in_func(ids.store_conditional_d, func),
        atomic_rmw_w: module.declare_func_in_func(ids.atomic_rmw_w, func),
        atomic_rmw_d: module.declare_func_in_func(ids.atomic_rmw_d, func),
        ecall: module.declare_func_in_func(ids.ecall, func),
        ebreak: module.declare_func_in_func(ids.ebreak, func),
        halt: module.declare_func_in_func(ids.halt, func),
        div_s: module.declare_func_in_func(ids.div_s, func),
        div_u: module.declare_func_in_func(ids.div_u, func),
        rem_s: module.declare_func_in_func(ids.rem_s, func),
        rem_u: module.declare_func_in_func(ids.rem_u, func),
    }
}
