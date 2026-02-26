use cranelift_codegen::ir::{condcodes::IntCC, types, InstBuilder, MemFlags, Value};
use cranelift_frontend::FunctionBuilder;

use crate::ir::{AtomicWidth, EffectOp, MemWidth};
use crate::vm::{VM_PC_OFFSET, VM_REGS_OFFSET};

use super::pure::atomic_rmw_tag_value;
use super::HelperFuncRefs;

pub fn lower_effect(
    builder: &mut FunctionBuilder,
    op: &EffectOp,
    values: &[Option<Value>],
    helpers: &HelperFuncRefs,
    vm_value: Value,
    io_value: Value,
) -> Option<(crate::ir::ValueId, Value)> {
    match op {
        EffectOp::GetReg { dst, reg } => {
            let reg_idx = reg_index(*reg);
            let value = if reg_idx == 0 {
                builder.ins().iconst(types::I64, 0)
            } else {
                let offset = (VM_REGS_OFFSET as i64) + (reg_idx as i64 * 8);
                let addr = builder.ins().iadd_imm(vm_value, offset);
                builder.ins().load(types::I64, MemFlags::trusted(), addr, 0)
            };
            Some((*dst, value))
        }
        EffectOp::SetReg { reg, val } => {
            let reg_idx = reg_index(*reg);
            if reg_idx != 0 {
                let value = value_for(values, *val);
                let offset = (VM_REGS_OFFSET as i64) + (reg_idx as i64 * 8);
                let addr = builder.ins().iadd_imm(vm_value, offset);
                builder.ins().store(MemFlags::trusted(), value, addr, 0);
            }
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
            let addr = builder.ins().iadd_imm(vm_value, VM_PC_OFFSET as i64);
            let value = builder.ins().load(types::I64, MemFlags::trusted(), addr, 0);
            Some((*dst, value))
        }
        EffectOp::SetPc { val } => {
            let value = value_for(values, *val);
            let addr = builder.ins().iadd_imm(vm_value, VM_PC_OFFSET as i64);
            builder.ins().store(MemFlags::trusted(), value, addr, 0);
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
            let op_val = atomic_rmw_tag_value(builder, *op);
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
            let call = builder.ins().call(helpers.ecall, &[vm_value, io_value]);
            let halted = builder.inst_results(call)[0];
            let halted = builder.ins().icmp_imm(IntCC::NotEqual, halted, 0);
            let ret_block = builder.create_block();
            let cont_block = builder.create_block();
            builder.ins().brif(halted, ret_block, &[], cont_block, &[]);
            builder.seal_block(ret_block);
            builder.seal_block(cont_block);
            builder.switch_to_block(ret_block);
            builder.ins().return_(&[]);
            builder.switch_to_block(cont_block);
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
