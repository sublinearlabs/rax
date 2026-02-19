use std::collections::HashMap;

use crate::decode::{compressed::decode_compressed, Instruction};
use crate::trace::Tracer;
use crate::util::mask16;
use crate::HostIO;
use crate::{decode, VM};

pub struct Runner {
    io: HostIO,
    basic_blocks: HashMap<u64, Vec<(Instruction, u32, bool)>>,
    cycles: u64,
    elapsed: std::time::Duration,
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
        // cycles / microseconds = Mhz
        println!(
            "{:.2} Mhz",
            self.cycles as f64 / self.elapsed.as_micros() as f64
        )
    }

    pub fn step<T: Tracer>(&mut self, vm: &mut VM<T>) {
        if let Some(block) = self.basic_blocks.get(&vm.pc()) {
            for (i, (insn, insn_bytes, is_compressed)) in block.iter().enumerate() {
                let current_pc = vm.pc();
                let next_pc = current_pc.wrapping_add(if *is_compressed { 2 } else { 4 });

                // Begin tracing this instruction
                vm.tracer.begin_instruction(
                    self.cycles + i as u64,
                    current_pc,
                    &vm.registers,
                    &vm.f_reg,
                    *insn_bytes,
                    insn,
                );

                vm.set_pc(next_pc);

                // Execute the instruction (this will update PC)
                vm.execute_instruction(insn.clone(), *is_compressed, current_pc, &mut self.io);

                // Record next PC
                vm.tracer.record_next_pc(vm.pc());

                // Check for halt
                if vm.halted {
                    vm.tracer.record_halt();
                    vm.tracer.commit();
                    break;
                }

                // Commit the trace row
                vm.tracer.commit();
            }
            self.cycles = self.cycles.wrapping_add(block.len() as u64);
            return;
        }

        // Build the basic block
        let leader = vm.pc();
        let mut block = vec![];

        loop {
            let insn = vm.load_u16(vm.pc() as usize);
            let is_compressed = insn & mask16(2) != 0b11;

            let (insn, insn_bytes) = if is_compressed {
                (decode_compressed(insn), insn as u32)
            } else {
                let insn_upper = vm.load_u16((vm.pc() + 2) as usize);
                let insn = (insn_upper as u32) << 16 | insn as u32;
                (decode::decode(insn), insn)
            };

            if let Instruction::Illegal(_) = &insn {
                vm.halted = true;
                break;
            }

            // // Begin tracing this instruction
            vm.tracer.begin_instruction(
                self.cycles,
                vm.pc(),
                &vm.registers,
                &vm.f_reg,
                insn_bytes,
                &insn,
            );

            let current_pc = vm.pc();
            let next_pc = current_pc.wrapping_add(if is_compressed { 2 } else { 4 });
            vm.set_pc(next_pc);

            // Execute the instruction (this will update PC)
            vm.execute_instruction(insn.clone(), is_compressed, current_pc, &mut self.io);

            // Record next PC (set during execute_instruction or default to pc+4)
            vm.tracer.record_next_pc(vm.pc());

            self.cycles = self.cycles.wrapping_add(1);

            // Check for halt
            if vm.halted {
                vm.tracer.record_halt();
                vm.tracer.commit();
                break;
            }

            block.push((insn.clone(), insn_bytes, is_compressed));
            vm.tracer.commit();

            if insn.is_branch_or_jmp() {
                self.basic_blocks.insert(leader, block);
                break;
            }
        }
    }
}
