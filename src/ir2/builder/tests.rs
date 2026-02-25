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
        builder.require_single_exit();
        builder.ret();
    }

    #[test]
    #[should_panic(expected = "expected single exit")]
    fn require_single_exit_panics_on_multiple_exits() {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        let _other = builder.block();

        builder.switch_to(entry);
        builder.require_single_exit();
    }

    #[test]
    #[should_panic(expected = "no current block")]
    fn require_single_exit_panics_on_no_current_block() {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        let target = builder.block();

        builder.switch_to(entry);
        builder.br(target, vec![]);
        builder.require_single_exit();
    }

    #[test]
    #[should_panic(expected = "no current block")]
    fn emit_after_ret_panics_no_current_block() {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);
        builder.ret();

        builder.emit_pure(PureOp::ConstI64(1), IrType::I64);
    }

    #[test]
    #[should_panic(expected = "no current block")]
    fn emit_after_br_panics_no_current_block() {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        let target = builder.block();

        builder.switch_to(entry);
        builder.br(target, vec![]);

        builder.emit_pure(PureOp::ConstI64(1), IrType::I64);
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
