use std::collections::HashMap;

#[cfg(feature = "ext_c")]
use crate::decode::compressed::decode_compressed;
use crate::decode::Instruction;
use crate::ir::execute_ir;
#[cfg(feature = "ext_a")]
use crate::ir::lower::a::lower_a;
use crate::ir::lower::i::lower_i;
#[cfg(feature = "ext_m")]
use crate::ir::lower::m::lower_m;
use crate::ir::IrFunction;
use crate::trace::Tracer;
#[cfg(feature = "ext_c")]
use crate::util::mask16;
use crate::HostIO;
use crate::{decode, VM};

fn lower_instruction(insn: &Instruction, current_pc: u64, next_pc: u64) -> IrFunction {
    match insn {
        // I instructions
        Instruction::Add(_)
        | Instruction::Sub(_)
        | Instruction::Sll(_)
        | Instruction::Slt(_)
        | Instruction::Sltu(_)
        | Instruction::Xor(_)
        | Instruction::Srl(_)
        | Instruction::Sra(_)
        | Instruction::Or(_)
        | Instruction::And(_)
        | Instruction::Addi(_)
        | Instruction::Slti(_)
        | Instruction::Sltiu(_)
        | Instruction::Xori(_)
        | Instruction::Ori(_)
        | Instruction::Andi(_)
        | Instruction::Slli(_)
        | Instruction::Srli(_)
        | Instruction::Srai(_)
        | Instruction::Lb(_)
        | Instruction::Lh(_)
        | Instruction::Lw(_)
        | Instruction::Lbu(_)
        | Instruction::Lhu(_)
        | Instruction::Sb(_)
        | Instruction::Sh(_)
        | Instruction::Sw(_)
        | Instruction::Beq(_)
        | Instruction::Bne(_)
        | Instruction::Blt(_)
        | Instruction::Bge(_)
        | Instruction::Bltu(_)
        | Instruction::Bgeu(_)
        | Instruction::Jal(_)
        | Instruction::Jalr(_)
        | Instruction::Lui(_)
        | Instruction::Auipc(_)
        | Instruction::Addiw(_)
        | Instruction::Slliw(_)
        | Instruction::Srliw(_)
        | Instruction::Sraiw(_)
        | Instruction::Addw(_)
        | Instruction::Subw(_)
        | Instruction::Sllw(_)
        | Instruction::Srlw(_)
        | Instruction::Sraw(_)
        | Instruction::Ld(_)
        | Instruction::Lwu(_)
        | Instruction::Sd(_) => lower_i(insn, current_pc, next_pc),

        // M instructions
        #[cfg(feature = "ext_m")]
        Instruction::Mul(_)
        | Instruction::Mulh(_)
        | Instruction::Mulhsu(_)
        | Instruction::Mulhu(_)
        | Instruction::Mulw(_)
        | Instruction::Div(_)
        | Instruction::Divu(_)
        | Instruction::Rem(_)
        | Instruction::Remu(_)
        | Instruction::Divw(_)
        | Instruction::Divuw(_)
        | Instruction::Remw(_)
        | Instruction::Remuw(_) => lower_m(insn, current_pc, next_pc),

        // A instructions
        #[cfg(feature = "ext_a")]
        Instruction::LrW(_)
        | Instruction::ScW(_)
        | Instruction::AmoSwapW(_)
        | Instruction::AmoAddW(_)
        | Instruction::AmoXorW(_)
        | Instruction::AmoAndW(_)
        | Instruction::AmoOrW(_)
        | Instruction::AmoMinW(_)
        | Instruction::AmoMaxW(_)
        | Instruction::AmoMinuW(_)
        | Instruction::AmoMaxuW(_)
        | Instruction::LrD(_)
        | Instruction::ScD(_)
        | Instruction::AmoSwapD(_)
        | Instruction::AmoAddD(_)
        | Instruction::AmoXorD(_)
        | Instruction::AmoAndD(_)
        | Instruction::AmoOrD(_)
        | Instruction::AmoMinD(_)
        | Instruction::AmoMaxD(_)
        | Instruction::AmoMinuD(_)
        | Instruction::AmoMaxuD(_) => lower_a(insn, current_pc, next_pc),

        // Other instructions
        _ => lower_i(insn, current_pc, next_pc), // fallback to I for now
    }
}

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
                let func = lower_instruction(insn, current_pc, next_pc);
                execute_ir(&func, vm, &mut self.io);

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
            let (insn, insn_bytes, is_compressed) = {
                #[cfg(feature = "ext_c")]
                {
                    let insn = vm.load_u16(vm.pc() as usize);
                    let is_compressed = insn & mask16(2) != 0b11;
                    if is_compressed {
                        (decode_compressed(insn), insn as u32, true)
                    } else {
                        let insn_upper = vm.load_u16((vm.pc() + 2) as usize);
                        let insn = (insn_upper as u32) << 16 | insn as u32;
                        (decode::decode(insn), insn, false)
                    }
                }
                #[cfg(not(feature = "ext_c"))]
                {
                    let insn = vm.load_u32(vm.pc() as usize);
                    (decode::decode(insn), insn, false)
                }
            };

            if let Instruction::Illegal(_) = &insn {
                vm.exit_code = 1;
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
            let func = lower_instruction(&insn, current_pc, next_pc);
            execute_ir(&func, vm, &mut self.io);

            // Record next PC (set during execute_instruction or default to pc+4)
            vm.tracer.record_next_pc(vm.pc());

            self.cycles = self.cycles.wrapping_add(1);

            // Check for halt
            if vm.halted {
                vm.tracer.record_halt();
                vm.tracer.commit();
                break;
            }

            let is_branch = insn.is_branch_or_jmp();
            block.push((insn, insn_bytes, is_compressed));
            vm.tracer.commit();

            if is_branch {
                self.basic_blocks.insert(leader, block);
                break;
            }
        }
    }
}
