use crate::ecall::handle_ecall;
use crate::ir::AtomicRmwOp;
use crate::trace::NoopTracer;
use crate::util::mask;
use crate::{HostIO, VM};

#[inline]
fn vm_ptr<'a>(vm: *mut VM<NoopTracer>) -> &'a mut VM<NoopTracer> {
    unsafe { &mut *vm }
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_get_csr(vm: *mut VM<NoopTracer>, csr: u32) -> u64 {
    let vm = vm_ptr(vm);
    vm.read_csr(csr) as u64
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_set_csr(vm: *mut VM<NoopTracer>, csr: u32, val: u64) {
    let vm = vm_ptr(vm);
    vm.set_csr(csr, val as u32);
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_load_u8(vm: *mut VM<NoopTracer>, addr: u64) -> u64 {
    let vm = vm_ptr(vm);
    vm.load_u8(addr as usize) as u64
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_load_u16(vm: *mut VM<NoopTracer>, addr: u64) -> u64 {
    let vm = vm_ptr(vm);
    vm.load_u16(addr as usize) as u64
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_load_u32(vm: *mut VM<NoopTracer>, addr: u64) -> u64 {
    let vm = vm_ptr(vm);
    vm.load_u32(addr as usize) as u64
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_load_u64(vm: *mut VM<NoopTracer>, addr: u64) -> u64 {
    let vm = vm_ptr(vm);
    vm.load_u64(addr as usize)
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_store_u8(vm: *mut VM<NoopTracer>, addr: u64, val: u64) {
    let vm = vm_ptr(vm);
    vm.store_u8(addr as usize, val as u8);
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_store_u16(vm: *mut VM<NoopTracer>, addr: u64, val: u64) {
    let vm = vm_ptr(vm);
    vm.store_u16(addr as usize, val as u16);
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_store_u32(vm: *mut VM<NoopTracer>, addr: u64, val: u64) {
    let vm = vm_ptr(vm);
    vm.store_u32(addr as usize, val as u32);
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_store_u64(vm: *mut VM<NoopTracer>, addr: u64, val: u64) {
    let vm = vm_ptr(vm);
    vm.store_u64(addr as usize, val);
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_load_reserved_w(vm: *mut VM<NoopTracer>, addr: u64) -> u64 {
    let vm = vm_ptr(vm);
    let value = vm.load_u32(addr as usize) as u64;
    vm.reservation_set = addr;
    value & mask(32)
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_load_reserved_d(vm: *mut VM<NoopTracer>, addr: u64) -> u64 {
    let vm = vm_ptr(vm);
    let value = vm.load_u64(addr as usize);
    vm.reservation_set = addr;
    value
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_store_conditional_w(vm: *mut VM<NoopTracer>, addr: u64, val: u64) -> u64 {
    let vm = vm_ptr(vm);
    let success = addr == vm.reservation_set;
    if success {
        vm.store_u32(addr as usize, val as u32);
    }
    vm.reservation_set = 0;
    if success {
        0
    } else {
        1
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_store_conditional_d(vm: *mut VM<NoopTracer>, addr: u64, val: u64) -> u64 {
    let vm = vm_ptr(vm);
    let success = addr == vm.reservation_set;
    if success {
        vm.store_u64(addr as usize, val);
    }
    vm.reservation_set = 0;
    if success {
        0
    } else {
        1
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_atomic_rmw_w(vm: *mut VM<NoopTracer>, addr: u64, val: u64, op: u32) -> u64 {
    let vm = vm_ptr(vm);
    let read_value = vm.load_u32(addr as usize) as u64;
    let rs2_val = val & mask(32);
    let op = decode_atomic_rmw_op(op);
    let write_value = atomic_rmw_w(read_value, rs2_val, op);
    vm.store_u32(addr as usize, write_value as u32);
    read_value & mask(32)
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_atomic_rmw_d(vm: *mut VM<NoopTracer>, addr: u64, val: u64, op: u32) -> u64 {
    let vm = vm_ptr(vm);
    let read_value = vm.load_u64(addr as usize);
    let op = decode_atomic_rmw_op(op);
    let write_value = atomic_rmw_d(read_value, val, op);
    vm.store_u64(addr as usize, write_value);
    read_value
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_ecall(vm: *mut VM<NoopTracer>, io: *mut HostIO) -> u64 {
    let vm = vm_ptr(vm);
    let io = unsafe { &mut *io };
    handle_ecall(vm, io);
    vm.halted as u64
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_ebreak(vm: *mut VM<NoopTracer>, io: *mut HostIO) {
    let vm = vm_ptr(vm);
    let io = unsafe { &mut *io };
    handle_ecall(vm, io);
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_halt(vm: *mut VM<NoopTracer>, code: u64) {
    let vm = vm_ptr(vm);
    vm.exit_code = code;
    vm.halted = true;
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_div_s(ty_bits: u8, a: u64, b: u64) -> u64 {
    let bits = ty_bits as u32;
    let a_signed = sign_extend(a, bits);
    let b_signed = sign_extend(b, bits);
    if b_signed == 0 {
        return mask(bits as u8);
    }
    let min = signed_min(bits);
    if a_signed == min && b_signed == -1 {
        return mask_value(min as u64, bits);
    }
    mask_value((a_signed / b_signed) as u64, bits)
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_div_u(ty_bits: u8, a: u64, b: u64) -> u64 {
    let bits = ty_bits as u32;
    if b == 0 {
        return mask(bits as u8);
    }
    mask_value(a / b, bits)
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_rem_s(ty_bits: u8, a: u64, b: u64) -> u64 {
    let bits = ty_bits as u32;
    let a_signed = sign_extend(a, bits);
    let b_signed = sign_extend(b, bits);
    if b_signed == 0 {
        return mask_value(a_signed as u64, bits);
    }
    let min = signed_min(bits);
    if a_signed == min && b_signed == -1 {
        return 0;
    }
    mask_value((a_signed % b_signed) as u64, bits)
}

#[unsafe(no_mangle)]
pub extern "C" fn jit_rem_u(ty_bits: u8, a: u64, b: u64) -> u64 {
    let bits = ty_bits as u32;
    if b == 0 {
        return mask_value(a, bits);
    }
    mask_value(a % b, bits)
}

fn decode_atomic_rmw_op(op: u32) -> AtomicRmwOp {
    match op {
        0 => AtomicRmwOp::Xchg,
        1 => AtomicRmwOp::Add,
        2 => AtomicRmwOp::And,
        3 => AtomicRmwOp::Or,
        4 => AtomicRmwOp::Xor,
        5 => AtomicRmwOp::Min,
        6 => AtomicRmwOp::Max,
        7 => AtomicRmwOp::Umin,
        _ => AtomicRmwOp::Umax,
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

fn mask_value(value: u64, bits: u32) -> u64 {
    value & mask(bits as u8)
}

fn sign_extend(value: u64, bits: u32) -> i64 {
    if bits >= 64 {
        value as i64
    } else {
        let shift = 64 - bits;
        ((value << shift) as i64) >> shift
    }
}

fn signed_min(bits: u32) -> i64 {
    if bits >= 64 {
        i64::MIN
    } else {
        -(1i64 << (bits - 1))
    }
}
