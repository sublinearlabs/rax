use std::collections::HashMap;

#[cfg(feature = "ext_c")]
use crate::decode::compressed::decode_compressed;
use crate::decode::Instruction;
use crate::ir::execute_ir;
#[cfg(feature = "ext_a")]
use crate::ir::lower::a::lower_a_into;
use crate::ir::lower::i::lower_i_into;
#[cfg(feature = "ext_m")]
use crate::ir::lower::m::lower_m_into;
use crate::ir::{IrBuilder, IrFunction};
use crate::trace::Tracer;
#[cfg(feature = "ext_c")]
use crate::util::mask16;
use crate::HostIO;
use crate::{decode, VM};

fn lower_instruction_into(
    insn: &Instruction,
    current_pc: u64,
    next_pc: u64,
    builder: &mut IrBuilder,
) {
    match insn {
        Instruction::Illegal(_) => {
            builder.halt(1);
            builder.ret();
        }
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
        | Instruction::Sd(_) => lower_i_into(insn, current_pc, next_pc, builder),

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
        | Instruction::Remuw(_) => lower_m_into(insn, current_pc, next_pc, builder),

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
        | Instruction::AmoMaxuD(_) => lower_a_into(insn, current_pc, next_pc, builder),

        // Other instructions
        _ => lower_i_into(insn, current_pc, next_pc, builder), // fallback to I for now
    }
}

pub struct Runner {
    io: HostIO,
    basic_blocks: HashMap<u64, CachedBlock>,
    cycles: u64,
    elapsed: std::time::Duration,
}

struct DecodedBlock {
    insns: Vec<(Instruction, bool)>,
    terminated_by_branch: bool,
    terminated_by_illegal: bool,
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
        // cycles / microseconds = Mhz
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
            block.push((insn, is_compressed));

            if is_branch || is_illegal || is_halt {
                return DecodedBlock {
                    insns: block,
                    terminated_by_branch: is_branch,
                    terminated_by_illegal: is_illegal,
                };
            }

            pc = pc.wrapping_add(if is_compressed { 2 } else { 4 });
        }
    }

    fn lower_basic_block(&self, start_pc: u64, block: &DecodedBlock) -> CachedBlock {
        let mut builder = IrBuilder::new();
        let entry = builder.block();
        builder.switch_to(entry);

        let mut pc = start_pc;
        let mut insn_count = 0u64;
        let mut terminated = false;

        for (insn, is_compressed) in &block.insns {
            let current_pc = pc;
            let next_pc = current_pc.wrapping_add(if *is_compressed { 2 } else { 4 });
            let is_branch = insn.is_branch_or_jmp();
            let is_illegal = matches!(insn, Instruction::Illegal(_));
            let is_halt = matches!(insn, Instruction::Ebreak);

            builder.set_ret_suppressed(!is_branch && !is_illegal && !is_halt);
            lower_instruction_into(insn, current_pc, next_pc, &mut builder);
            insn_count = insn_count.wrapping_add(1);

            if is_illegal || is_halt {
                terminated = true;
                break;
            }

            if is_branch {
                terminated = true;
                break;
            }

            let next_pc_val = builder.imm_u64(next_pc);
            builder.set_pc(next_pc_val);
            pc = next_pc;
        }

        if !terminated {
            builder.set_ret_suppressed(false);
            builder.ret();
        }

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

        if block.terminated_by_branch {
            self.basic_blocks.insert(leader, lowered);
        }
    }
}
