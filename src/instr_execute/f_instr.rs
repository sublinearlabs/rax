// F instructions

use crate::Tracer;
use crate::VM;
use crate::decode::{R4, RF};
use crate::util::{classify32, is_snan_f32, mask32, sext};

#[inline(always)]
pub(crate) fn execute_FmaddS<T: Tracer>(vm: &mut VM<T>, insn: R4) {
    let a = vm.read_f32(insn.rs1);
    let b = vm.read_f32(insn.rs2);
    let c = vm.read_f32(insn.rs3);
    let mut res = a.mul_add(b, c);

    // Canonicalize NaN
    if res.is_nan() && !a.is_nan() && !b.is_nan() && !c.is_nan() {
        res = f32::from_bits(0x7FC00000);
    }

    vm.write_f32(insn.rd, res);
    vm.raise_fflags_fma_f32(a, b, c, res);
}

#[inline(always)]
pub(crate) fn execute_FmsubS<T: Tracer>(vm: &mut VM<T>, insn: R4) {
    let a = vm.read_f32(insn.rs1);
    let b = vm.read_f32(insn.rs2);
    let c = vm.read_f32(insn.rs3);
    let res = a.mul_add(b, -c);
    vm.write_f32(insn.rd, res);
    vm.raise_fflags_fma_f32(a, b, -c, res);
}

#[inline(always)]
pub(crate) fn execute_FnmsubS<T: Tracer>(vm: &mut VM<T>, insn: R4) {
    let a = vm.read_f32(insn.rs1);
    let b = vm.read_f32(insn.rs2);
    let c = vm.read_f32(insn.rs3);
    let res = (-a).mul_add(b, c);
    vm.write_f32(insn.rd, res);
    vm.raise_fflags_fma_f32(-a, b, c, res);
}

#[inline(always)]
pub(crate) fn execute_FnmaddS<T: Tracer>(vm: &mut VM<T>, insn: R4) {
    let a = vm.read_f32(insn.rs1);
    let b = vm.read_f32(insn.rs2);
    let c = vm.read_f32(insn.rs3);
    let res = (-a).mul_add(b, -c);
    vm.write_f32(insn.rd, res);
    vm.raise_fflags_fma_f32(-a, b, -c, res);
}

#[inline(always)]
pub(crate) fn execute_FaddS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let a = vm.read_f32(insn.rs1);
    let b = vm.read_f32(insn.rs2);
    let mut res = a + b;

    // Canonicalize NaN
    if res.is_nan() && !a.is_nan() && !b.is_nan() {
        res = f32::from_bits(0x7FC00000);
    }

    vm.write_f32(insn.rd, res);
    vm.raise_fflags_f32(a, b, res, '+');
}

#[inline(always)]
pub(crate) fn execute_FsubS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let a = vm.read_f32(insn.rs1);
    let b = vm.read_f32(insn.rs2);
    let mut res = a - b;

    // Canonicalize NaN: RISC-V requires positive quiet NaN
    if res.is_nan() && !a.is_nan() && !b.is_nan() {
        res = f32::from_bits(0x7FC00000); // Canonical positive qNaN
    }

    vm.write_f32(insn.rd, res);
    vm.raise_fflags_f32(a, b, res, '-');
}

#[inline(always)]
pub(crate) fn execute_FmulS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let a = vm.read_f32(insn.rs1);
    let b = vm.read_f32(insn.rs2);
    let mut res = a * b;

    // Canonicalize NaN
    if res.is_nan() && !a.is_nan() && !b.is_nan() {
        res = f32::from_bits(0x7FC00000);
    }

    vm.write_f32(insn.rd, res);
    vm.raise_fflags_f32(a, b, res, '*');
}

#[inline(always)]
pub(crate) fn execute_FdivS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let a = vm.read_f32(insn.rs1);
    let b = vm.read_f32(insn.rs2);
    let mut res = a / b;

    // Canonicalize NaN
    if res.is_nan() && !a.is_nan() && !b.is_nan() {
        res = f32::from_bits(0x7FC00000);
    }

    vm.write_f32(insn.rd, res);
    vm.raise_fflags_f32(a, b, res, '/');
}

#[inline(always)]
pub(crate) fn execute_FsqrtS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let a = vm.read_f32(insn.rs1);

    if is_snan_f32(a) || (a < 0.0 && !a.is_nan()) {
        vm.fcsr_reg |= 0b10000;
    }

    let mut res = a.sqrt();

    // Canonicalize NaN for sqrt of negative
    if res.is_nan() && !a.is_nan() {
        res = f32::from_bits(0x7FC00000);
    }

    if !res.is_nan() && a >= 0.0 {
        let exact = (a as f64).sqrt();
        if exact != (res as f64) {
            vm.fcsr_reg |= 0b00001;
            vm.tracer.record_csr_reg(vm.fcsr_reg);
        }
    }

    vm.write_f32(insn.rd, res);
}

#[inline(always)]
pub(crate) fn execute_FsgnjS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let rs1_bits = (vm.f_reg[insn.rs1 as usize] & 0xFFFFFFFF) as u32;
    let rs2_bits = (vm.f_reg[insn.rs2 as usize] & 0xFFFFFFFF) as u32;
    let sign = rs2_bits & (1 << 31);
    let val = rs1_bits & mask32(31);
    let result = sign | val;
    let res = 0xFFFF_FFFF_0000_0000 | (result as u64);
    vm.f_reg[insn.rd as usize] = res;
    vm.tracer.record_rd(insn.rd, res);
}

#[inline(always)]
pub(crate) fn execute_FsgnjnS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let rs1_bits = (vm.f_reg[insn.rs1 as usize] & 0xFFFFFFFF) as u32;
    let rs2_bits = (vm.f_reg[insn.rs2 as usize] & 0xFFFFFFFF) as u32;
    let sign = (rs2_bits ^ (1 << 31)) & (1 << 31);
    let val = rs1_bits & mask32(31);
    let result = sign | val;
    let res = 0xFFFF_FFFF_0000_0000 | (result as u64);
    vm.f_reg[insn.rd as usize] = res;
    vm.tracer.record_rd(insn.rd, res);
}

#[inline(always)]
pub(crate) fn execute_FsgnjxS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let rs1_bits = (vm.f_reg[insn.rs1 as usize] & 0xFFFFFFFF) as u32;
    let rs2_bits = (vm.f_reg[insn.rs2 as usize] & 0xFFFFFFFF) as u32;
    let sign = (rs1_bits & (1 << 31)) ^ (rs2_bits & (1 << 31));
    let val = rs1_bits & mask32(31);
    let result = sign | val;
    let res = 0xFFFF_FFFF_0000_0000 | (result as u64);
    vm.f_reg[insn.rd as usize] = res;
    vm.tracer.record_rd(insn.rd, res);
}

#[inline(always)]
pub(crate) fn execute_FminS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let a = vm.read_f32(insn.rs1);
    let b = vm.read_f32(insn.rs2);

    // Set NV flag for signaling NaN
    if is_snan_f32(a) || is_snan_f32(b) {
        vm.fcsr_reg |= 0b10000;
        vm.tracer.record_csr_reg(vm.fcsr_reg);
    }

    let res = if a.is_nan() && b.is_nan() {
        f32::from_bits(0x7FC00000) // Canonical NaN
    } else if a.is_nan() {
        b
    } else if b.is_nan() {
        a
    } else if a == 0.0 && b == 0.0 {
        // -0.0 is less than +0.0
        if a.to_bits() & 0x80000000 != 0 { a } else { b }
    } else {
        a.min(b)
    };
    vm.write_f32(insn.rd, res);
}

#[inline(always)]
pub(crate) fn execute_FmaxS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let a = vm.read_f32(insn.rs1);
    let b = vm.read_f32(insn.rs2);

    // Set NV flag for signaling NaN
    if is_snan_f32(a) || is_snan_f32(b) {
        vm.fcsr_reg |= 0b10000;
        vm.tracer.record_csr_reg(vm.fcsr_reg);
    }

    let res = if a.is_nan() && b.is_nan() {
        f32::from_bits(0x7FC00000) // Canonical NaN
    } else if a.is_nan() {
        b
    } else if b.is_nan() {
        a
    } else if a == 0.0 && b == 0.0 {
        // +0.0 is greater than -0.0
        if a.to_bits() & 0x80000000 == 0 { a } else { b }
    } else {
        a.max(b)
    };
    vm.write_f32(insn.rd, res);
}

#[inline(always)]
pub(crate) fn execute_FcvtWS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let val = vm.read_f32(insn.rs1);

    let (result, flags): (i32, u32) = if val.is_nan() {
        (i32::MAX, 0b10000)
    } else if val >= 2147483648.0_f32 {
        (i32::MAX, 0b10000)
    } else if val < -2147483648.0_f32 {
        (i32::MIN, 0b10000)
    } else {
        let int_val = val.trunc() as i32;
        let inexact = if val != val.trunc() { 0b00001 } else { 0 };
        (int_val, inexact)
    };

    vm.fcsr_reg |= flags;
    vm.reg_mut(insn.rd, result as i64 as u64);
    vm.tracer.record_csr_reg(vm.fcsr_reg);
}

#[inline(always)]
pub(crate) fn execute_FcvtWuS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let val = vm.read_f32(insn.rs1);

    let (result, flags): (u32, u32) = if val.is_nan() {
        (u32::MAX, 0b10000) // NV
    } else if val <= -1.0 {
        // -1.0 or less cannot be represented as unsigned - invalid
        (0_u32, 0b10000) // NV
    } else if val < 0.0 {
        // Between -1.0 (exclusive) and 0.0 - truncates to 0, inexact
        (0_u32, 0b00001) // NX only
    } else if val >= 4294967296.0_f32 {
        (u32::MAX, 0b10000) // NV
    } else {
        let truncated = val.trunc();
        let int_val = truncated as u32;
        let inexact = if val != truncated { 0b00001 } else { 0 };
        (int_val, inexact)
    };

    vm.fcsr_reg |= flags;
    vm.reg_mut(insn.rd, result as i32 as i64 as u64);
    vm.tracer.record_csr_reg(vm.fcsr_reg);
}

#[inline(always)]
pub(crate) fn execute_FmvXW<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let raw_bits = (vm.f_reg[insn.rs1 as usize] & 0xFFFFFFFF) as u32;
    let result = sext(raw_bits as u64, 32);

    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_FeqS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let a = vm.read_f32(insn.rs1);
    let b = vm.read_f32(insn.rs2);

    // FeqS only sets NV for signaling NaN
    if is_snan_f32(a) || is_snan_f32(b) {
        vm.fcsr_reg |= 0b10000;
        vm.tracer.record_csr_reg(vm.fcsr_reg);
    }

    let res = if a.is_nan() || b.is_nan() {
        0
    } else {
        (a == b) as u64
    };
    vm.reg_mut(insn.rd, res);
}

#[inline(always)]
pub(crate) fn execute_FltS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let a = vm.read_f32(insn.rs1);
    let b = vm.read_f32(insn.rs2);

    // FltS sets NV for ANY NaN (not just signaling)
    if a.is_nan() || b.is_nan() {
        vm.fcsr_reg |= 0b10000;
        vm.tracer.record_csr_reg(vm.fcsr_reg);
        vm.reg_mut(insn.rd, 0);
    } else {
        vm.reg_mut(insn.rd, (a < b) as u64);
    }
}

#[inline(always)]
pub(crate) fn execute_FleS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let a = vm.read_f32(insn.rs1);
    let b = vm.read_f32(insn.rs2);

    // FleS sets NV for ANY NaN (not just signaling)
    if a.is_nan() || b.is_nan() {
        vm.fcsr_reg |= 0b10000;
        vm.tracer.record_csr_reg(vm.fcsr_reg);
        vm.reg_mut(insn.rd, 0);
    } else {
        vm.reg_mut(insn.rd, (a <= b) as u64);
    }
}

#[inline(always)]
pub(crate) fn execute_FclassS<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let val = classify32(vm.read_f32(insn.rs1).to_bits());
    vm.reg_mut(insn.rd, val);
}

#[inline(always)]
pub(crate) fn execute_FcvtSW<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let a = (vm.reg(insn.rs1) as i32) as f32;
    vm.write_f32(insn.rd, a);
}

#[inline(always)]
pub(crate) fn execute_FcvtSWu<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let a = (vm.reg(insn.rs1) as u32) as f32;
    vm.write_f32(insn.rd, a);
}

#[inline(always)]
pub(crate) fn execute_FmvWX<T: Tracer>(vm: &mut VM<T>, insn: RF) {
    let a = f32::from_bits(vm.reg(insn.rs1) as u32);
    vm.write_f32(insn.rd, a);
}
