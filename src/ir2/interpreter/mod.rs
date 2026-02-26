use crate::ir2::{BlockId, IrFunction, Op, Terminator};
use crate::trace::Tracer;
use crate::{HostIO, VM};

mod effect;
mod pure;

use effect::exec_effect;
use pure::eval_pure;

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

#[cfg(test)]
mod tests {
    use super::execute_ir;
    use crate::decode::{B, I, Instruction};
    use crate::ir2::IrBuilder;
    use crate::ir2::lower::lower_instruction_into;
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
