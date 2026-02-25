use std::fmt;

use crate::ir2::{IrFunction, Op};

impl fmt::Display for IrFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, block) in self.blocks.iter().enumerate() {
            write!(f, "bb{}(", i)?;
            for (idx, arg) in block.args.iter().enumerate() {
                if idx > 0 {
                    write!(f, ", ")?;
                }
                let ty = self.value_type(*arg);
                write!(f, "v{}: {:?}", arg.0, ty)?;
            }
            writeln!(f, "):")?;

            for op in &block.ops {
                match op {
                    Op::Pure { dst, op } => {
                        writeln!(f, "  v{} = {:?}", dst.0, op)?;
                    }
                    Op::Effect(effect) => {
                        writeln!(f, "  {:?}", effect)?;
                    }
                }
            }

            match &block.term {
                Some(term) => writeln!(f, "  {:?}", term)?,
                None => writeln!(f, "  <no terminator>")?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir2::{Block, EffectOp, IrType, PureOp, Terminator, ValueId};

    #[test]
    fn display_formats_block_and_ops() {
        let mut func = IrFunction::new();
        func.value_types = vec![IrType::I64, IrType::I64, IrType::I1];

        let block = Block {
            args: vec![ValueId(0)],
            ops: vec![
                Op::Pure {
                    dst: ValueId(1),
                    op: PureOp::Add(ValueId(0), ValueId(0)),
                },
                Op::Effect(EffectOp::SetPc { val: ValueId(1) }),
            ],
            term: Some(Terminator::Ret),
        };

        func.blocks.push(block);

        let text = format!("{}", func);
        assert!(text.contains("bb0(v0: I64):"));
        assert!(text.contains("v1 = Add(ValueId(0), ValueId(0))"));
        assert!(text.contains("SetPc"));
        assert!(text.contains("Ret"));
    }
}
