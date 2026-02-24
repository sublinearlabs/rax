use crate::decode::Instruction;
use crate::ir::{AtomicRmwOp, AtomicWidth, IrBuilder, IrFunction, IrType};

pub(crate) fn lower_a(insn: &Instruction, _current_pc: u64, _next_pc: u64) -> IrFunction {
    let mut builder = IrBuilder::new();
    let entry = builder.block();
    builder.switch_to(entry);

    match insn {
        Instruction::LrW(r) => {
            let addr = builder.reg(r.rs1);
            let value = builder.lr_w(addr);
            builder.set_reg_idx(r.rd, value);
            builder.ret();
        }
        Instruction::LrD(r) => {
            let addr = builder.reg(r.rs1);
            let value = builder.lr_d(addr);
            builder.set_reg_idx(r.rd, value);
            builder.ret();
        }
        Instruction::ScW(r) => {
            let addr = builder.reg(r.rs1);
            let value = builder.reg(r.rs2);
            let result = builder.sc_w(addr, value);
            builder.set_reg_idx(r.rd, result);
            builder.ret();
        }
        Instruction::ScD(r) => {
            let addr = builder.reg(r.rs1);
            let value = builder.reg(r.rs2);
            let result = builder.sc_d(addr, value);
            builder.set_reg_idx(r.rd, result);
            builder.ret();
        }
        Instruction::AmoSwapW(r) => amo_w(&mut builder, r, AtomicRmwOp::Xchg),
        Instruction::AmoAddW(r) => amo_w(&mut builder, r, AtomicRmwOp::Add),
        Instruction::AmoXorW(r) => amo_w(&mut builder, r, AtomicRmwOp::Xor),
        Instruction::AmoAndW(r) => amo_w(&mut builder, r, AtomicRmwOp::And),
        Instruction::AmoOrW(r) => amo_w(&mut builder, r, AtomicRmwOp::Or),
        Instruction::AmoMinW(r) => amo_w(&mut builder, r, AtomicRmwOp::Min),
        Instruction::AmoMaxW(r) => amo_w(&mut builder, r, AtomicRmwOp::Max),
        Instruction::AmoMinuW(r) => amo_w(&mut builder, r, AtomicRmwOp::Umin),
        Instruction::AmoMaxuW(r) => amo_w(&mut builder, r, AtomicRmwOp::Umax),
        Instruction::AmoSwapD(r) => amo_d(&mut builder, r, AtomicRmwOp::Xchg),
        Instruction::AmoAddD(r) => amo_d(&mut builder, r, AtomicRmwOp::Add),
        Instruction::AmoXorD(r) => amo_d(&mut builder, r, AtomicRmwOp::Xor),
        Instruction::AmoAndD(r) => amo_d(&mut builder, r, AtomicRmwOp::And),
        Instruction::AmoOrD(r) => amo_d(&mut builder, r, AtomicRmwOp::Or),
        Instruction::AmoMinD(r) => amo_d(&mut builder, r, AtomicRmwOp::Min),
        Instruction::AmoMaxD(r) => amo_d(&mut builder, r, AtomicRmwOp::Max),
        Instruction::AmoMinuD(r) => amo_d(&mut builder, r, AtomicRmwOp::Umin),
        Instruction::AmoMaxuD(r) => amo_d(&mut builder, r, AtomicRmwOp::Umax),
        _ => panic!("IR lowering missing for A instruction {:?}", insn),
    }

    builder.finish()
}

fn amo_w(builder: &mut IrBuilder, r: &crate::decode::R, op: AtomicRmwOp) {
    let addr = builder.reg(r.rs1);
    let value = builder.reg(r.rs2);
    let read = builder.atomic_rmw(op, AtomicWidth::W, addr, value);
    let read64 = builder.sext(read, IrType::I32, IrType::I64);
    builder.set_reg_idx(r.rd, read64);
    builder.ret();
}

fn amo_d(builder: &mut IrBuilder, r: &crate::decode::R, op: AtomicRmwOp) {
    let addr = builder.reg(r.rs1);
    let value = builder.reg(r.rs2);
    let read = builder.atomic_rmw(op, AtomicWidth::D, addr, value);
    builder.set_reg_idx(r.rd, read);
    builder.ret();
}

#[cfg(test)]
mod tests {
    use super::lower_a;
    use crate::decode::{Instruction, R};
    use crate::ir::execute_ir;
    use crate::trace::NoopTracer;
    use crate::{HostIO, VM};

    #[test]
    fn lower_lr_sc_w_round_trip() {
        let lr = Instruction::LrW(R {
            rd: 1,
            rs1: 2,
            rs2: 0,
        });
        let sc = Instruction::ScW(R {
            rd: 3,
            rs1: 2,
            rs2: 4,
        });

        let func_lr = lower_a(&lr, 0, 4);
        let func_sc = lower_a(&sc, 0, 4);

        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(2, 0x10);
        vm.reg_mut(4, 7);
        vm.store_u32(0x10, 1);
        let mut io = HostIO::new();
        execute_ir(&func_lr, &mut vm, &mut io);
        execute_ir(&func_sc, &mut vm, &mut io);

        assert_eq!(vm.load_u32(0x10), 7);
        assert_eq!(vm.reg(3), 0);
    }

    #[test]
    fn lower_amo_add_d_updates_memory() {
        let amo = Instruction::AmoAddD(R {
            rd: 1,
            rs1: 2,
            rs2: 3,
        });

        let func = lower_a(&amo, 0, 4);

        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(2, 0x20);
        vm.reg_mut(3, 5);
        vm.store_u64(0x20, 10);
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.load_u64(0x20), 15);
        assert_eq!(vm.reg(1), 10);
    }
}
