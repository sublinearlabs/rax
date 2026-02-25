use crate::ir2::IrBuilder;

impl IrBuilder {
    pub fn require_single_exit(&self) {
        let block = self.current_block.expect("no current block");
        if self.exit_count != 1 {
            panic!("expected single exit, found {}", self.exit_count);
        }
        let idx = block.0 as usize;
        if !self.exit_flags.get(idx).copied().unwrap_or(false) {
            panic!("current block is not an exit");
        }
    }
}
