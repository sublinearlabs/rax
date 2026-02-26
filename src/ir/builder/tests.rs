#[cfg(test)]
mod tests {
    use crate::ir::IrBuilder;

    #[test]
    fn build_single_block_with_ret() {
        // Proof sketch:
        // - C1: Emitting ops/terminators requires a current block.
        // - C6: ret terminates the current block, keeps it in exits, and clears current.
        // Therefore: a single block can be built and terminated with ret.
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);
        builder.const_i64(7);
        builder.ret();

        let func = builder.finish();
        assert_eq!(func.blocks.len(), 1);
        assert!(func.blocks[0].term.is_some());
    }

    #[test]
    fn require_single_exit_ok() {
        // Proof sketch:
        // - C4: Every new block starts as an exit.
        // - C7: require_single_exit asserts current exists and is the only exit.
        // Therefore: with a single block, require_single_exit succeeds before ret.
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);
        builder.const_i64(0);
        builder.require_single_exit();
        builder.ret();
    }

    #[test]
    #[should_panic(expected = "require_single_exit failed")]
    fn require_single_exit_panics_on_multiple_exits() {
        // Proof sketch:
        // - C4: Every new block starts as an exit.
        // - C7: require_single_exit asserts current exists and is the only exit.
        // Therefore: creating a second block makes exits>1 and require_single_exit panics.
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        let _other = builder.block();

        builder.switch_to(entry);
        builder.require_single_exit();
    }

    #[test]
    #[should_panic(expected = "require_single_exit failed")]
    fn require_single_exit_panics_on_no_current_block() {
        // Proof sketch:
        // - C5: br/cbr terminate the current block, remove it from exits, and clear current.
        // - C7: require_single_exit asserts current exists and is the only exit.
        // Therefore: after br, require_single_exit panics due to no current block.
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
        // Proof sketch:
        // - C6: ret terminates the current block, keeps it in exits, and clears current.
        // - C1: Emitting ops/terminators requires a current block.
        // Therefore: emitting after ret must panic.
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);
        builder.ret();

        builder.const_i64(1);
    }

    #[test]
    #[should_panic(expected = "no current block")]
    fn emit_after_br_panics_no_current_block() {
        // Proof sketch:
        // - C5: br/cbr terminate the current block, remove it from exits, and clear current.
        // - C1: Emitting ops/terminators requires a current block.
        // Therefore: emitting after br must panic.
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        let target = builder.block();

        builder.switch_to(entry);
        builder.br(target, vec![]);

        builder.const_i64(1);
    }

    #[test]
    #[should_panic(expected = "cannot switch to terminated block")]
    fn switch_to_panics_on_terminated_block() {
        // Proof sketch:
        // - C6: ret terminates the current block, keeps it in exits, and clears current.
        // - C3: Switching to a terminated block is forbidden.
        // Therefore: switching back to a ret-terminated block must panic.
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);
        builder.ret();

        builder.switch_to(entry);
    }
}
