use crate::ecall::handle_ecall;
use crate::ir::{BlockId, EffectOp, IrFunction, IrType, Op, PureOp, Reg, Terminator, ValueId};
use crate::trace::Tracer;
use crate::{HostIO, VM};

pub fn execute_ir<T: Tracer>(func: &IrFunction, vm: &mut VM<T>, io: &mut HostIO) {
    let mut values = vec![0i64; func.value_types.len()];
    let mut current = BlockId(0);
    let mut pending_args: Vec<i64> = Vec::new();

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
                let cond_val = values[cond.0 as usize];
                if cond_val != 0 {
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

fn eval_pure(op: &PureOp, values: &[i64]) -> i64 {
    match op {
        PureOp::ConstI64(v) => *v,
        PureOp::Add(a, b) => values[a.0 as usize].wrapping_add(values[b.0 as usize]),
        PureOp::Sub(a, b) => values[a.0 as usize].wrapping_sub(values[b.0 as usize]),
        PureOp::Mul(a, b) => values[a.0 as usize].wrapping_mul(values[b.0 as usize]),
        PureOp::Mulh(a, b) => {
            let a = values[a.0 as usize] as i128;
            let b = values[b.0 as usize] as i128;
            (a * b >> 64) as i64
        }
        PureOp::Mulhu(a, b) => {
            let a = values[a.0 as usize] as u64 as u128;
            let b = values[b.0 as usize] as u64 as u128;
            (a * b >> 64) as i64
        }
        PureOp::Mulhsu(a, b) => {
            let a = values[a.0 as usize] as i128;
            let b = values[b.0 as usize] as u64 as i128;
            (a * b >> 64) as i64
        }
        PureOp::Div(a, b) => {
            let b_val = values[b.0 as usize];
            if b_val == 0 {
                -1 // RISC-V division by zero
            } else if values[a.0 as usize] == i64::MIN && b_val == -1 {
                i64::MIN // RISC-V overflow case
            } else {
                values[a.0 as usize] / b_val
            }
        }
        PureOp::Divu(a, b) => {
            let b_val = values[b.0 as usize] as u64;
            if b_val == 0 {
                u64::MAX as i64 // RISC-V unsigned division by zero
            } else {
                ((values[a.0 as usize] as u64) / b_val) as i64
            }
        }
        PureOp::Rem(a, b) => {
            let b_val = values[b.0 as usize];
            if b_val == 0 {
                values[a.0 as usize] // RISC-V remainder by zero
            } else if values[a.0 as usize] == i64::MIN && b_val == -1 {
                0 // RISC-V overflow case
            } else {
                values[a.0 as usize] % b_val
            }
        }
        PureOp::Remu(a, b) => {
            let b_val = values[b.0 as usize] as u64;
            if b_val == 0 {
                values[a.0 as usize] // RISC-V unsigned remainder by zero
            } else {
                ((values[a.0 as usize] as u64) % b_val) as i64
            }
        }
        PureOp::And(a, b) => values[a.0 as usize] & values[b.0 as usize],
        PureOp::Or(a, b) => values[a.0 as usize] | values[b.0 as usize],
        PureOp::Xor(a, b) => values[a.0 as usize] ^ values[b.0 as usize],
        PureOp::Shl(a, b) => {
            let sh = values[b.0 as usize] as u32;
            values[a.0 as usize] << sh
        }
        PureOp::Shr(a, b) => {
            let sh = values[b.0 as usize] as u32;
            ((values[a.0 as usize] as u64) >> sh) as i64
        }
        PureOp::Sar(a, b) => {
            let sh = values[b.0 as usize] as u32;
            values[a.0 as usize] >> sh
        }
        PureOp::Eq(a, b) => bool_to_i64(values[a.0 as usize] == values[b.0 as usize]),
        PureOp::Ne(a, b) => bool_to_i64(values[a.0 as usize] != values[b.0 as usize]),
        PureOp::Lt(a, b) => bool_to_i64(values[a.0 as usize] < values[b.0 as usize]),
        PureOp::Ltu(a, b) => {
            bool_to_i64((values[a.0 as usize] as u64) < (values[b.0 as usize] as u64))
        }
        PureOp::Ge(a, b) => bool_to_i64(values[a.0 as usize] >= values[b.0 as usize]),
        PureOp::Geu(a, b) => {
            bool_to_i64((values[a.0 as usize] as u64) >= (values[b.0 as usize] as u64))
        }
        PureOp::Sext { v, from, to } => sext(values[v.0 as usize], *from, *to),
        PureOp::Zext { v, from, to } => zext(values[v.0 as usize], *from, *to),
        PureOp::Trunc { v, from, to } => trunc(values[v.0 as usize], *from, *to),
        PureOp::Select { cond, t, f } => {
            let c = values[cond.0 as usize];
            if c != 0 {
                values[t.0 as usize]
            } else {
                values[f.0 as usize]
            }
        }
    }
}

fn exec_effect<T: Tracer>(op: &EffectOp, values: &mut [i64], vm: &mut VM<T>, io: &mut HostIO) {
    match op {
        EffectOp::GetReg { dst, reg } => {
            let idx = reg_index(*reg);
            values[dst.0 as usize] = vm.reg(idx) as i64;
        }
        EffectOp::SetReg { reg, val } => {
            let idx = reg_index(*reg);
            vm.reg_mut(idx, values[val.0 as usize] as u64);
        }
        EffectOp::GetCsr { dst, csr } => {
            values[dst.0 as usize] = vm.read_csr(*csr) as i64;
        }
        EffectOp::SetCsr { csr, val } => {
            vm.set_csr(*csr, values[val.0 as usize] as u32);
        }
        EffectOp::GetPc { dst } => {
            values[dst.0 as usize] = vm.pc() as i64;
        }
        EffectOp::SetPc { val } => {
            vm.set_pc(values[val.0 as usize] as u64);
        }
        EffectOp::Load8s { dst, addr } => {
            let addr = values[addr.0 as usize] as usize;
            let v = vm.load_u8(addr) as i8 as i64;
            values[dst.0 as usize] = v;
        }
        EffectOp::Load8u { dst, addr } => {
            let addr = values[addr.0 as usize] as usize;
            let v = vm.load_u8(addr) as u8 as i64;
            values[dst.0 as usize] = v;
        }
        EffectOp::Load16s { dst, addr } => {
            let addr = values[addr.0 as usize] as usize;
            let v = vm.load_u16(addr) as i16 as i64;
            values[dst.0 as usize] = v;
        }
        EffectOp::Load16u { dst, addr } => {
            let addr = values[addr.0 as usize] as usize;
            let v = vm.load_u16(addr) as u16 as i64;
            values[dst.0 as usize] = v;
        }
        EffectOp::Load32s { dst, addr } => {
            let addr = values[addr.0 as usize] as usize;
            let v = vm.load_u32(addr) as i32 as i64;
            values[dst.0 as usize] = v;
        }
        EffectOp::Load32u { dst, addr } => {
            let addr = values[addr.0 as usize] as usize;
            let v = vm.load_u32(addr) as u32 as i64;
            values[dst.0 as usize] = v;
        }
        EffectOp::Load64 { dst, addr } => {
            let addr = values[addr.0 as usize] as usize;
            let v = vm.load_u64(addr) as i64;
            values[dst.0 as usize] = v;
        }
        EffectOp::Store8 { addr, val } => {
            let addr = values[addr.0 as usize] as usize;
            vm.store_u8(addr, values[val.0 as usize] as u8);
        }
        EffectOp::Store16 { addr, val } => {
            let addr = values[addr.0 as usize] as usize;
            vm.store_u16(addr, values[val.0 as usize] as u16);
        }
        EffectOp::Store32 { addr, val } => {
            let addr = values[addr.0 as usize] as usize;
            vm.store_u32(addr, values[val.0 as usize] as u32);
        }
        EffectOp::Store64 { addr, val } => {
            let addr = values[addr.0 as usize] as usize;
            vm.store_u64(addr, values[val.0 as usize] as u64);
        }
        EffectOp::Ecall | EffectOp::Ebreak => {
            handle_ecall(vm, io);
        }
    }
}

fn bool_to_i64(v: bool) -> i64 {
    if v {
        1
    } else {
        0
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

fn sext(value: i64, from: IrType, to: IrType) -> i64 {
    match (from, to) {
        (IrType::I1, IrType::I64)
        | (IrType::I8, IrType::I16)
        | (IrType::I8, IrType::I32)
        | (IrType::I8, IrType::I64)
        | (IrType::I16, IrType::I32)
        | (IrType::I16, IrType::I64)
        | (IrType::I32, IrType::I64) => {}
        _ => panic!("invalid sext {:?} -> {:?}", from, to),
    }

    match from {
        IrType::I1 => {
            if value != 0 {
                1
            } else {
                0
            }
        }
        IrType::I8 => (value as i8) as i64,
        IrType::I16 => (value as i16) as i64,
        IrType::I32 => (value as i32) as i64,
        _ => panic!("invalid sext source {:?}", from),
    }
}

fn zext(value: i64, from: IrType, to: IrType) -> i64 {
    match (from, to) {
        (IrType::I1, IrType::I64)
        | (IrType::I8, IrType::I16)
        | (IrType::I8, IrType::I32)
        | (IrType::I8, IrType::I64)
        | (IrType::I16, IrType::I32)
        | (IrType::I16, IrType::I64)
        | (IrType::I32, IrType::I64) => {}
        _ => panic!("invalid zext {:?} -> {:?}", from, to),
    }

    match from {
        IrType::I1 => {
            if value != 0 {
                1
            } else {
                0
            }
        }
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
