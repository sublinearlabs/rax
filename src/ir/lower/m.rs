use crate::decode::Instruction;
use crate::ir::{IrBuilder, IrFunction, IrType, ValueId};

pub(crate) fn lower_m(insn: &Instruction, _current_pc: u64, _next_pc: u64) -> IrFunction {
    let mut builder = IrBuilder::new();
    let entry = builder.block();
    builder.switch_to(entry);

    match insn {
        // Multiplication
        Instruction::Mul(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.mul(rs1, rs2); // Lower 64 bits
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Mulw(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let rs1_trunc = trunc_i32(&mut builder, rs1);
            let rs2_trunc = trunc_i32(&mut builder, rs2);
            let a = sext_i32(&mut builder, rs1_trunc);
            let b = sext_i32(&mut builder, rs2_trunc);
            let prod = builder.mul(a, b);
            let trunc = trunc_i32(&mut builder, prod);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Mulh(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.mulh(rs1, rs2); // Upper 64 bits (signed)
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Mulhu(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.mulhu(rs1, rs2); // Upper 64 bits (unsigned)
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Mulhsu(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.mulhsu(rs1, rs2); // Upper 64 bits (signed × unsigned)
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }

        // Division
        Instruction::Div(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.div(rs1, rs2); // Signed division
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Divw(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let rs1_trunc = trunc_i32(&mut builder, rs1);
            let rs2_trunc = trunc_i32(&mut builder, rs2);
            let a = sext_i32(&mut builder, rs1_trunc);
            let b = sext_i32(&mut builder, rs2_trunc);
            let quot = builder.div(a, b);
            let trunc = trunc_i32(&mut builder, quot);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Divu(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.divu(rs1, rs2); // Unsigned division
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Divuw(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let rs1_trunc = trunc_i32(&mut builder, rs1);
            let rs2_trunc = trunc_i32(&mut builder, rs2);
            let a = zext_i32(&mut builder, rs1_trunc);
            let b = zext_i32(&mut builder, rs2_trunc);
            let quot = builder.divu(a, b);
            let trunc = trunc_i32(&mut builder, quot);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }

        // Remainder
        Instruction::Rem(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.rem(rs1, rs2); // Signed remainder
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Remw(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let rs1_trunc = trunc_i32(&mut builder, rs1);
            let rs2_trunc = trunc_i32(&mut builder, rs2);
            let a = sext_i32(&mut builder, rs1_trunc);
            let b = sext_i32(&mut builder, rs2_trunc);
            let rem = builder.rem(a, b);
            let trunc = trunc_i32(&mut builder, rem);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Remu(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let v = builder.remu(rs1, rs2); // Unsigned remainder
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }
        Instruction::Remuw(r) => {
            let rs1 = builder.reg(r.rs1);
            let rs2 = builder.reg(r.rs2);
            let rs1_trunc = trunc_i32(&mut builder, rs1);
            let rs2_trunc = trunc_i32(&mut builder, rs2);
            let a = zext_i32(&mut builder, rs1_trunc);
            let b = zext_i32(&mut builder, rs2_trunc);
            let rem = builder.remu(a, b);
            let trunc = trunc_i32(&mut builder, rem);
            let v = sext_i32(&mut builder, trunc);
            builder.set_reg_idx(r.rd, v);
            builder.ret();
        }

        _ => panic!("IR lowering missing for M instruction {:?}", insn),
    }

    builder.finish()
}

fn trunc_i32(builder: &mut IrBuilder, value: ValueId) -> ValueId {
    builder.trunc(value, IrType::I64, IrType::I32)
}

fn sext_i32(builder: &mut IrBuilder, value: ValueId) -> ValueId {
    builder.sext(value, IrType::I32, IrType::I64)
}

fn zext_i32(builder: &mut IrBuilder, value: ValueId) -> ValueId {
    builder.zext(value, IrType::I32, IrType::I64)
}

#[cfg(test)]
mod tests {
    use super::lower_m;
    use crate::decode::{Instruction, R};
    use crate::ir::execute_ir;
    use crate::trace::NoopTracer;
    use crate::{HostIO, VM};

    #[test]
    fn lower_mul_sets_register() {
        let insn = Instruction::Mul(R {
            rd: 3,
            rs1: 1,
            rs2: 2,
        });
        let func = lower_m(&insn, 0, 4);

        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(1, 10);
        vm.reg_mut(2, 20);
        vm.set_pc(4);
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.reg(3), 200); // 10 * 20
    }

    #[test]
    fn lower_div_sets_register() {
        let insn = Instruction::Div(R {
            rd: 3,
            rs1: 1,
            rs2: 2,
        });
        let func = lower_m(&insn, 0, 4);

        let mut vm = VM::<NoopTracer>::init();
        vm.reg_mut(1, 20);
        vm.reg_mut(2, 10);
        vm.set_pc(4);
        let mut io = HostIO::new();
        execute_ir(&func, &mut vm, &mut io);

        assert_eq!(vm.reg(3), 2); // 20 / 10
    }
}
