use crate::ir2::IrBuilder;

impl IrBuilder {
    pub fn require_single_exit(&self) {
        // C7: require_single_exit asserts current exists and is the only exit.
        if self.current_block.is_none() || self.exit_count != 1 {
            panic!("require_single_exit failed: current or exit_count invalid");
        }
    }
}
