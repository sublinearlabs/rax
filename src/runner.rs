use std::collections::HashMap;

use crate::HostIO;
use crate::decode::Instruction;
#[cfg(feature = "ext_c")]
use crate::decode::compressed::decode_compressed;
use crate::ir::execute_ir;
use crate::ir::lower::lower_instruction_into;
use crate::ir::{IrBuilder, IrFunction};
use crate::trace::Tracer;
#[cfg(feature = "ext_c")]
use crate::util::mask16;
use crate::{VM, decode};

pub struct Runner {
    io: HostIO,
    basic_blocks: HashMap<u64, CachedBlock>,
    cycles: u64,
    elapsed: std::time::Duration,
}

struct DecodedBlock {
    insns: Vec<(Instruction, bool)>,
    terminator: (Instruction, bool),
}

struct CachedBlock {
    ir: IrFunction,
    insn_count: u64,
}

impl Runner {
    pub fn new() -> Self {
        Self {
            io: HostIO::new(),
            basic_blocks: HashMap::new(),
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

    pub fn run<T: Tracer>(&mut self, vm: &mut VM<T>) {
        let start = std::time::Instant::now();
        while !vm.halted {
            self.step(vm);
        }
        self.elapsed = start.elapsed();
    }

    pub fn run_with_timing<T: Tracer>(&mut self, vm: &mut VM<T>) {
        self.run(vm);
        println!("run took: {:?}ms", self.elapsed.as_micros());
        println!("run took: {:?}s", self.elapsed.as_secs_f64());

        println!("cycles: {}", self.cycles);
        println!(
            "{:.2} Mhz",
            self.cycles as f64 / self.elapsed.as_micros() as f64
        )
    }

    fn decode_basic_block<T: Tracer>(&self, vm: &mut VM<T>, start_pc: u64) -> DecodedBlock {
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

    fn lower_basic_block(&self, start_pc: u64, block: &DecodedBlock) -> CachedBlock {
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

        CachedBlock {
            ir: builder.finish(),
            insn_count,
        }
    }

    fn execute_basic_block<T: Tracer>(io: &mut HostIO, vm: &mut VM<T>, block: &IrFunction) {
        execute_ir(block, vm, io);
    }

    pub fn step<T: Tracer>(&mut self, vm: &mut VM<T>) {
        if let Some(block) = self.basic_blocks.get(&vm.pc()) {
            Self::execute_basic_block(&mut self.io, vm, &block.ir);
            self.cycles = self.cycles.wrapping_add(block.insn_count);
            return;
        }

        let leader = vm.pc();
        let block = self.decode_basic_block(vm, leader);
        let lowered = self.lower_basic_block(leader, &block);

        Self::execute_basic_block(&mut self.io, vm, &lowered.ir);
        self.cycles = self.cycles.wrapping_add(lowered.insn_count);

        self.basic_blocks.insert(leader, lowered);
    }
}
