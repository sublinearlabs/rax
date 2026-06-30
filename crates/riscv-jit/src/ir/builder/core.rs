use crate::ir::{BlockId, EffectOp, IrFunction, IrType, Op, PureOp, Terminator, ValueId};

// Builder constraints (C1-C7):
// C1: Emitting ops/terminators requires a current block.
// C2: A block's terminator is set once.
// C3: Switching to a terminated block is forbidden.
// C4: Every new block starts as an exit.
// C5: br/cbr terminate the current block, remove it from exits, and clear current.
// C6: ret terminates the current block, keeps it in exits, and clears current.
// C7: require_single_exit asserts current exists and is the only exit.

pub struct IrBuilder {
    pub(crate) func: IrFunction,
    pub(crate) current_block: Option<BlockId>,
    pub(crate) exit_flags: Vec<bool>,
    pub(crate) exit_count: usize,
}

impl IrBuilder {
    pub fn new() -> Self {
        Self {
            func: IrFunction::new(),
            current_block: None,
            exit_flags: Vec::new(),
            exit_count: 0,
        }
    }

    pub fn finish(self) -> IrFunction {
        self.func
    }

    pub(crate) fn new_value(&mut self, ty: IrType) -> ValueId {
        let id = ValueId(self.func.value_types.len() as u32);
        self.func.value_types.push(ty);
        id
    }

    pub(crate) fn value_type(&self, v: ValueId) -> IrType {
        self.func.value_types[v.0 as usize]
    }

    pub(crate) fn push_op(&mut self, op: Op) {
        // C1: Emitting ops/terminators requires a current block.
        let block = self.current_block.expect("no current block");
        self.func.blocks[block.0 as usize].ops.push(op);
    }

    pub(crate) fn set_term(&mut self, term: Terminator) {
        // C1: Emitting ops/terminators requires a current block.
        // C2: A block's terminator is set once.
        // C5: br/cbr terminate the current block, remove it from exits, and clear current.
        // C6: ret terminates the current block, keeps it in exits, and clears current.
        let block = self.current_block.expect("no current block");
        let idx = block.0 as usize;
        let is_branch = matches!(term, Terminator::Br { .. } | Terminator::Cbr { .. });
        let block = &mut self.func.blocks[idx];
        block.term = Some(term);
        if is_branch {
            if self.exit_flags.get(idx).copied().unwrap_or(false) {
                self.exit_flags[idx] = false;
                self.exit_count = self.exit_count.saturating_sub(1);
            }
        }
        self.current_block = None;
    }

    pub fn emit_pure(&mut self, op: PureOp, ty: IrType) -> ValueId {
        let dst = self.new_value(ty);
        self.push_op(Op::Pure { dst, op });
        dst
    }

    pub fn emit_effect(&mut self, op: EffectOp) {
        self.push_op(Op::Effect(op));
    }

    pub fn br(&mut self, target: BlockId, args: Vec<ValueId>) {
        self.check_block_args(target, &args);
        self.set_term(Terminator::Br { target, args });
    }

    pub fn cbr(
        &mut self,
        cond: ValueId,
        t: BlockId,
        f: BlockId,
        t_args: Vec<ValueId>,
        f_args: Vec<ValueId>,
    ) {
        self.expect_type(cond, IrType::I1);
        self.check_block_args(t, &t_args);
        self.check_block_args(f, &f_args);
        self.set_term(Terminator::Cbr {
            cond,
            t,
            f,
            t_args,
            f_args,
        });
    }

    pub fn ret(&mut self) {
        self.set_term(Terminator::Ret);
    }
}
