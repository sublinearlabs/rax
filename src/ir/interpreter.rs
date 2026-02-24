use crate::ecall::handle_ecall;
use crate::ir::{BlockId, EffectOp, IrFunction, IrType, Op, PureOp, Reg, Terminator, ValueId};
use crate::trace::Tracer;
use crate::{HostIO, VM};

#[derive(Clone, Copy)]
enum Value {
    I1(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Value {
    fn as_i64(&self) -> i64 {
        match self {
            Value::I1(v) => *v as i64,
            Value::I8(v) => *v as i64,
            Value::I16(v) => *v as i64,
            Value::I32(v) => *v as i64,
            Value::I64(v) => *v,
            Value::F32(v) => v.to_bits() as i64,
            Value::F64(v) => v.to_bits() as i64,
        }
    }

    fn as_f32(&self) -> f32 {
        match self {
            Value::F32(v) => *v,
            _ => panic!("expected F32 value"),
        }
    }

    fn as_bool(&self) -> bool {
        match self {
            Value::I1(v) => *v,
            _ => panic!("expected I1 value"),
        }
    }
}

pub fn execute_ir<T: Tracer>(func: &IrFunction, vm: &mut VM<T>, io: &mut HostIO) {
    let mut values: Vec<Value> = func
        .value_types
        .iter()
        .map(|ty| match ty {
            IrType::I1 => Value::I1(false),
            IrType::I8 => Value::I8(0),
            IrType::I16 => Value::I16(0),
            IrType::I32 => Value::I32(0),
            IrType::I64 => Value::I64(0),
            IrType::F32 => Value::F32(0.0),
            IrType::F64 => Value::F64(0.0),
        })
        .collect();
    let mut current = BlockId(0);
    let mut pending_args: Vec<Value> = Vec::new();

    loop {
        let block = &func.blocks[current.0 as usize];

        if !pending_args.is_empty() {
            if block.args.len() != pending_args.len() {
                panic!("block arg count mismatch");
            }
            for (arg_id, value) in block.args.iter().zip(pending_args.drain(..)) {
                values[arg_id.0 as usize] = value;
            }
        }

        for op in &block.ops {
            match op {
                Op::Pure { dst, op } => {
                    let value = eval_pure(op, &values);
                    values[dst.0 as usize] = value;
                }
                Op::Effect(effect) => {
                    exec_effect(effect, &mut values, vm, io);
                    if vm.halted {
                        return;
                    }
                }
            }
        }

        match block.term.as_ref().expect("missing terminator") {
            Terminator::Br { target, args } => {
                pending_args = args.iter().map(|v| values[v.0 as usize]).collect();
                current = *target;
            }
            Terminator::Cbr {
                cond,
                t,
                f,
                t_args,
                f_args,
            } => {
                let cond_val = values[cond.0 as usize].as_bool();
                if cond_val {
                    pending_args = t_args.iter().map(|v| values[v.0 as usize]).collect();
                    current = *t;
                } else {
                    pending_args = f_args.iter().map(|v| values[v.0 as usize]).collect();
                    current = *f;
                }
            }
            Terminator::Ret => return,
        }
    }
}

fn eval_pure(op: &PureOp, values: &[Value]) -> Value {
    match op {
        PureOp::ConstI64(v) => Value::I64(*v),
        PureOp::ConstF32(v) => Value::F32(f32::from_bits(*v)),
        PureOp::Add(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64();
            Value::I64(a.wrapping_add(b))
        }
        PureOp::Sub(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64();
            Value::I64(a.wrapping_sub(b))
        }
        PureOp::Mul(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64();
            Value::I64(a.wrapping_mul(b))
        }
        PureOp::Mulh(a, b) => {
            let a = values[a.0 as usize].as_i64() as i128;
            let b = values[b.0 as usize].as_i64() as i128;
            Value::I64((a * b >> 64) as i64)
        }
        PureOp::Mulhu(a, b) => {
            let a = values[a.0 as usize].as_i64() as u128;
            let b = values[b.0 as usize].as_i64() as u128;
            Value::I64((a * b >> 64) as i64)
        }
        PureOp::Mulhsu(a, b) => {
            let a = values[a.0 as usize].as_i64() as i128;
            let b = values[b.0 as usize].as_i64() as u128 as i128;
            Value::I64((a * b >> 64) as i64)
        }
        PureOp::Div(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64();
            if b == 0 {
                Value::I64(-1) // RISC-V division by zero
            } else {
                Value::I64(a / b)
            }
        }
        PureOp::Divu(a, b) => {
            let a = values[a.0 as usize].as_i64() as u64;
            let b = values[b.0 as usize].as_i64() as u64;
            if b == 0 {
                Value::I64(u64::MAX as i64) // RISC-V unsigned division by zero
            } else {
                Value::I64((a / b) as i64)
            }
        }
        PureOp::Rem(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64();
            if b == 0 {
                Value::I64(a) // RISC-V remainder by zero
            } else {
                Value::I64(a % b)
            }
        }
        PureOp::Remu(a, b) => {
            let a = values[a.0 as usize].as_i64() as u64;
            let b = values[b.0 as usize].as_i64() as u64;
            if b == 0 {
                Value::I64(a as i64) // RISC-V unsigned remainder by zero
            } else {
                Value::I64((a % b) as i64)
            }
        }
        PureOp::And(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64();
            Value::I64(a & b)
        }
        PureOp::Or(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64();
            Value::I64(a | b)
        }
        PureOp::Xor(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64();
            Value::I64(a ^ b)
        }
        PureOp::Shl(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64() as u32;
            Value::I64(a << b)
        }
        PureOp::Shr(a, b) => {
            let a = values[a.0 as usize].as_i64() as u64;
            let b = values[b.0 as usize].as_i64() as u32;
            Value::I64((a >> b) as i64)
        }
        PureOp::Sar(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64() as u32;
            Value::I64(a >> b)
        }
        PureOp::Eq(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64();
            Value::I1(a == b)
        }
        PureOp::Ne(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64();
            Value::I1(a != b)
        }
        PureOp::Lt(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64();
            Value::I1(a < b)
        }
        PureOp::Ltu(a, b) => {
            let a = values[a.0 as usize].as_i64() as u64;
            let b = values[b.0 as usize].as_i64() as u64;
            Value::I1(a < b)
        }
        PureOp::Ge(a, b) => {
            let a = values[a.0 as usize].as_i64();
            let b = values[b.0 as usize].as_i64();
            Value::I1(a >= b)
        }
        PureOp::Geu(a, b) => {
            let a = values[a.0 as usize].as_i64() as u64;
            let b = values[b.0 as usize].as_i64() as u64;
            Value::I1(a >= b)
        }
        PureOp::Sext { v, from, to } => {
            let v = values[v.0 as usize].as_i64();
            Value::I64(sext(v, *from, *to))
        }
        PureOp::Zext { v, from, to } => {
            let v = values[v.0 as usize].as_i64();
            Value::I64(zext(v, *from, *to))
        }
        PureOp::Trunc { v, from, to } => {
            let v = values[v.0 as usize].as_i64();
            Value::I64(trunc(v, *from, *to))
        }
        PureOp::Select { cond, t, f } => {
            let c = values[cond.0 as usize].as_bool();
            if c {
                values[t.0 as usize]
            } else {
                values[f.0 as usize]
            }
        }
        // Floating point operations
        PureOp::Fadd(a, b) => {
            let a = values[a.0 as usize].as_f32();
            let b = values[b.0 as usize].as_f32();
            Value::F32(a + b)
        }
        PureOp::Fsub(a, b) => {
            let a = values[a.0 as usize].as_f32();
            let b = values[b.0 as usize].as_f32();
            Value::F32(a - b)
        }
        PureOp::Fmul(a, b) => {
            let a = values[a.0 as usize].as_f32();
            let b = values[b.0 as usize].as_f32();
            Value::F32(a * b)
        }
        PureOp::Fdiv(a, b) => {
            let a = values[a.0 as usize].as_f32();
            let b = values[b.0 as usize].as_f32();
            Value::F32(a / b)
        }
        PureOp::Fsqrt(v) => {
            let v = values[v.0 as usize].as_f32();
            Value::F32(v.sqrt())
        }
        PureOp::Fmin(a, b) => {
            let a = values[a.0 as usize].as_f32();
            let b = values[b.0 as usize].as_f32();
            Value::F32(a.min(b))
        }
        PureOp::Fmax(a, b) => {
            let a = values[a.0 as usize].as_f32();
            let b = values[b.0 as usize].as_f32();
            Value::F32(a.max(b))
        }
        PureOp::Feq(a, b) => {
            let a = values[a.0 as usize].as_f32();
            let b = values[b.0 as usize].as_f32();
            Value::I1(a == b)
        }
        PureOp::Flt(a, b) => {
            let a = values[a.0 as usize].as_f32();
            let b = values[b.0 as usize].as_f32();
            Value::I1(a < b)
        }
        PureOp::Fle(a, b) => {
            let a = values[a.0 as usize].as_f32();
            let b = values[b.0 as usize].as_f32();
            Value::I1(a <= b)
        }
        PureOp::Fsgnj(a, b) => {
            let a = values[a.0 as usize].as_f32();
            let b = values[b.0 as usize].as_f32();
            let sign = b.to_bits() & 0x8000_0000;
            let mag = a.to_bits() & 0x7FFF_FFFF;
            Value::F32(f32::from_bits(sign | mag))
        }
        PureOp::Fsgnjn(a, b) => {
            let a = values[a.0 as usize].as_f32();
            let b = values[b.0 as usize].as_f32();
            let sign = (b.to_bits() & 0x8000_0000) ^ 0x8000_0000;
            let mag = a.to_bits() & 0x7FFF_FFFF;
            Value::F32(f32::from_bits(sign | mag))
        }
        PureOp::Fsgnjx(a, b) => {
            let a = values[a.0 as usize].as_f32();
            let b = values[b.0 as usize].as_f32();
            let sign = (a.to_bits() & 0x8000_0000) ^ (b.to_bits() & 0x8000_0000);
            let mag = a.to_bits() & 0x7FFF_FFFF;
            Value::F32(f32::from_bits(sign | mag))
        }
        PureOp::FcvtF32I32(v) => {
            let v = values[v.0 as usize].as_i64() as i32;
            Value::F32(v as f32)
        }
        PureOp::FcvtF32I64(v) => {
            let v = values[v.0 as usize].as_i64();
            Value::F32(v as f32)
        }
        PureOp::FcvtF32U32(v) => {
            let v = values[v.0 as usize].as_i64() as u32;
            Value::F32(v as f32)
        }
        PureOp::FcvtF32U64(v) => {
            let v = values[v.0 as usize].as_i64() as u64;
            Value::F32(v as f32)
        }
        PureOp::FcvtI32F32(v) => {
            let v = values[v.0 as usize].as_f32();
            Value::I32(v as i32)
        }
        PureOp::FcvtI64F32(v) => {
            let v = values[v.0 as usize].as_f32();
            Value::I64(v as i64)
        }
        PureOp::FcvtU32F32(v) => {
            let v = values[v.0 as usize].as_f32();
            Value::I32(v as u32 as i32)
        }
        PureOp::FcvtU64F32(v) => {
            let v = values[v.0 as usize].as_f32();
            Value::I64(v as u64 as i64)
        }
        PureOp::FmvF32(v) => {
            let v = values[v.0 as usize].as_i64() as u32;
            Value::F32(f32::from_bits(v))
        }
        PureOp::FmvI32(v) => {
            let v = values[v.0 as usize].as_f32();
            Value::I32(v.to_bits() as i32)
        }
    }
}

fn exec_effect<T: Tracer>(op: &EffectOp, values: &mut [Value], vm: &mut VM<T>, io: &mut HostIO) {
    match op {
        EffectOp::GetReg { dst, reg } => match reg {
            Reg::X0
            | Reg::X1
            | Reg::X2
            | Reg::X3
            | Reg::X4
            | Reg::X5
            | Reg::X6
            | Reg::X7
            | Reg::X8
            | Reg::X9
            | Reg::X10
            | Reg::X11
            | Reg::X12
            | Reg::X13
            | Reg::X14
            | Reg::X15
            | Reg::X16
            | Reg::X17
            | Reg::X18
            | Reg::X19
            | Reg::X20
            | Reg::X21
            | Reg::X22
            | Reg::X23
            | Reg::X24
            | Reg::X25
            | Reg::X26
            | Reg::X27
            | Reg::X28
            | Reg::X29
            | Reg::X30
            | Reg::X31 => {
                let idx = reg_index(*reg);
                let val = vm.reg(idx) as i64;
                values[dst.0 as usize] = Value::I64(val);
            }
            Reg::F0
            | Reg::F1
            | Reg::F2
            | Reg::F3
            | Reg::F4
            | Reg::F5
            | Reg::F6
            | Reg::F7
            | Reg::F8
            | Reg::F9
            | Reg::F10
            | Reg::F11
            | Reg::F12
            | Reg::F13
            | Reg::F14
            | Reg::F15
            | Reg::F16
            | Reg::F17
            | Reg::F18
            | Reg::F19
            | Reg::F20
            | Reg::F21
            | Reg::F22
            | Reg::F23
            | Reg::F24
            | Reg::F25
            | Reg::F26
            | Reg::F27
            | Reg::F28
            | Reg::F29
            | Reg::F30
            | Reg::F31 => {
                let idx = freg_index(*reg);
                let val = vm.read_f32(idx);
                values[dst.0 as usize] = Value::F32(val);
            }
        },
        EffectOp::SetReg { reg, val } => match reg {
            Reg::X0
            | Reg::X1
            | Reg::X2
            | Reg::X3
            | Reg::X4
            | Reg::X5
            | Reg::X6
            | Reg::X7
            | Reg::X8
            | Reg::X9
            | Reg::X10
            | Reg::X11
            | Reg::X12
            | Reg::X13
            | Reg::X14
            | Reg::X15
            | Reg::X16
            | Reg::X17
            | Reg::X18
            | Reg::X19
            | Reg::X20
            | Reg::X21
            | Reg::X22
            | Reg::X23
            | Reg::X24
            | Reg::X25
            | Reg::X26
            | Reg::X27
            | Reg::X28
            | Reg::X29
            | Reg::X30
            | Reg::X31 => {
                let idx = reg_index(*reg);
                let val = values[val.0 as usize].as_i64() as u64;
                vm.reg_mut(idx, val);
            }
            Reg::F0
            | Reg::F1
            | Reg::F2
            | Reg::F3
            | Reg::F4
            | Reg::F5
            | Reg::F6
            | Reg::F7
            | Reg::F8
            | Reg::F9
            | Reg::F10
            | Reg::F11
            | Reg::F12
            | Reg::F13
            | Reg::F14
            | Reg::F15
            | Reg::F16
            | Reg::F17
            | Reg::F18
            | Reg::F19
            | Reg::F20
            | Reg::F21
            | Reg::F22
            | Reg::F23
            | Reg::F24
            | Reg::F25
            | Reg::F26
            | Reg::F27
            | Reg::F28
            | Reg::F29
            | Reg::F30
            | Reg::F31 => {
                let idx = freg_index(*reg);
                let val = values[val.0 as usize].as_f32();
                vm.write_f32(idx, val);
            }
        },
        EffectOp::GetCsr { dst, csr } => {
            let val = vm.read_csr(*csr) as i64;
            values[dst.0 as usize] = Value::I64(val);
        }
        EffectOp::SetCsr { csr, val } => {
            let val = values[val.0 as usize].as_i64() as u32;
            vm.set_csr(*csr, val);
        }
        EffectOp::GetPc { dst } => {
            let val = vm.pc() as i64;
            values[dst.0 as usize] = Value::I64(val);
        }
        EffectOp::SetPc { val } => {
            let val = values[val.0 as usize].as_i64() as u64;
            vm.set_pc(val);
        }
        EffectOp::Load8s { dst, addr } => {
            let addr = values[addr.0 as usize].as_i64() as usize;
            let v = vm.load_u8(addr) as i8 as i64;
            values[dst.0 as usize] = Value::I64(v);
        }
        EffectOp::Load8u { dst, addr } => {
            let addr = values[addr.0 as usize].as_i64() as usize;
            let v = vm.load_u8(addr) as u8 as i64;
            values[dst.0 as usize] = Value::I64(v);
        }
        EffectOp::Load16s { dst, addr } => {
            let addr = values[addr.0 as usize].as_i64() as usize;
            let v = vm.load_u16(addr) as i16 as i64;
            values[dst.0 as usize] = Value::I64(v);
        }
        EffectOp::Load16u { dst, addr } => {
            let addr = values[addr.0 as usize].as_i64() as usize;
            let v = vm.load_u16(addr) as u16 as i64;
            values[dst.0 as usize] = Value::I64(v);
        }
        EffectOp::Load32s { dst, addr } => {
            let addr = values[addr.0 as usize].as_i64() as usize;
            let v = vm.load_u32(addr) as i32 as i64;
            values[dst.0 as usize] = Value::I64(v);
        }
        EffectOp::Load32u { dst, addr } => {
            let addr = values[addr.0 as usize].as_i64() as usize;
            let v = vm.load_u32(addr) as u32 as i64;
            values[dst.0 as usize] = Value::I64(v);
        }
        EffectOp::Load64 { dst, addr } => {
            let addr = values[addr.0 as usize].as_i64() as usize;
            let v = vm.load_u64(addr) as i64;
            values[dst.0 as usize] = Value::I64(v);
        }
        EffectOp::Store8 { addr, val } => {
            let addr = values[addr.0 as usize].as_i64() as usize;
            let val = values[val.0 as usize].as_i64() as u8;
            vm.store_u8(addr, val);
        }
        EffectOp::Store16 { addr, val } => {
            let addr = values[addr.0 as usize].as_i64() as usize;
            let val = values[val.0 as usize].as_i64() as u16;
            vm.store_u16(addr, val);
        }
        EffectOp::Store32 { addr, val } => {
            let addr = values[addr.0 as usize].as_i64() as usize;
            let val = values[val.0 as usize].as_i64() as u32;
            vm.store_u32(addr, val);
        }
        EffectOp::Store64 { addr, val } => {
            let addr = values[addr.0 as usize].as_i64() as usize;
            let val = values[val.0 as usize].as_i64() as u64;
            vm.store_u64(addr, val);
        }
        // Floating point load/store
        EffectOp::LoadF32 { dst, addr } => {
            let addr = values[addr.0 as usize].as_i64() as usize;
            let v = vm.load_u32(addr);
            values[dst.0 as usize] = Value::F32(f32::from_bits(v));
        }
        EffectOp::StoreF32 { addr, val } => {
            let addr = values[addr.0 as usize].as_i64() as usize;
            let val = values[val.0 as usize].as_f32().to_bits();
            vm.store_u32(addr, val);
        }
        EffectOp::Ecall => {
            handle_ecall(vm, io);
        }
        EffectOp::Ebreak => {
            vm.halted = true;
            vm.exit_code = 1;
        }
    }
}

fn bool_to_i64(v: bool) -> i64 {
    if v { 1 } else { 0 }
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
        _ => panic!("not an integer register: {:?}", reg),
    }
}

fn freg_index(reg: Reg) -> u8 {
    match reg {
        Reg::F0 => 0,
        Reg::F1 => 1,
        Reg::F2 => 2,
        Reg::F3 => 3,
        Reg::F4 => 4,
        Reg::F5 => 5,
        Reg::F6 => 6,
        Reg::F7 => 7,
        Reg::F8 => 8,
        Reg::F9 => 9,
        Reg::F10 => 10,
        Reg::F11 => 11,
        Reg::F12 => 12,
        Reg::F13 => 13,
        Reg::F14 => 14,
        Reg::F15 => 15,
        Reg::F16 => 16,
        Reg::F17 => 17,
        Reg::F18 => 18,
        Reg::F19 => 19,
        Reg::F20 => 20,
        Reg::F21 => 21,
        Reg::F22 => 22,
        Reg::F23 => 23,
        Reg::F24 => 24,
        Reg::F25 => 25,
        Reg::F26 => 26,
        Reg::F27 => 27,
        Reg::F28 => 28,
        Reg::F29 => 29,
        Reg::F30 => 30,
        Reg::F31 => 31,
        _ => panic!("not a floating point register: {:?}", reg),
    }
}

fn sext(value: i64, from: IrType, to: IrType) -> i64 {
    match (from, to) {
        (IrType::I8, IrType::I16)
        | (IrType::I8, IrType::I32)
        | (IrType::I8, IrType::I64)
        | (IrType::I16, IrType::I32)
        | (IrType::I16, IrType::I64)
        | (IrType::I32, IrType::I64) => {}
        _ => panic!("invalid sext {:?} -> {:?}", from, to),
    }

    match from {
        IrType::I8 => (value as i8) as i64,
        IrType::I16 => (value as i16) as i64,
        IrType::I32 => (value as i32) as i64,
        _ => panic!("invalid sext source {:?}", from),
    }
}

fn zext(value: i64, from: IrType, to: IrType) -> i64 {
    match (from, to) {
        (IrType::I1, IrType::I8)
        | (IrType::I1, IrType::I16)
        | (IrType::I1, IrType::I32)
        | (IrType::I1, IrType::I64)
        | (IrType::I8, IrType::I16)
        | (IrType::I8, IrType::I32)
        | (IrType::I8, IrType::I64)
        | (IrType::I16, IrType::I32)
        | (IrType::I16, IrType::I64)
        | (IrType::I32, IrType::I64) => {}
        _ => panic!("invalid zext {:?} -> {:?}", from, to),
    }

    match from {
        IrType::I1 => value & 1,
        IrType::I8 => (value as u8) as i64,
        IrType::I16 => (value as u16) as i64,
        IrType::I32 => (value as u32) as i64,
        _ => panic!("invalid zext source {:?}", from),
    }
}

fn trunc(value: i64, from: IrType, to: IrType) -> i64 {
    match (from, to) {
        (IrType::I64, IrType::I32) => (value as u32) as i64,
        (IrType::I64, IrType::I16) => (value as u16) as i64,
        (IrType::I64, IrType::I8) => (value as u8) as i64,
        (IrType::I32, IrType::I16) => (value as u16) as i64,
        (IrType::I32, IrType::I8) => (value as u8) as i64,
        (IrType::I16, IrType::I8) => (value as u8) as i64,
        _ => panic!("invalid trunc {:?} -> {:?}", from, to),
    }
}

#[cfg(test)]
mod tests {
    use super::execute_ir;
    use crate::ir::{IrBuilder, Reg};
    use crate::trace::NoopTracer;
    use crate::{HostIO, VM};

    #[test]
    fn execute_ir_updates_registers() {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);

        let ten = builder.const_i64(10);
        builder.set_reg(Reg::X1, ten);

        let x1 = builder.get_reg(Reg::X1);
        let five = builder.const_i64(5);
        let sum = builder.add(x1, five);
        builder.set_reg(Reg::X2, sum);

        builder.ret();
        let func = builder.finish();

        let mut vm = VM::<NoopTracer>::init();
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.reg(1), 10);
        assert_eq!(vm.reg(2), 15);
    }

    #[test]
    fn execute_ir_loads_and_stores_memory() {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);

        let addr = builder.const_i64(0x10);
        let value = builder.const_i64(0x1234_5678);
        builder.store32(addr, value);

        let loaded = builder.load32u(addr);
        builder.set_reg(Reg::X3, loaded);

        builder.ret();
        let func = builder.finish();

        let mut vm = VM::<NoopTracer>::init();
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.load_u32(0x10), 0x1234_5678);
        assert_eq!(vm.reg(3), 0x1234_5678);
    }
}
