use super::{HostIO, VM};
#[cfg(feature = "ext_c")]
use rax_core::decode::compressed::decode_compressed;
use rax_core::decode::decode;
#[cfg(feature = "ext_c")]
use rax_core::util::mask16;

pub struct Runner {
    io: HostIO,
    cycles: u64,
    elapsed: std::time::Duration,
}

fn fetch_insn(vm: &mut VM) -> (rax_core::decode::Instruction, bool) {
    let pc = vm.pc();
    #[cfg(feature = "ext_c")]
    {
        let raw = vm.load_u16(pc as usize);
        let is_compressed = raw & mask16(2) != 0b11;
        if is_compressed {
            (decode_compressed(raw), true)
        } else {
            let raw = vm.load_u32(pc as usize);
            (decode(raw), false)
        }
    }
    #[cfg(not(feature = "ext_c"))]
    {
        let raw = vm.load_u32(pc as usize);
        (decode(raw), false)
    }
}

impl Runner {
    pub fn new() -> Self {
        Self {
            io: HostIO::new(),
            cycles: 0,
            elapsed: std::time::Duration::default(),
        }
    }

    pub fn set_input_stream(&mut self, input: Vec<u8>) {
        self.io.set_input_stream(input);
    }

    pub fn set_capture_output(&mut self, capture_output: bool) {
        self.io.set_capture_output(capture_output);
    }

    pub fn stdout(&self) -> &[u8] {
        self.io.stdout()
    }

    pub fn stderr(&self) -> &[u8] {
        self.io.stderr()
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
            let current_pc = vm.pc();
            let (insn, is_compressed) = fetch_insn(vm);
            let next_pc = current_pc.wrapping_add(if is_compressed { 2 } else { 4 });
            vm.set_pc(next_pc);
            vm.execute_instruction(&insn, is_compressed, current_pc, &mut self.io);
            self.cycles = self.cycles.wrapping_add(1);
        }
        self.elapsed = start.elapsed();
    }

    pub fn step(&mut self, vm: &mut VM) {
        let current_pc = vm.pc();
        let (insn, is_compressed) = fetch_insn(vm);
        let next_pc = current_pc.wrapping_add(if is_compressed { 2 } else { 4 });
        vm.set_pc(next_pc);
        vm.execute_instruction(&insn, is_compressed, current_pc, &mut self.io);
        self.cycles = self.cycles.wrapping_add(1);
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
}
