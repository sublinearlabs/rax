use crate::ir::{ConstVal, IrType, PureOp};
use riscv_core::util::mask;

pub(crate) fn eval_pure(op: &PureOp, values: &[u64], types: &[IrType]) -> u64 {
    match op {
        PureOp::Const(c) => const_value(c),
        PureOp::Add(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            mask_value(a.wrapping_add(b), ty)
        }
        PureOp::Sub(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            mask_value(a.wrapping_sub(b), ty)
        }
        PureOp::Mul(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            mask_value(a.wrapping_mul(b), ty)
        }
        PureOp::Div(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            div_value(a, b, ty)
        }
        PureOp::Divu(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            divu_value(a, b, ty)
        }
        PureOp::Rem(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            rem_value(a, b, ty)
        }
        PureOp::Remu(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            remu_value(a, b, ty)
        }
        PureOp::And(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            mask_value(a & b, ty)
        }
        PureOp::Or(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            mask_value(a | b, ty)
        }
        PureOp::Xor(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            mask_value(a ^ b, ty)
        }
        PureOp::Shl(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let sh = shift_amount(values[b.0 as usize], ty);
            mask_value(a << sh, ty)
        }
        PureOp::Shr(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let sh = shift_amount(values[b.0 as usize], ty);
            mask_value(a >> sh, ty)
        }
        PureOp::Sar(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let sh = shift_amount(values[b.0 as usize], ty);
            let signed = sign_extend(a, ty);
            mask_value((signed >> sh) as u64, ty)
        }
        PureOp::Eq(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            bool_to_u64(a == b)
        }
        PureOp::Ne(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            bool_to_u64(a != b)
        }
        PureOp::Lt(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            bool_to_u64(sign_extend(a, ty) < sign_extend(b, ty))
        }
        PureOp::Ltu(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            bool_to_u64(a < b)
        }
        PureOp::Ge(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            bool_to_u64(sign_extend(a, ty) >= sign_extend(b, ty))
        }
        PureOp::Geu(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            bool_to_u64(a >= b)
        }
        PureOp::Sext { v, from, to } => sext_value(values[v.0 as usize], *from, *to),
        PureOp::Zext { v, from, to } => zext_value(values[v.0 as usize], *from, *to),
        PureOp::Trunc { v, from: _, to } => trunc_value(values[v.0 as usize], *to),
        PureOp::Select { cond, t, f } => {
            if values[cond.0 as usize] & 1 != 0 {
                values[t.0 as usize]
            } else {
                values[f.0 as usize]
            }
        }
    }
}

pub(crate) fn mask_value(value: u64, ty: IrType) -> u64 {
    value & ty_mask(ty)
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

fn bool_to_u64(v: bool) -> u64 {
    if v {
        1
    } else {
        0
    }
}

fn ty_bits(ty: IrType) -> u8 {
    match ty {
        IrType::I1 => 1,
        IrType::I8 => 8,
        IrType::I16 => 16,
        IrType::I32 => 32,
        IrType::I64 => 64,
    }
}

fn ty_mask(ty: IrType) -> u64 {
    mask(ty_bits(ty))
}

fn sign_extend(value: u64, ty: IrType) -> i64 {
    let bits = ty_bits(ty) as u32;
    if bits == 64 {
        value as i64
    } else {
        let shift = 64 - bits;
        ((value << shift) as i64) >> shift
    }
}

fn shift_amount(value: u64, ty: IrType) -> u32 {
    let bits = ty_bits(ty) as u64;
    if bits <= 1 {
        0
    } else {
        (value & (bits - 1)) as u32
    }
}

fn div_value(a: u64, b: u64, ty: IrType) -> u64 {
    let bits = ty_bits(ty) as u32;
    let a_signed = sign_extend(a, ty);
    let b_signed = sign_extend(b, ty);
    if b_signed == 0 {
        return ty_mask(ty);
    }
    let min = signed_min(bits);
    if a_signed == min && b_signed == -1 {
        return mask_value(min as u64, ty);
    }
    mask_value((a_signed / b_signed) as u64, ty)
}

fn divu_value(a: u64, b: u64, ty: IrType) -> u64 {
    if b == 0 {
        return ty_mask(ty);
    }
    mask_value(a / b, ty)
}

fn rem_value(a: u64, b: u64, ty: IrType) -> u64 {
    let bits = ty_bits(ty) as u32;
    let a_signed = sign_extend(a, ty);
    let b_signed = sign_extend(b, ty);
    if b_signed == 0 {
        return mask_value(a_signed as u64, ty);
    }
    let min = signed_min(bits);
    if a_signed == min && b_signed == -1 {
        return 0;
    }
    mask_value((a_signed % b_signed) as u64, ty)
}

fn remu_value(a: u64, b: u64, ty: IrType) -> u64 {
    if b == 0 {
        return mask_value(a, ty);
    }
    mask_value(a % b, ty)
}

fn signed_min(bits: u32) -> i64 {
    if bits == 64 {
        i64::MIN
    } else {
        -(1i64 << (bits - 1))
    }
}

fn sext_value(value: u64, from: IrType, to: IrType) -> u64 {
    let v = mask_value(value, from);
    let signed = sign_extend(v, from);
    mask_value(signed as u64, to)
}

fn zext_value(value: u64, from: IrType, to: IrType) -> u64 {
    let v = mask_value(value, from);
    mask_value(v, to)
}

fn trunc_value(value: u64, to: IrType) -> u64 {
    mask_value(value, to)
}
