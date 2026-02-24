use crate::decode::Instruction;
use crate::ir::{IrBuilder, IrFunction};

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
        Instruction::AmoSwapW(r) => amo_w(&mut builder, r, IrAmoOp::Swap),
        Instruction::AmoAddW(r) => amo_w(&mut builder, r, IrAmoOp::Add),
        Instruction::AmoXorW(r) => amo_w(&mut builder, r, IrAmoOp::Xor),
        Instruction::AmoAndW(r) => amo_w(&mut builder, r, IrAmoOp::And),
        Instruction::AmoOrW(r) => amo_w(&mut builder, r, IrAmoOp::Or),
        Instruction::AmoMinW(r) => amo_w(&mut builder, r, IrAmoOp::Min),
        Instruction::AmoMaxW(r) => amo_w(&mut builder, r, IrAmoOp::Max),
        Instruction::AmoMinuW(r) => amo_w(&mut builder, r, IrAmoOp::Minu),
        Instruction::AmoMaxuW(r) => amo_w(&mut builder, r, IrAmoOp::Maxu),
        Instruction::AmoSwapD(r) => amo_d(&mut builder, r, IrAmoOp::Swap),
        Instruction::AmoAddD(r) => amo_d(&mut builder, r, IrAmoOp::Add),
        Instruction::AmoXorD(r) => amo_d(&mut builder, r, IrAmoOp::Xor),
        Instruction::AmoAndD(r) => amo_d(&mut builder, r, IrAmoOp::And),
        Instruction::AmoOrD(r) => amo_d(&mut builder, r, IrAmoOp::Or),
        Instruction::AmoMinD(r) => amo_d(&mut builder, r, IrAmoOp::Min),
        Instruction::AmoMaxD(r) => amo_d(&mut builder, r, IrAmoOp::Max),
        Instruction::AmoMinuD(r) => amo_d(&mut builder, r, IrAmoOp::Minu),
        Instruction::AmoMaxuD(r) => amo_d(&mut builder, r, IrAmoOp::Maxu),
        _ => panic!("IR lowering missing for A instruction {:?}", insn),
    }

    builder.finish()
}

#[derive(Clone, Copy)]
enum IrAmoOp {
    Swap,
    Add,
    Xor,
    And,
    Or,
    Min,
    Max,
    Minu,
    Maxu,
}

fn amo_w(builder: &mut IrBuilder, r: &crate::decode::R, op: IrAmoOp) {
    let addr = builder.reg(r.rs1);
    let value = builder.reg(r.rs2);
    let read = match op {
        IrAmoOp::Swap => builder.amo_swap_w(addr, value),
        IrAmoOp::Add => builder.amo_add_w(addr, value),
        IrAmoOp::Xor => builder.amo_xor_w(addr, value),
        IrAmoOp::And => builder.amo_and_w(addr, value),
        IrAmoOp::Or => builder.amo_or_w(addr, value),
        IrAmoOp::Min => builder.amo_min_w(addr, value),
        IrAmoOp::Max => builder.amo_max_w(addr, value),
        IrAmoOp::Minu => builder.amo_minu_w(addr, value),
        IrAmoOp::Maxu => builder.amo_maxu_w(addr, value),
    };
    builder.set_reg_idx(r.rd, read);
    builder.ret();
}

fn amo_d(builder: &mut IrBuilder, r: &crate::decode::R, op: IrAmoOp) {
    let addr = builder.reg(r.rs1);
    let value = builder.reg(r.rs2);
    let read = match op {
        IrAmoOp::Swap => builder.amo_swap_d(addr, value),
        IrAmoOp::Add => builder.amo_add_d(addr, value),
        IrAmoOp::Xor => builder.amo_xor_d(addr, value),
        IrAmoOp::And => builder.amo_and_d(addr, value),
        IrAmoOp::Or => builder.amo_or_d(addr, value),
        IrAmoOp::Min => builder.amo_min_d(addr, value),
        IrAmoOp::Max => builder.amo_max_d(addr, value),
        IrAmoOp::Minu => builder.amo_minu_d(addr, value),
        IrAmoOp::Maxu => builder.amo_maxu_d(addr, value),
    };
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
