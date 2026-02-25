#[cfg(test)]
mod tests {
    use crate::ir2::{IrBuilder, IrType, PureOp};

    #[test]
    fn build_single_block_with_ret() {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);
        builder.emit_pure(PureOp::ConstI64(7), IrType::I64);
        builder.ret();

        let func = builder.finish();
        assert_eq!(func.blocks.len(), 1);
        assert!(func.blocks[0].term.is_some());
    }

    #[test]
    fn require_single_exit_ok() {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);
        builder.emit_pure(PureOp::ConstI64(0), IrType::I64);
        builder.ret();
        builder.require_single_exit();
    }

    #[test]
    #[should_panic(expected = "expected single exit")]
    fn require_single_exit_panics_on_multiple_exits() {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        let then_block = builder.block();
        let else_block = builder.block();

        builder.switch_to(entry);
        let cond = builder.emit_pure(PureOp::ConstI64(1), IrType::I1);
        builder.cbr(cond, then_block, else_block, vec![], vec![]);

        builder.switch_to(then_block);
        builder.ret();
        builder.switch_to(else_block);
        builder.ret();

        builder.require_single_exit();
    }

    #[test]
    #[should_panic(expected = "current block is not an exit")]
    fn require_single_exit_panics_on_non_exit_current_block() {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        let target = builder.block();

        builder.switch_to(entry);
        builder.br(target, vec![]);
        builder.switch_to(entry);

        builder.require_single_exit();
    }

    #[test]
    #[should_panic(expected = "cannot switch to terminated block")]
    fn switch_to_panics_on_terminated_block() {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);
        builder.ret();

        builder.switch_to(entry);
    }
}
