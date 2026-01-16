// Register Opcodes

use crate::VM;
use crate::decode::{B, I, J, R, S, Sh, U};
use crate::trace::{MemOp, Tracer};
use crate::util::{mask, sext};

#[inline(always)]
pub(crate) fn execute_Add<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let result = vm.reg(insn.rs1).wrapping_add(vm.reg(insn.rs2));
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Sub<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let result = vm.reg(insn.rs1).wrapping_sub(vm.reg(insn.rs2));
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Xor<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let result = vm.reg(insn.rs1) ^ vm.reg(insn.rs2);
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Or<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let result = vm.reg(insn.rs1) | vm.reg(insn.rs2);
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_And<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let result = vm.reg(insn.rs1) & vm.reg(insn.rs2);
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Sll<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let result = vm.reg(insn.rs1) << (vm.reg(insn.rs2) & mask(6));
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Srl<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let result = vm.reg(insn.rs1) >> (vm.reg(insn.rs2) & mask(6));
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Sra<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let val = vm.reg(insn.rs1) as i64;
    let result = (val >> (vm.reg(insn.rs2) & mask(6))) as u64;
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Slt<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let result = if (vm.reg(insn.rs1) as i64) < (vm.reg(insn.rs2) as i64) {
        1
    } else {
        0
    };
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Sltu<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let result = if vm.reg(insn.rs1) < vm.reg(insn.rs2) {
        1
    } else {
        0
    };
    vm.reg_mut(insn.rd, result);
}

// Immediate Opcodes
#[inline(always)]
pub(crate) fn execute_Addi<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let result = vm.reg(insn.rs1).wrapping_add(insn.imm as u64);
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Xori<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let result = vm.reg(insn.rs1) ^ insn.imm as u64;
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Ori<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let result = vm.reg(insn.rs1) | insn.imm as u64;
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Andi<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let result = vm.reg(insn.rs1) & insn.imm as u64;
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Slli<T: Tracer>(vm: &mut VM<T>, insn: Sh) {
    let result = vm.reg(insn.rs1) << insn.shamt;
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Srli<T: Tracer>(vm: &mut VM<T>, insn: Sh) {
    let result = vm.reg(insn.rs1) >> insn.shamt;
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Srai<T: Tracer>(vm: &mut VM<T>, insn: Sh) {
    let shift = insn.shamt;
    let val = vm.reg(insn.rs1) as i64;
    let result = (val >> shift) as u64;
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Slti<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let result = if (vm.reg(insn.rs1) as i64) < (insn.imm as i64) {
        1
    } else {
        0
    };
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Sltiu<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let result = if vm.reg(insn.rs1) < insn.imm as u64 {
        1
    } else {
        0
    };
    vm.reg_mut(insn.rd, result);
}

// Load Opcodes
#[inline(always)]
pub(crate) fn execute_Lb<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let addr = vm.reg(insn.rs1).wrapping_add(insn.imm as u64);
    let raw_value = vm.load_u8(addr as usize) as u64;
    let result = sext(raw_value, 8);
    vm.tracer.record_mem_op(MemOp::LoadByte {
        addr,
        value: raw_value as u8,
        signed: true,
    });
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Lbu<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let addr = vm.reg(insn.rs1).wrapping_add(insn.imm as u64);
    let result = vm.load_u8(addr as usize) as u64;
    vm.tracer.record_mem_op(MemOp::LoadByte {
        addr,
        value: result as u8,
        signed: false,
    });
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Lh<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let addr = vm.reg(insn.rs1).wrapping_add(insn.imm as u64);
    let raw_value = vm.load_u16(addr as usize) as u64;
    let result = sext(raw_value, 16);
    vm.tracer.record_mem_op(MemOp::LoadHalf {
        addr,
        value: raw_value as u16,
        signed: true,
    });
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Lhu<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let addr = vm.reg(insn.rs1).wrapping_add(insn.imm as u64);
    let result = vm.load_u16(addr as usize) as u64;
    vm.tracer.record_mem_op(MemOp::LoadHalf {
        addr,
        value: result as u16,
        signed: false,
    });
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Lw<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let addr = vm.reg(insn.rs1).wrapping_add(insn.imm as u64);
    let raw_value = vm.load_u32(addr as usize) as u64;
    let result = sext(raw_value, 32);
    vm.tracer.record_mem_op(MemOp::LoadWord {
        addr,
        value: raw_value as u32,
        signed: true,
    });
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Lwu<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let addr = vm.reg(insn.rs1).wrapping_add(insn.imm as u64);
    let result = vm.load_u32(addr as usize) as u64;
    vm.tracer.record_mem_op(MemOp::LoadWord {
        addr,
        value: result as u32,
        signed: false,
    });
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Ld<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let addr = vm.reg(insn.rs1).wrapping_add(insn.imm as u64);
    let result = vm.load_u64(addr as usize);
    vm.tracer.record_mem_op(MemOp::LoadDouble {
        addr,
        value: result,
    });
    vm.reg_mut(insn.rd, result);
}

// Store Opcodes
#[inline(always)]
pub(crate) fn execute_Sb<T: Tracer>(vm: &mut VM<T>, insn: S) {
    let addr = vm.reg(insn.rs1).wrapping_add(insn.imm as u64);
    let value = vm.reg(insn.rs2);
    vm.store_u8(addr as usize, value as u8);
    vm.tracer.record_mem_op(MemOp::StoreByte {
        addr,
        value: value as u8,
    });
}

#[inline(always)]
pub(crate) fn execute_Sh<T: Tracer>(vm: &mut VM<T>, insn: S) {
    let addr = vm.reg(insn.rs1).wrapping_add(insn.imm as u64);
    let value = vm.reg(insn.rs2);
    vm.store_u16(addr as usize, value as u16);
    vm.tracer.record_mem_op(MemOp::StoreHalf {
        addr,
        value: value as u16,
    });
}

#[inline(always)]
pub(crate) fn execute_Sw<T: Tracer>(vm: &mut VM<T>, insn: S) {
    let addr = vm.reg(insn.rs1).wrapping_add(insn.imm as u64);
    let value = vm.reg(insn.rs2);
    vm.store_u32(addr as usize, value as u32);
    vm.tracer.record_mem_op(MemOp::StoreWord {
        addr,
        value: value as u32,
    });
}

#[inline(always)]
pub(crate) fn execute_Sd<T: Tracer>(vm: &mut VM<T>, insn: S) {
    let addr = vm.reg(insn.rs1).wrapping_add(insn.imm as u64);
    let value = vm.reg(insn.rs2);
    vm.store_u64(addr as usize, value);
    vm.tracer.record_mem_op(MemOp::StoreDouble { addr, value });
}

// Branch Opcodes
#[inline(always)]
pub(crate) fn execute_Beq<T: Tracer>(vm: &mut VM<T>, insn: B) -> bool {
    if vm.reg(insn.rs1) == vm.reg(insn.rs2) {
        vm.pc = vm.pc.wrapping_add(insn.imm as u64);
        return true;
    }
    false
}

#[inline(always)]
pub(crate) fn execute_Bne<T: Tracer>(vm: &mut VM<T>, insn: B) -> bool {
    if vm.reg(insn.rs1) != vm.reg(insn.rs2) {
        vm.pc = vm.pc.wrapping_add(insn.imm as u64);
        return true;
    }
    false
}

#[inline(always)]
pub(crate) fn execute_Blt<T: Tracer>(vm: &mut VM<T>, insn: B) -> bool {
    if (vm.reg(insn.rs1) as i64) < (vm.reg(insn.rs2) as i64) {
        vm.pc = vm.pc.wrapping_add(insn.imm as u64);
        return true;
    }
    false
}

#[inline(always)]
pub(crate) fn execute_Bltu<T: Tracer>(vm: &mut VM<T>, insn: B) -> bool {
    if vm.reg(insn.rs1) < vm.reg(insn.rs2) {
        vm.pc = vm.pc.wrapping_add(insn.imm as u64);
        return true;
    }
    false
}

#[inline(always)]
pub(crate) fn execute_Bge<T: Tracer>(vm: &mut VM<T>, insn: B) -> bool {
    if (vm.reg(insn.rs1) as i64) >= (vm.reg(insn.rs2) as i64) {
        vm.pc = vm.pc.wrapping_add(insn.imm as u64);
        return true;
    }
    false
}

#[inline(always)]
pub(crate) fn execute_Bgeu<T: Tracer>(vm: &mut VM<T>, insn: B) -> bool {
    if vm.reg(insn.rs1) >= vm.reg(insn.rs2) {
        vm.pc = vm.pc.wrapping_add(insn.imm as u64);
        return true;
    };
    false
}

// Jump opcodes
#[inline(always)]
pub(crate) fn execute_Jal<T: Tracer>(vm: &mut VM<T>, insn: J) {
    let result = vm.pc.wrapping_add(4);
    vm.reg_mut(insn.rd, result);
    vm.pc = vm.pc.wrapping_add(insn.imm as u64);
    return;
}

#[inline(always)]
pub(crate) fn execute_Jalr<T: Tracer>(vm: &mut VM<T>, insn: I, is_compressed: bool) {
    let target = vm.reg(insn.rs1).wrapping_add(insn.imm as u64);
    let result = vm.pc.wrapping_add(if is_compressed { 2 } else { 4 });
    vm.reg_mut(insn.rd, result);
    vm.pc = target;
    return;
}

// Lui and Auipc
#[inline(always)]
pub(crate) fn execute_Lui<T: Tracer>(vm: &mut VM<T>, insn: U) {
    vm.reg_mut(insn.rd, insn.imm as u64);
}

#[inline(always)]
pub(crate) fn execute_Auipc<T: Tracer>(vm: &mut VM<T>, insn: U) {
    let result = vm.pc.wrapping_add(insn.imm as u64);
    vm.reg_mut(insn.rd, result);
}

// RV64I Rs
#[inline(always)]
pub(crate) fn execute_Addiw<T: Tracer>(vm: &mut VM<T>, insn: I) {
    let res = vm.reg(insn.rs1).wrapping_add(insn.imm as u64) & mask(32);
    let result = sext(res, 32);
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Slliw<T: Tracer>(vm: &mut VM<T>, insn: Sh) {
    let val = vm.reg(insn.rs1) << insn.shamt;
    let result = sext(val & mask(32), 32);
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Srliw<T: Tracer>(vm: &mut VM<T>, insn: Sh) {
    let result = sext((vm.reg(insn.rs1) & mask(32)) >> insn.shamt, 32);
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Sraiw<T: Tracer>(vm: &mut VM<T>, insn: Sh) {
    let shift = insn.shamt;
    let a = (vm.reg(insn.rs1) & mask(32)) as i32;
    let result = (a >> shift) as i64 as u64;
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Addw<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let result = sext(
        vm.reg(insn.rs1).wrapping_add(vm.reg(insn.rs2)) & mask(32),
        32,
    );
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Subw<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let a = vm.reg(insn.rs1) as i32;
    let b = vm.reg(insn.rs2) as i32;
    let result = a.wrapping_sub(b) as i64 as u64;
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Sllw<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let a = vm.reg(insn.rs1);
    let shift = vm.reg(insn.rs2) & mask(5);
    let result = sext((a << shift) & mask(32), 32);
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Srlw<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let a = vm.reg(insn.rs1) & mask(32);
    let shift = vm.reg(insn.rs2) & mask(5);
    let result = sext(a >> shift, 32);
    vm.reg_mut(insn.rd, result);
}

#[inline(always)]
pub(crate) fn execute_Sraw<T: Tracer>(vm: &mut VM<T>, insn: R) {
    let a = (vm.reg(insn.rs1) & mask(32)) as i32;
    let shift = vm.reg(insn.rs2) & mask(5);
    let result = (a >> shift) as i64 as u64;
    vm.reg_mut(insn.rd, result);
}
