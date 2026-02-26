use crate::ir::{execute_ir, IrBuilder, IrFunction, IrType, MemWidth, Reg};
use crate::jit::compile::compile_ir_function;
use crate::jit::jit_module::build_jit_module;
use crate::trace::NoopTracer;
use crate::{HostIO, VM};

#[test]
fn lower_ir_function_matches_interpreter() {
    let ir = build_test_ir();
    let mut jit = build_jit_module();
    let jit_fn = compile_ir_function(&mut jit, &ir);

    let mut vm_ir = VM::<NoopTracer>::init();
    let mut io_ir = HostIO::new();
    vm_ir.reg_mut(1, 10);
    vm_ir.reg_mut(2, 10);
    execute_ir(&ir, &mut vm_ir, &mut io_ir);

    let mut vm_jit = VM::<NoopTracer>::init();
    let mut io_jit = HostIO::new();
    vm_jit.reg_mut(1, 10);
    vm_jit.reg_mut(2, 10);
    unsafe {
        jit_fn(&mut vm_jit as *mut _, &mut io_jit as *mut _);
    }
    assert_vm_matches(&mut vm_ir, &mut vm_jit);

    let mut vm_ir = VM::<NoopTracer>::init();
    let mut io_ir = HostIO::new();
    vm_ir.reg_mut(1, 5);
    vm_ir.reg_mut(2, 9);
    execute_ir(&ir, &mut vm_ir, &mut io_ir);

    let mut vm_jit = VM::<NoopTracer>::init();
    let mut io_jit = HostIO::new();
    vm_jit.reg_mut(1, 5);
    vm_jit.reg_mut(2, 9);
    unsafe {
        jit_fn(&mut vm_jit as *mut _, &mut io_jit as *mut _);
    }
    assert_vm_matches(&mut vm_ir, &mut vm_jit);
}

fn build_test_ir() -> IrFunction {
    let mut builder = IrBuilder::new();
    let entry = builder.block();
    let then_block = builder.block_with_args(&[IrType::I64]);
    let else_block = builder.block_with_args(&[IrType::I64]);

    builder.switch_to(entry);
    let x1 = builder.get_reg(Reg::X1);
    let x2 = builder.get_reg(Reg::X2);
    let cond = builder.eq(x1, x2);
    builder.cbr(cond, then_block, else_block, vec![x1], vec![x2]);

    builder.switch_to(then_block);
    let arg = builder.block_arg(then_block, 0);
    let five = builder.const_i64(5);
    let val = builder.add(arg, five, IrType::I64);
    builder.set_reg(Reg::X3, val);
    let addr = builder.const_i64(0x100);
    builder.store(addr, val, MemWidth::W64);
    let pc = builder.const_i64(0x200);
    builder.set_pc(pc);
    builder.halt(0);
    builder.ret();

    builder.switch_to(else_block);
    let arg = builder.block_arg(else_block, 0);
    let seven = builder.const_i64(7);
    let val = builder.add(arg, seven, IrType::I64);
    builder.set_reg(Reg::X3, val);
    let addr = builder.const_i64(0x100);
    builder.store(addr, val, MemWidth::W64);
    let pc = builder.const_i64(0x300);
    builder.set_pc(pc);
    builder.halt(1);
    builder.ret();

    builder.finish()
}

fn assert_vm_matches(vm_ir: &mut VM<NoopTracer>, vm_jit: &mut VM<NoopTracer>) {
    assert_eq!(vm_ir.halted, vm_jit.halted, "halted mismatch");
    assert_eq!(vm_ir.exit_code, vm_jit.exit_code, "exit code mismatch");
    assert_eq!(vm_ir.pc(), vm_jit.pc(), "pc mismatch");
    assert_eq!(vm_ir.reg(3), vm_jit.reg(3), "x3 mismatch");
    assert_eq!(
        vm_ir.load_u64(0x100),
        vm_jit.load_u64(0x100),
        "memory mismatch"
    );
}
