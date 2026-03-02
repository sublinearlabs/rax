use std::collections::HashMap;

use crate::HostIO;
use crate::decode::Instruction;
#[cfg(feature = "ext_c")]
use crate::decode::compressed::decode_compressed;
use crate::ir::lower::lower_instruction_into;
use crate::ir::{IrBuilder, IrFunction};
use crate::jit::compile::{JitFn, compile_ir_function};
use crate::jit::jit_module::{HelperFuncIds, build_jit_module, declare_helpers};
use crate::trace::NoopTracer;
#[cfg(feature = "ext_c")]
use crate::util::mask16;
use crate::{VM, decode};
use cranelift_module::Module;

pub struct Runner {
    io: HostIO,
    jit_module: cranelift_jit::JITModule,
    helper_ids: HelperFuncIds,
    jit_cache: HashMap<u64, JitBlock>,
    jit_counter: u64,
    cycles: u64,
    elapsed: std::time::Duration,
}

struct DecodedBlock {
    insns: Vec<(Instruction, bool)>,
    terminator: (Instruction, bool),
}

struct JitBlock {
    func: JitFn,
    insn_count: u64,
}

struct LoweredBlock {
    ir: IrFunction,
    insn_count: u64,
}

impl Runner {
    pub fn new() -> Self {
        let mut jit_module = build_jit_module();
        let ptr_ty = jit_module.isa().pointer_type();
        let helper_ids = declare_helpers(&mut jit_module, ptr_ty);
        Self {
            io: HostIO::new(),
            jit_module,
            helper_ids,
            jit_cache: HashMap::new(),
            jit_counter: 0,
            cycles: 0,
            elapsed: std::time::Duration::default(),
        }
    }

    pub fn set_input_stream(&mut self, input: Vec<u8>) {
        self.io.set_input_stream(input);
    }

    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.elapsed
    }

    pub fn run(&mut self, vm: &mut VM) {
        let start = std::time::Instant::now();
        while !vm.halted {
            self.step(vm);
        }
        self.elapsed = start.elapsed();
    }

    pub fn run_with_timing(&mut self, vm: &mut VM) {
        self.run(vm);
        println!("run took: {:?}ms", self.elapsed.as_micros());
        println!("run took: {:?}s", self.elapsed.as_secs_f64());

        println!("cycles: {}", self.cycles);
        println!(
            "{:.2} Mhz",
            self.cycles as f64 / self.elapsed.as_micros() as f64
        )
    }

    fn decode_basic_block(&self, vm: &mut VM, start_pc: u64) -> DecodedBlock {
        let mut block = vec![];
        let mut pc = start_pc;

        loop {
            let (insn, is_compressed) = {
                #[cfg(feature = "ext_c")]
                {
                    let insn = vm.load_u16(pc as usize);
                    let is_compressed = insn & mask16(2) != 0b11;
                    if is_compressed {
                        (decode_compressed(insn), true)
                    } else {
                        let insn = vm.load_u32(pc as usize);
                        let insn_upper = vm.load_u16((pc + 2) as usize);
                        let insn = (insn_upper as u32) << 16 | insn as u32;
                        (decode::decode(insn), false)
                    }
                }
                #[cfg(not(feature = "ext_c"))]
                {
                    let insn = vm.load_u32(pc as usize);
                    (decode::decode(insn), false)
                }
            };

            let is_branch = insn.is_branch_or_jmp();
            let is_illegal = matches!(insn, Instruction::Illegal(_));
            let is_halt = matches!(insn, Instruction::Ebreak);

            if is_branch || is_illegal || is_halt {
                return DecodedBlock {
                    insns: block,
                    terminator: (insn, is_compressed),
                };
            }

            block.push((insn, is_compressed));

            pc = pc.wrapping_add(if is_compressed { 2 } else { 4 });
        }
    }

    fn lower_basic_block(&self, start_pc: u64, block: &DecodedBlock) -> LoweredBlock {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);

        let mut pc = start_pc;
        let mut insn_count = 0u64;

        for (insn, is_compressed) in &block.insns {
            let current_pc = pc;
            let next_pc = current_pc.wrapping_add(if *is_compressed { 2 } else { 4 });

            lower_instruction_into(insn, current_pc, next_pc, &mut builder);
            insn_count = insn_count.wrapping_add(1);

            let next_pc_val = builder.const_i64(next_pc as i64);
            builder.set_pc(next_pc_val);
            builder.require_single_exit();
            pc = next_pc;
        }

        let (insn, is_compressed) = &block.terminator;
        let current_pc = pc;
        let next_pc = current_pc.wrapping_add(if *is_compressed { 2 } else { 4 });

        lower_instruction_into(insn, current_pc, next_pc, &mut builder);
        insn_count = insn_count.wrapping_add(1);

        LoweredBlock {
            ir: builder.finish(),
            insn_count,
        }
    }

    pub fn step(&mut self, vm: &mut VM) {
        if let Some(block) = self.jit_cache.get(&vm.pc()) {
            let vm_ptr = vm as *mut VM;
            let io_ptr = &mut self.io as *mut HostIO;
            unsafe {
                (block.func)(vm_ptr, io_ptr);
            }
            self.cycles = self.cycles.wrapping_add(block.insn_count);
            return;
        }

        let leader = vm.pc();
        let block = self.decode_basic_block(vm, leader);
        let lowered = self.lower_basic_block(leader, &block);

        let name = format!("ir_entry_{:x}", self.jit_counter);
        self.jit_counter = self.jit_counter.wrapping_add(1);
        let jit_fn =
            compile_ir_function(&mut self.jit_module, &self.helper_ids, &lowered.ir, &name);
        let vm_ptr = vm as *mut VM;
        let io_ptr = &mut self.io as *mut HostIO;
        unsafe {
            jit_fn(vm_ptr, io_ptr);
        }
        self.jit_cache.insert(
            leader,
            JitBlock {
                func: jit_fn,
                insn_count: lowered.insn_count,
            },
        );

        self.cycles = self.cycles.wrapping_add(lowered.insn_count);
    }
}
