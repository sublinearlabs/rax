use crate::ecall::handle_ecall;
use crate::ir2::{
    AtomicRmwOp, AtomicWidth, BlockId, EffectOp, IrFunction, IrType, MemWidth, Op, PureOp, Reg,
    Terminator,
};
use crate::trace::Tracer;
use crate::util::mask;
use crate::{HostIO, VM};

pub fn execute_ir<T: Tracer>(func: &IrFunction, vm: &mut VM<T>, io: &mut HostIO) {
    let mut values = vec![0u64; func.value_types.len()];
    let mut current = BlockId(0);
    let mut max_args = 0usize;
    for block in &func.blocks {
        if block.args.len() > max_args {
            max_args = block.args.len();
        }
    }
    let mut pending_args: Vec<u64> = Vec::with_capacity(max_args);

    loop {
        let block = &func.blocks[current.0 as usize];

        if !pending_args.is_empty() {
            if block.args.len() != pending_args.len() {
                panic!("block arg count mismatch");
            }
            for (idx, arg_id) in block.args.iter().enumerate() {
                values[arg_id.0 as usize] = pending_args[idx];
            }
            pending_args.clear();
        }

        for op in &block.ops {
            match op {
                Op::Pure { dst, op } => {
                    let value = eval_pure(op, &values, &func.value_types);
                    values[dst.0 as usize] = value;
                }
                Op::Effect(effect) => {
                    exec_effect(effect, &mut values, &func.value_types, vm, io);
                    if vm.halted {
                        return;
                    }
                }
            }
        }

        match block.term.as_ref().expect("missing terminator") {
            Terminator::Br { target, args } => {
                pending_args.clear();
                pending_args.reserve(args.len());
                for v in args {
                    pending_args.push(values[v.0 as usize]);
                }
                current = *target;
            }
            Terminator::Cbr {
                cond,
                t,
                f,
                t_args,
                f_args,
            } => {
                let cond_val = values[cond.0 as usize];
                if cond_val != 0 {
                    pending_args.clear();
                    pending_args.reserve(t_args.len());
                    for v in t_args {
                        pending_args.push(values[v.0 as usize]);
                    }
                    current = *t;
                } else {
                    pending_args.clear();
                    pending_args.reserve(f_args.len());
                    for v in f_args {
                        pending_args.push(values[v.0 as usize]);
                    }
                    current = *f;
                }
            }
            Terminator::Ret => return,
        }
    }
}

fn eval_pure(op: &PureOp, values: &[u64], types: &[IrType]) -> u64 {
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
        PureOp::Rem(a, b) => {
            let ty = types[a.0 as usize];
            let a = mask_value(values[a.0 as usize], ty);
            let b = mask_value(values[b.0 as usize], ty);
            rem_value(a, b, ty)
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

fn exec_effect<T: Tracer>(
    op: &EffectOp,
    values: &mut [u64],
    types: &[IrType],
    vm: &mut VM<T>,
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
        EffectOp::Load {
            dst,
            addr,
            width,
            signed: _,
        } => {
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

fn const_value(c: &crate::ir2::ConstVal) -> u64 {
    match *c {
        crate::ir2::ConstVal::I1(v) => {
            if v {
                1
            } else {
                0
            }
        }
        crate::ir2::ConstVal::I8(v) => (v as i64) as u64,
        crate::ir2::ConstVal::I16(v) => (v as i64) as u64,
        crate::ir2::ConstVal::I32(v) => (v as i64) as u64,
        crate::ir2::ConstVal::I64(v) => v as u64,
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

fn mask_value(value: u64, ty: IrType) -> u64 {
    value & ty_mask(ty)
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

#[cfg(test)]
mod tests {
    use super::execute_ir;
    use crate::decode::{Instruction, B, I};
    use crate::ir2::lower::lower_instruction_into;
    use crate::ir2::IrBuilder;
    use crate::trace::NoopTracer;
    use crate::{HostIO, VM};

    #[test]
    fn execute_ir_addi_then_beq_sets_reg_and_pc() {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);

        let addi = Instruction::Addi(I {
            rd: 1,
            rs1: 0,
            imm: 7,
        });
        lower_instruction_into(&addi, 0, 4, &mut builder);

        let beq = Instruction::Beq(B {
            rs1: 1,
            rs2: 1,
            imm: 12,
        });
        lower_instruction_into(&beq, 4, 8, &mut builder);

        let func = builder.finish();
        let mut vm = VM::<NoopTracer>::init();
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.reg(1), 7);
        assert_eq!(vm.pc(), 16);
    }
}
