use rax_interpreter::handle_ecall;
use crate::ir::{AtomicRmwOp, AtomicWidth, EffectOp, IrType, MemWidth, Reg};
use rax_core::util::mask;
use rax_interpreter::{HostIO, VM};

use super::pure::mask_value;

pub(crate) fn exec_effect(
    op: &EffectOp,
    values: &mut [u64],
    types: &[IrType],
    vm: &mut VM,
    io: &mut HostIO,
) {
    match op {
        EffectOp::GetReg { dst, reg } => {
            let idx = reg_index(*reg);
            values[dst.0 as usize] = vm.reg(idx);
        }
        EffectOp::SetReg { reg, val } => {
            let idx = reg_index(*reg);
            vm.reg_mut(idx, values[val.0 as usize]);
        }
        EffectOp::GetCsr { dst, csr } => {
            values[dst.0 as usize] = vm.read_csr(*csr) as u64;
        }
        EffectOp::SetCsr { csr, val } => {
            vm.set_csr(*csr, values[val.0 as usize] as u32);
        }
        EffectOp::GetPc { dst } => {
            values[dst.0 as usize] = vm.pc();
        }
        EffectOp::SetPc { val } => {
            vm.set_pc(values[val.0 as usize]);
        }
        EffectOp::Load { dst, addr, width } => {
            let addr = values[addr.0 as usize] as usize;
            let value = match width {
                MemWidth::W8 => vm.load_u8(addr) as u64,
                MemWidth::W16 => vm.load_u16(addr) as u64,
                MemWidth::W32 => vm.load_u32(addr) as u64,
                MemWidth::W64 => vm.load_u64(addr),
            };
            let dst_ty = types[dst.0 as usize];
            values[dst.0 as usize] = mask_value(value, dst_ty);
        }
        EffectOp::Store { addr, val, width } => {
            let addr = values[addr.0 as usize] as usize;
            let value = values[val.0 as usize];
            match width {
                MemWidth::W8 => vm.store_u8(addr, value as u8),
                MemWidth::W16 => vm.store_u16(addr, value as u16),
                MemWidth::W32 => vm.store_u32(addr, value as u32),
                MemWidth::W64 => vm.store_u64(addr, value),
            }
        }
        EffectOp::LoadReserved { dst, addr, width } => {
            let addr = values[addr.0 as usize] as u64;
            let value = match width {
                AtomicWidth::W => vm.load_u32(addr as usize) as u64,
                AtomicWidth::D => vm.load_u64(addr as usize),
            };
            vm.reservation_set = addr;
            let dst_ty = types[dst.0 as usize];
            values[dst.0 as usize] = mask_value(value, dst_ty);
        }
        EffectOp::StoreConditional {
            dst,
            addr,
            val,
            width,
        } => {
            let addr = values[addr.0 as usize] as u64;
            let value = values[val.0 as usize];
            let success = addr == vm.reservation_set;
            if success {
                match width {
                    AtomicWidth::W => vm.store_u32(addr as usize, value as u32),
                    AtomicWidth::D => vm.store_u64(addr as usize, value),
                }
            }
            vm.reservation_set = 0;
            let dst_ty = types[dst.0 as usize];
            values[dst.0 as usize] = mask_value(if success { 0 } else { 1 }, dst_ty);
        }
        EffectOp::AtomicRmw {
            dst,
            addr,
            val,
            op,
            width,
        } => {
            let addr = values[addr.0 as usize] as u64;
            match width {
                AtomicWidth::W => {
                    let read_value = vm.load_u32(addr as usize) as u64;
                    let rs2_val = values[val.0 as usize] & mask(32);
                    let write_value = atomic_rmw_w(read_value, rs2_val, *op);
                    vm.store_u32(addr as usize, write_value as u32);
                    values[dst.0 as usize] = read_value & mask(32);
                }
                AtomicWidth::D => {
                    let read_value = vm.load_u64(addr as usize);
                    let rs2_val = values[val.0 as usize];
                    let write_value = atomic_rmw_d(read_value, rs2_val, *op);
                    vm.store_u64(addr as usize, write_value);
                    values[dst.0 as usize] = read_value;
                }
            }
        }
        EffectOp::Ecall | EffectOp::Ebreak => {
            handle_ecall(vm, io);
        }
        EffectOp::Halt { code } => {
            vm.exit_code = *code;
            vm.halted = true;
        }
    }
}

fn reg_index(reg: Reg) -> u8 {
    match reg {
        Reg::X0 => 0,
        Reg::X1 => 1,
        Reg::X2 => 2,
        Reg::X3 => 3,
        Reg::X4 => 4,
        Reg::X5 => 5,
        Reg::X6 => 6,
        Reg::X7 => 7,
        Reg::X8 => 8,
        Reg::X9 => 9,
        Reg::X10 => 10,
        Reg::X11 => 11,
        Reg::X12 => 12,
        Reg::X13 => 13,
        Reg::X14 => 14,
        Reg::X15 => 15,
        Reg::X16 => 16,
        Reg::X17 => 17,
        Reg::X18 => 18,
        Reg::X19 => 19,
        Reg::X20 => 20,
        Reg::X21 => 21,
        Reg::X22 => 22,
        Reg::X23 => 23,
        Reg::X24 => 24,
        Reg::X25 => 25,
        Reg::X26 => 26,
        Reg::X27 => 27,
        Reg::X28 => 28,
        Reg::X29 => 29,
        Reg::X30 => 30,
        Reg::X31 => 31,
    }
}

fn atomic_rmw_w(read_value: u64, rs2_val: u64, op: AtomicRmwOp) -> u64 {
    let read_i32 = read_value as i32;
    let rs2_i32 = rs2_val as i32;
    let result = match op {
        AtomicRmwOp::Xchg => rs2_val,
        AtomicRmwOp::Add => (read_i32.wrapping_add(rs2_i32) as i64) as u64,
        AtomicRmwOp::And => (read_i32 & rs2_i32) as i64 as u64,
        AtomicRmwOp::Or => (read_i32 | rs2_i32) as i64 as u64,
        AtomicRmwOp::Xor => (read_i32 ^ rs2_i32) as i64 as u64,
        AtomicRmwOp::Min => read_i32.min(rs2_i32) as i64 as u64,
        AtomicRmwOp::Max => read_i32.max(rs2_i32) as i64 as u64,
        AtomicRmwOp::Umin => read_value.min(rs2_val),
        AtomicRmwOp::Umax => read_value.max(rs2_val),
    };
    result & mask(32)
}

fn atomic_rmw_d(read_value: u64, rs2_val: u64, op: AtomicRmwOp) -> u64 {
    match op {
        AtomicRmwOp::Xchg => rs2_val,
        AtomicRmwOp::Add => read_value.wrapping_add(rs2_val),
        AtomicRmwOp::And => read_value & rs2_val,
        AtomicRmwOp::Or => read_value | rs2_val,
        AtomicRmwOp::Xor => read_value ^ rs2_val,
        AtomicRmwOp::Min => (read_value as i64).min(rs2_val as i64) as u64,
        AtomicRmwOp::Max => (read_value as i64).max(rs2_val as i64) as u64,
        AtomicRmwOp::Umin => read_value.min(rs2_val),
        AtomicRmwOp::Umax => read_value.max(rs2_val),
    }
}
