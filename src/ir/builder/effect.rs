use crate::ir::{AtomicRmwOp, AtomicWidth, EffectOp, IrBuilder, IrType, MemWidth, Reg, ValueId};

impl IrBuilder {
    pub fn get_reg(&mut self, reg: Reg) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.emit_effect(EffectOp::GetReg { dst, reg });
        dst
    }

    pub fn set_reg(&mut self, reg: Reg, val: ValueId) {
        self.expect_type(val, IrType::I64);
        self.emit_effect(EffectOp::SetReg { reg, val });
    }

    pub fn get_pc(&mut self) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.emit_effect(EffectOp::GetPc { dst });
        dst
    }

    pub fn set_pc(&mut self, val: ValueId) {
        self.expect_type(val, IrType::I64);
        self.emit_effect(EffectOp::SetPc { val });
    }

    pub fn get_csr(&mut self, csr: u32) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.emit_effect(EffectOp::GetCsr { dst, csr });
        dst
    }

    pub fn set_csr(&mut self, csr: u32, val: ValueId) {
        self.expect_type(val, IrType::I64);
        self.emit_effect(EffectOp::SetCsr { csr, val });
    }

    pub fn load(&mut self, addr: ValueId, width: MemWidth, ty: IrType) -> ValueId {
        self.expect_type(addr, IrType::I64);
        self.expect_mem_width_type(width, ty);
        let dst = self.new_value(ty);
        self.emit_effect(EffectOp::Load { dst, addr, width });
        dst
    }

    pub fn store(&mut self, addr: ValueId, val: ValueId, width: MemWidth) {
        self.expect_type(addr, IrType::I64);
        self.expect_mem_width_value(width, val);
        self.emit_effect(EffectOp::Store { addr, val, width });
    }

    pub fn load_reserved(&mut self, addr: ValueId, width: AtomicWidth, ty: IrType) -> ValueId {
        self.expect_type(addr, IrType::I64);
        self.expect_atomic_width_type(width, ty);
        let dst = self.new_value(ty);
        self.emit_effect(EffectOp::LoadReserved { dst, addr, width });
        dst
    }

    pub fn store_conditional(
        &mut self,
        addr: ValueId,
        val: ValueId,
        width: AtomicWidth,
        ty: IrType,
    ) -> ValueId {
        self.expect_type(addr, IrType::I64);
        self.expect_atomic_width_value(width, val);
        self.expect_atomic_width_type(width, ty);
        let dst = self.new_value(ty);
        self.emit_effect(EffectOp::StoreConditional {
            dst,
            addr,
            val,
            width,
        });
        dst
    }

    pub fn atomic_rmw(
        &mut self,
        op: AtomicRmwOp,
        width: AtomicWidth,
        addr: ValueId,
        val: ValueId,
        ty: IrType,
    ) -> ValueId {
        self.expect_type(addr, IrType::I64);
        self.expect_atomic_width_value(width, val);
        self.expect_atomic_width_type(width, ty);
        let dst = self.new_value(ty);
        self.emit_effect(EffectOp::AtomicRmw {
            dst,
            addr,
            val,
            op,
            width,
        });
        dst
    }

    pub fn ecall(&mut self) {
        self.emit_effect(EffectOp::Ecall);
    }

    pub fn ebreak(&mut self) {
        self.emit_effect(EffectOp::Ebreak);
    }

    pub fn halt(&mut self, code: u64) {
        self.emit_effect(EffectOp::Halt { code });
    }
}
