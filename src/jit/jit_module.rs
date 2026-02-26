use cranelift_codegen::ir::types;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};

use crate::jit::helpers;

pub struct HelperFuncIds {
    pub get_reg: FuncId,
    pub set_reg: FuncId,
    pub get_pc: FuncId,
    pub set_pc: FuncId,
    pub get_csr: FuncId,
    pub set_csr: FuncId,
    pub load_u8: FuncId,
    pub load_u16: FuncId,
    pub load_u32: FuncId,
    pub load_u64: FuncId,
    pub store_u8: FuncId,
    pub store_u16: FuncId,
    pub store_u32: FuncId,
    pub store_u64: FuncId,
    pub load_reserved_w: FuncId,
    pub load_reserved_d: FuncId,
    pub store_conditional_w: FuncId,
    pub store_conditional_d: FuncId,
    pub atomic_rmw_w: FuncId,
    pub atomic_rmw_d: FuncId,
    pub ecall: FuncId,
    pub ebreak: FuncId,
    pub halt: FuncId,
    pub div_s: FuncId,
    pub div_u: FuncId,
    pub rem_s: FuncId,
    pub rem_u: FuncId,
}

// Registers helper symbols (name -> address) for JIT linking.
pub fn build_jit_module() -> JITModule {
    let mut builder =
        JITBuilder::new(cranelift_module::default_libcall_names()).expect("jit builder");
    builder.symbol("jit_get_reg", helpers::jit_get_reg as *const u8);
    builder.symbol("jit_set_reg", helpers::jit_set_reg as *const u8);
    builder.symbol("jit_get_pc", helpers::jit_get_pc as *const u8);
    builder.symbol("jit_set_pc", helpers::jit_set_pc as *const u8);
    builder.symbol("jit_get_csr", helpers::jit_get_csr as *const u8);
    builder.symbol("jit_set_csr", helpers::jit_set_csr as *const u8);
    builder.symbol("jit_load_u8", helpers::jit_load_u8 as *const u8);
    builder.symbol("jit_load_u16", helpers::jit_load_u16 as *const u8);
    builder.symbol("jit_load_u32", helpers::jit_load_u32 as *const u8);
    builder.symbol("jit_load_u64", helpers::jit_load_u64 as *const u8);
    builder.symbol("jit_store_u8", helpers::jit_store_u8 as *const u8);
    builder.symbol("jit_store_u16", helpers::jit_store_u16 as *const u8);
    builder.symbol("jit_store_u32", helpers::jit_store_u32 as *const u8);
    builder.symbol("jit_store_u64", helpers::jit_store_u64 as *const u8);
    builder.symbol(
        "jit_load_reserved_w",
        helpers::jit_load_reserved_w as *const u8,
    );
    builder.symbol(
        "jit_load_reserved_d",
        helpers::jit_load_reserved_d as *const u8,
    );
    builder.symbol(
        "jit_store_conditional_w",
        helpers::jit_store_conditional_w as *const u8,
    );
    builder.symbol(
        "jit_store_conditional_d",
        helpers::jit_store_conditional_d as *const u8,
    );
    builder.symbol("jit_atomic_rmw_w", helpers::jit_atomic_rmw_w as *const u8);
    builder.symbol("jit_atomic_rmw_d", helpers::jit_atomic_rmw_d as *const u8);
    builder.symbol("jit_ecall", helpers::jit_ecall as *const u8);
    builder.symbol("jit_ebreak", helpers::jit_ebreak as *const u8);
    builder.symbol("jit_halt", helpers::jit_halt as *const u8);
    builder.symbol("jit_div_s", helpers::jit_div_s as *const u8);
    builder.symbol("jit_div_u", helpers::jit_div_u as *const u8);
    builder.symbol("jit_rem_s", helpers::jit_rem_s as *const u8);
    builder.symbol("jit_rem_u", helpers::jit_rem_u as *const u8);
    JITModule::new(builder)
}

// Declares helper signatures so Cranelift can emit correct calls.
pub fn declare_helpers(module: &mut JITModule, ptr_ty: types::Type) -> HelperFuncIds {
    fn declare(
        module: &mut JITModule,
        name: &str,
        params: &[types::Type],
        ret: Option<types::Type>,
    ) -> FuncId {
        let mut sig = module.make_signature();
        for &param in params {
            sig.params.push(cranelift_codegen::ir::AbiParam::new(param));
        }
        if let Some(ret) = ret {
            sig.returns.push(cranelift_codegen::ir::AbiParam::new(ret));
        }
        module
            .declare_function(name, Linkage::Import, &sig)
            .expect("declare helper")
    }

    let i8 = types::I8;
    let i32 = types::I32;
    let i64 = types::I64;

    HelperFuncIds {
        get_reg: declare(module, "jit_get_reg", &[ptr_ty, i8], Some(i64)),
        set_reg: declare(module, "jit_set_reg", &[ptr_ty, i8, i64], None),
        get_pc: declare(module, "jit_get_pc", &[ptr_ty], Some(i64)),
        set_pc: declare(module, "jit_set_pc", &[ptr_ty, i64], None),
        get_csr: declare(module, "jit_get_csr", &[ptr_ty, i32], Some(i64)),
        set_csr: declare(module, "jit_set_csr", &[ptr_ty, i32, i64], None),
        load_u8: declare(module, "jit_load_u8", &[ptr_ty, i64], Some(i64)),
        load_u16: declare(module, "jit_load_u16", &[ptr_ty, i64], Some(i64)),
        load_u32: declare(module, "jit_load_u32", &[ptr_ty, i64], Some(i64)),
        load_u64: declare(module, "jit_load_u64", &[ptr_ty, i64], Some(i64)),
        store_u8: declare(module, "jit_store_u8", &[ptr_ty, i64, i64], None),
        store_u16: declare(module, "jit_store_u16", &[ptr_ty, i64, i64], None),
        store_u32: declare(module, "jit_store_u32", &[ptr_ty, i64, i64], None),
        store_u64: declare(module, "jit_store_u64", &[ptr_ty, i64, i64], None),
        load_reserved_w: declare(module, "jit_load_reserved_w", &[ptr_ty, i64], Some(i64)),
        load_reserved_d: declare(module, "jit_load_reserved_d", &[ptr_ty, i64], Some(i64)),
        store_conditional_w: declare(
            module,
            "jit_store_conditional_w",
            &[ptr_ty, i64, i64],
            Some(i64),
        ),
        store_conditional_d: declare(
            module,
            "jit_store_conditional_d",
            &[ptr_ty, i64, i64],
            Some(i64),
        ),
        atomic_rmw_w: declare(
            module,
            "jit_atomic_rmw_w",
            &[ptr_ty, i64, i64, i32],
            Some(i64),
        ),
        atomic_rmw_d: declare(
            module,
            "jit_atomic_rmw_d",
            &[ptr_ty, i64, i64, i32],
            Some(i64),
        ),
        ecall: declare(module, "jit_ecall", &[ptr_ty, ptr_ty], Some(i64)),
        ebreak: declare(module, "jit_ebreak", &[ptr_ty, ptr_ty], None),
        halt: declare(module, "jit_halt", &[ptr_ty, i64], None),
        div_s: declare(module, "jit_div_s", &[i8, i64, i64], Some(i64)),
        div_u: declare(module, "jit_div_u", &[i8, i64, i64], Some(i64)),
        rem_s: declare(module, "jit_rem_s", &[i8, i64, i64], Some(i64)),
        rem_u: declare(module, "jit_rem_u", &[i8, i64, i64], Some(i64)),
    }
}
