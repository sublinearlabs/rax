use crate::ir::{
    Block, BlockId, EffectOp, IrFunction, IrType, Op, PureOp, Reg, Terminator, ValueId,
};
use crate::util::mask;

pub struct IrBuilder {
    func: IrFunction,
    current_block: Option<BlockId>,
}

impl IrBuilder {
    pub fn new() -> Self {
        Self {
            func: IrFunction::new(),
            current_block: None,
        }
    }

    pub fn finish(self) -> IrFunction {
        self.func
    }

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
        id
    }

    pub fn block(&mut self) -> BlockId {
        self.block_with_args(&[])
    }

    pub fn block_arg(&self, block: BlockId, index: usize) -> ValueId {
        self.func.blocks[block.0 as usize].args[index]
    }

    pub fn switch_to(&mut self, block: BlockId) {
        self.current_block = Some(block);
    }

    pub fn const_i64(&mut self, v: i64) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Pure {
            dst,
            op: PureOp::ConstI64(v),
        });
        dst
    }

    pub fn add(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Add, a, b)
    }

    pub fn sub(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Sub, a, b)
    }

    pub fn mul(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Mul, a, b)
    }

    pub fn mulh(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Mulh, a, b)
    }

    pub fn mulhu(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Mulhu, a, b)
    }

    pub fn mulhsu(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Mulhsu, a, b)
    }

    pub fn div(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Div, a, b)
    }

    pub fn divu(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Divu, a, b)
    }

    pub fn rem(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Rem, a, b)
    }

    pub fn remu(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Remu, a, b)
    }

    pub fn and(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::And, a, b)
    }

    pub fn or(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Or, a, b)
    }

    pub fn xor(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Xor, a, b)
    }

    pub fn not(&mut self, value: ValueId) -> ValueId {
        let all_ones = self.const_i64(-1);
        self.xor(value, all_ones)
    }

    pub fn shl(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Shl, a, b)
    }

    pub fn shr(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Shr, a, b)
    }

    pub fn sar(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_i64(PureOp::Sar, a, b)
    }

    pub fn eq(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_cmp(PureOp::Eq, a, b)
    }

    pub fn ne(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_cmp(PureOp::Ne, a, b)
    }

    pub fn lt(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_cmp(PureOp::Lt, a, b)
    }

    pub fn ltu(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_cmp(PureOp::Ltu, a, b)
    }

    pub fn ge(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_cmp(PureOp::Ge, a, b)
    }

    pub fn geu(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_cmp(PureOp::Geu, a, b)
    }

    pub fn sext(&mut self, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.un_op_typed(PureOp::Sext { v, from, to }, v, from, to)
    }

    pub fn zext(&mut self, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.un_op_typed(PureOp::Zext { v, from, to }, v, from, to)
    }

    pub fn trunc(&mut self, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.un_op_typed(PureOp::Trunc { v, from, to }, v, from, to)
    }

    pub fn select(&mut self, cond: ValueId, t: ValueId, f: ValueId) -> ValueId {
        self.expect_type(cond, IrType::I1);
        let ty = self.value_type(t);
        self.expect_type(f, ty);
        let dst = self.new_value(ty);
        self.push_op(Op::Pure {
            dst,
            op: PureOp::Select { cond, t, f },
        });
        dst
    }

    pub fn get_reg(&mut self, reg: Reg) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::GetReg { dst, reg }));
        dst
    }

    pub fn set_reg(&mut self, reg: Reg, val: ValueId) {
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::SetReg { reg, val }));
    }

    pub fn reg(&mut self, idx: u8) -> ValueId {
        self.get_reg(reg_from_u8(idx))
    }

    pub fn set_reg_idx(&mut self, idx: u8, val: ValueId) {
        self.set_reg(reg_from_u8(idx), val);
    }

    pub fn set_reg_if_needed(&mut self, idx: u8, val: ValueId) {
        if idx == 0 {
            return;
        }
        self.set_reg(reg_from_u8(idx), val);
    }

    pub fn get_freg(&mut self, reg: Reg) -> ValueId {
        let dst = self.new_value(IrType::F32);
        self.push_op(Op::Effect(EffectOp::GetReg { dst, reg }));
        dst
    }

    pub fn set_freg(&mut self, reg: Reg, val: ValueId) {
        self.expect_type(val, IrType::F32);
        self.push_op(Op::Effect(EffectOp::SetReg { reg, val }));
    }

    pub fn freg(&mut self, idx: u8) -> ValueId {
        self.get_freg(freg_from_u8(idx))
    }

    pub fn set_freg_idx(&mut self, idx: u8, val: ValueId) {
        self.set_freg(freg_from_u8(idx), val);
    }

    pub fn imm_i32(&mut self, value: i32) -> ValueId {
        self.const_i64(value as i64)
    }

    pub fn imm_u8(&mut self, value: u8) -> ValueId {
        self.const_i64(value as i64)
    }

    pub fn imm_u64(&mut self, value: u64) -> ValueId {
        self.const_i64(value as i64)
    }

    pub fn iconst(&mut self, value: i64) -> ValueId {
        self.const_i64(value)
    }

    pub fn zimm5(&mut self, value: u8) -> ValueId {
        self.const_i64((value & 0x1f) as i64)
    }

    pub fn shamt64(&mut self, value: ValueId) -> ValueId {
        let mask = self.imm_u64(mask(6));
        self.and(value, mask)
    }

    pub fn shamt32(&mut self, value: ValueId) -> ValueId {
        let mask = self.imm_u64(mask(5));
        self.and(value, mask)
    }

    pub fn addr(&mut self, rs1: u8, offset: i32) -> ValueId {
        let base = self.reg(rs1);
        let off = self.imm_i32(offset);
        self.add(base, off)
    }

    pub fn pc_plus(&mut self, current_pc: u64, offset: i32) -> ValueId {
        let base = self.imm_u64(current_pc);
        let off = self.imm_i32(offset);
        self.add(base, off)
    }

    pub fn get_csr(&mut self, csr: u32) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::GetCsr { dst, csr }));
        dst
    }

    pub fn set_csr(&mut self, csr: u32, val: ValueId) {
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::SetCsr { csr, val }));
    }

    pub fn get_pc(&mut self) -> ValueId {
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::GetPc { dst }));
        dst
    }

    pub fn set_pc(&mut self, val: ValueId) {
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::SetPc { val }));
    }

    pub fn load8s(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load8s { dst, addr }));
        dst
    }

    pub fn load8u(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load8u { dst, addr }));
        dst
    }

    pub fn load16s(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load16s { dst, addr }));
        dst
    }

    pub fn load16u(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load16u { dst, addr }));
        dst
    }

    pub fn load32s(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load32s { dst, addr }));
        dst
    }

    pub fn load32u(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load32u { dst, addr }));
        dst
    }

    pub fn load64(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Effect(EffectOp::Load64 { dst, addr }));
        dst
    }

    pub fn store8(&mut self, addr: ValueId, val: ValueId) {
        self.expect_type(addr, IrType::I64);
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::Store8 { addr, val }));
    }

    pub fn store16(&mut self, addr: ValueId, val: ValueId) {
        self.expect_type(addr, IrType::I64);
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::Store16 { addr, val }));
    }

    pub fn store32(&mut self, addr: ValueId, val: ValueId) {
        self.expect_type(addr, IrType::I64);
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::Store32 { addr, val }));
    }

    pub fn store64(&mut self, addr: ValueId, val: ValueId) {
        self.expect_type(addr, IrType::I64);
        self.expect_type(val, IrType::I64);
        self.push_op(Op::Effect(EffectOp::Store64 { addr, val }));
    }

    pub fn load_f32(&mut self, addr: ValueId) -> ValueId {
        self.expect_type(addr, IrType::I64);
        let dst = self.new_value(IrType::F32);
        self.push_op(Op::Effect(EffectOp::LoadF32 { dst, addr }));
        dst
    }

    pub fn store_f32(&mut self, addr: ValueId, val: ValueId) {
        self.expect_type(addr, IrType::I64);
        self.expect_type(val, IrType::F32);
        self.push_op(Op::Effect(EffectOp::StoreF32 { addr, val }));
    }

    pub fn fadd(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_f32(PureOp::Fadd, a, b)
    }

    pub fn fsub(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_f32(PureOp::Fsub, a, b)
    }

    pub fn fmul(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_f32(PureOp::Fmul, a, b)
    }

    pub fn fdiv(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_f32(PureOp::Fdiv, a, b)
    }

    pub fn fsqrt(&mut self, v: ValueId) -> ValueId {
        self.un_f32(PureOp::Fsqrt, v)
    }

    pub fn fmin(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_f32(PureOp::Fmin, a, b)
    }

    pub fn fmax(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_f32(PureOp::Fmax, a, b)
    }

    pub fn feq(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_f32_cmp(PureOp::Feq, a, b)
    }

    pub fn flt(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_f32_cmp(PureOp::Flt, a, b)
    }

    pub fn fle(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_f32_cmp(PureOp::Fle, a, b)
    }

    pub fn fsgnj(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_f32(PureOp::Fsgnj, a, b)
    }

    pub fn fsgnjn(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_f32(PureOp::Fsgnjn, a, b)
    }

    pub fn fsgnjx(&mut self, a: ValueId, b: ValueId) -> ValueId {
        self.bin_f32(PureOp::Fsgnjx, a, b)
    }

    pub fn fcvt_f32_i32(&mut self, v: ValueId) -> ValueId {
        self.un_f32_from_i32(PureOp::FcvtF32I32, v)
    }

    pub fn fcvt_f32_i64(&mut self, v: ValueId) -> ValueId {
        self.un_f32_from_i64(PureOp::FcvtF32I64, v)
    }

    pub fn fcvt_f32_u32(&mut self, v: ValueId) -> ValueId {
        self.un_f32_from_i32(PureOp::FcvtF32U32, v)
    }

    pub fn fcvt_f32_u64(&mut self, v: ValueId) -> ValueId {
        self.un_f32_from_i64(PureOp::FcvtF32U64, v)
    }

    pub fn fcvt_i32_f32(&mut self, v: ValueId) -> ValueId {
        self.un_i32_from_f32(PureOp::FcvtI32F32, v)
    }

    pub fn fcvt_i64_f32(&mut self, v: ValueId) -> ValueId {
        self.un_i64_from_f32(PureOp::FcvtI64F32, v)
    }

    pub fn fcvt_u32_f32(&mut self, v: ValueId) -> ValueId {
        self.un_i32_from_f32(PureOp::FcvtU32F32, v)
    }

    pub fn fcvt_u64_f32(&mut self, v: ValueId) -> ValueId {
        self.un_i64_from_f32(PureOp::FcvtU64F32, v)
    }

    pub fn fmv_f32(&mut self, v: ValueId) -> ValueId {
        self.un_f32_from_i32(PureOp::FmvF32, v)
    }

    pub fn fmv_i32(&mut self, v: ValueId) -> ValueId {
        self.un_i32_from_f32(PureOp::FmvI32, v)
    }

    pub fn fcvt_w_s(&mut self, v: ValueId) -> ValueId {
        self.un_i32_from_f32(PureOp::FcvtI32F32, v)
    }

    pub fn fcvt_wu_s(&mut self, v: ValueId) -> ValueId {
        self.un_i32_from_f32(PureOp::FcvtU32F32, v)
    }

    pub fn fcvt_s_w(&mut self, v: ValueId) -> ValueId {
        self.un_f32_from_i32(PureOp::FcvtF32I32, v)
    }

    pub fn fcvt_s_wu(&mut self, v: ValueId) -> ValueId {
        self.un_f32_from_i32(PureOp::FcvtF32U32, v)
    }

    pub fn fmadd(&mut self, a: ValueId, b: ValueId, c: ValueId) -> ValueId {
        // For now, implement as (a * b) + c
        let mul = self.fmul(a, b);
        self.fadd(mul, c)
    }

    pub fn fmsub(&mut self, a: ValueId, b: ValueId, c: ValueId) -> ValueId {
        // For now, implement as (a * b) - c
        let mul = self.fmul(a, b);
        self.fsub(mul, c)
    }

    pub fn fnmsub(&mut self, a: ValueId, b: ValueId, c: ValueId) -> ValueId {
        // For now, implement as -(a * b) + c
        let mul = self.fmul(a, b);
        let neg_mul = self.fneg(mul);
        self.fadd(neg_mul, c)
    }

    pub fn fnmadd(&mut self, a: ValueId, b: ValueId, c: ValueId) -> ValueId {
        // For now, implement as -(a * b) - c
        let mul = self.fmul(a, b);
        let neg_mul = self.fneg(mul);
        self.fsub(neg_mul, c)
    }

    pub fn fneg(&mut self, v: ValueId) -> ValueId {
        // Implement as -0.0 - v
        let neg_zero = self.fconst(0.0);
        let neg_zero = self.fneg_const(neg_zero);
        self.fsub(neg_zero, v)
    }

    pub fn fconst(&mut self, v: f32) -> ValueId {
        let dst = self.new_value(IrType::F32);
        self.push_op(Op::Pure {
            dst,
            op: PureOp::ConstF32(v.to_bits()),
        });
        dst
    }

    pub fn fneg_const(&mut self, v: ValueId) -> ValueId {
        // Flip the sign bit
        let bits = self.fbits(v);
        let sign_bit = self.iconst(0x8000_0000);
        let flipped = self.xor(bits, sign_bit);
        self.fbits_to_f32(flipped)
    }

    pub fn fbits(&mut self, v: ValueId) -> ValueId {
        self.un_i32_from_f32(PureOp::FmvI32, v)
    }

    pub fn fbits_to_f32(&mut self, v: ValueId) -> ValueId {
        self.un_f32_from_i32(PureOp::FmvF32, v)
    }

    pub fn ecall(&mut self) {
        self.push_op(Op::Effect(EffectOp::Ecall));
    }

    pub fn ebreak(&mut self) {
        self.push_op(Op::Effect(EffectOp::Ebreak));
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

    pub fn value_type(&self, v: ValueId) -> IrType {
        self.func.value_type(v)
    }

    fn new_value(&mut self, ty: IrType) -> ValueId {
        let id = ValueId(self.func.value_types.len() as u32);
        self.func.value_types.push(ty);
        id
    }

    fn push_op(&mut self, op: Op) {
        let block = self.current_block.expect("no current block");
        let block = &mut self.func.blocks[block.0 as usize];
        if block.term.is_some() {
            panic!("cannot add op after terminator");
        }
        block.ops.push(op);
    }

    fn set_term(&mut self, term: Terminator) {
        let block = self.current_block.expect("no current block");
        let block = &mut self.func.blocks[block.0 as usize];
        if block.term.is_some() {
            panic!("terminator already set");
        }
        block.term = Some(term);
    }

    fn expect_type(&self, v: ValueId, ty: IrType) {
        let actual = self.value_type(v);
        if actual != ty {
            panic!("type mismatch: expected {:?}, got {:?}", ty, actual);
        }
    }

    fn bin_i64(
        &mut self,
        make_op: fn(ValueId, ValueId) -> PureOp,
        a: ValueId,
        b: ValueId,
    ) -> ValueId {
        self.expect_type(a, IrType::I64);
        self.expect_type(b, IrType::I64);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Pure {
            dst,
            op: make_op(a, b),
        });
        dst
    }

    fn bin_cmp(
        &mut self,
        make_op: fn(ValueId, ValueId) -> PureOp,
        a: ValueId,
        b: ValueId,
    ) -> ValueId {
        self.expect_type(a, IrType::I64);
        self.expect_type(b, IrType::I64);
        let dst = self.new_value(IrType::I1);
        self.push_op(Op::Pure {
            dst,
            op: make_op(a, b),
        });
        dst
    }

    fn un_op_typed(&mut self, op: PureOp, v: ValueId, from: IrType, to: IrType) -> ValueId {
        self.expect_type(v, from);
        let dst = self.new_value(to);
        self.push_op(Op::Pure { dst, op });
        dst
    }

    fn bin_f32(
        &mut self,
        make_op: fn(ValueId, ValueId) -> PureOp,
        a: ValueId,
        b: ValueId,
    ) -> ValueId {
        self.expect_type(a, IrType::F32);
        self.expect_type(b, IrType::F32);
        let dst = self.new_value(IrType::F32);
        self.push_op(Op::Pure {
            dst,
            op: make_op(a, b),
        });
        dst
    }

    fn bin_f32_cmp(
        &mut self,
        make_op: fn(ValueId, ValueId) -> PureOp,
        a: ValueId,
        b: ValueId,
    ) -> ValueId {
        self.expect_type(a, IrType::F32);
        self.expect_type(b, IrType::F32);
        let dst = self.new_value(IrType::I1);
        self.push_op(Op::Pure {
            dst,
            op: make_op(a, b),
        });
        dst
    }

    fn un_f32(&mut self, make_op: fn(ValueId) -> PureOp, v: ValueId) -> ValueId {
        self.expect_type(v, IrType::F32);
        let dst = self.new_value(IrType::F32);
        self.push_op(Op::Pure {
            dst,
            op: make_op(v),
        });
        dst
    }

    fn un_f32_from_i32(&mut self, make_op: fn(ValueId) -> PureOp, v: ValueId) -> ValueId {
        self.expect_type(v, IrType::I32);
        let dst = self.new_value(IrType::F32);
        self.push_op(Op::Pure {
            dst,
            op: make_op(v),
        });
        dst
    }

    fn un_f32_from_i64(&mut self, make_op: fn(ValueId) -> PureOp, v: ValueId) -> ValueId {
        self.expect_type(v, IrType::I64);
        let dst = self.new_value(IrType::F32);
        self.push_op(Op::Pure {
            dst,
            op: make_op(v),
        });
        dst
    }

    fn un_i32_from_f32(&mut self, make_op: fn(ValueId) -> PureOp, v: ValueId) -> ValueId {
        self.expect_type(v, IrType::F32);
        let dst = self.new_value(IrType::I32);
        self.push_op(Op::Pure {
            dst,
            op: make_op(v),
        });
        dst
    }

    fn un_i64_from_f32(&mut self, make_op: fn(ValueId) -> PureOp, v: ValueId) -> ValueId {
        self.expect_type(v, IrType::F32);
        let dst = self.new_value(IrType::I64);
        self.push_op(Op::Pure {
            dst,
            op: make_op(v),
        });
        dst
    }

    fn check_block_args(&self, block: BlockId, args: &[ValueId]) {
        let block = &self.func.blocks[block.0 as usize];
        if block.args.len() != args.len() {
            panic!("block arg count mismatch");
        }
        for (arg, param) in args.iter().zip(block.args.iter()) {
            let expected = self.value_type(*param);
            let actual = self.value_type(*arg);
            if expected != actual {
                panic!(
                    "block arg type mismatch: expected {:?}, got {:?}",
                    expected, actual
                );
            }
        }
    }
}

fn reg_from_u8(idx: u8) -> Reg {
    match idx {
        0 => Reg::X0,
        1 => Reg::X1,
        2 => Reg::X2,
        3 => Reg::X3,
        4 => Reg::X4,
        5 => Reg::X5,
        6 => Reg::X6,
        7 => Reg::X7,
        8 => Reg::X8,
        9 => Reg::X9,
        10 => Reg::X10,
        11 => Reg::X11,
        12 => Reg::X12,
        13 => Reg::X13,
        14 => Reg::X14,
        15 => Reg::X15,
        16 => Reg::X16,
        17 => Reg::X17,
        18 => Reg::X18,
        19 => Reg::X19,
        20 => Reg::X20,
        21 => Reg::X21,
        22 => Reg::X22,
        23 => Reg::X23,
        24 => Reg::X24,
        25 => Reg::X25,
        26 => Reg::X26,
        27 => Reg::X27,
        28 => Reg::X28,
        29 => Reg::X29,
        30 => Reg::X30,
        31 => Reg::X31,
        _ => panic!("invalid register index: {}", idx),
    }
}

fn freg_from_u8(idx: u8) -> Reg {
    match idx {
        0 => Reg::F0,
        1 => Reg::F1,
        2 => Reg::F2,
        3 => Reg::F3,
        4 => Reg::F4,
        5 => Reg::F5,
        6 => Reg::F6,
        7 => Reg::F7,
        8 => Reg::F8,
        9 => Reg::F9,
        10 => Reg::F10,
        11 => Reg::F11,
        12 => Reg::F12,
        13 => Reg::F13,
        14 => Reg::F14,
        15 => Reg::F15,
        16 => Reg::F16,
        17 => Reg::F17,
        18 => Reg::F18,
        19 => Reg::F19,
        20 => Reg::F20,
        21 => Reg::F21,
        22 => Reg::F22,
        23 => Reg::F23,
        24 => Reg::F24,
        25 => Reg::F25,
        26 => Reg::F26,
        27 => Reg::F27,
        28 => Reg::F28,
        29 => Reg::F29,
        30 => Reg::F30,
        31 => Reg::F31,
        _ => panic!("invalid floating point register index: {}", idx),
    }
}
