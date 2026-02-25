use crate::ir2::{Block, BlockId, IrBuilder, IrType, ValueId};

impl IrBuilder {
    pub fn block_with_args(&mut self, arg_types: &[IrType]) -> BlockId {
        let mut args = Vec::with_capacity(arg_types.len());
        for &ty in arg_types {
            args.push(self.new_value(ty));
        }
        let id = BlockId(self.func.blocks.len() as u32);
        self.func.blocks.push(Block {
            args,
            ops: Vec::new(),
            term: None,
        });
        self.exit_flags.push(true);
        self.exit_count += 1;
        id
    }

    pub fn block(&mut self) -> BlockId {
        self.block_with_args(&[])
    }

    pub fn block_arg(&self, block: BlockId, index: usize) -> ValueId {
        self.func.blocks[block.0 as usize].args[index]
    }

    pub fn switch_to(&mut self, block: BlockId) {
        let target = &self.func.blocks[block.0 as usize];
        if target.term.is_some() {
            panic!("cannot switch to terminated block");
        }
        self.current_block = Some(block);
    }
}
