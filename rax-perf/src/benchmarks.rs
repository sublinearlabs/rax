use std::fs;
use std::path::PathBuf;

use crate::PerfResult;

pub struct Benchmark {
    pub name: &'static str,
    pub elf_path: PathBuf,
    pub stdin: Vec<u8>,
}

pub fn aot_benchmarks() -> PerfResult<Vec<Benchmark>> {
    Ok(vec![
        Benchmark {
            name: "fib_ima",
            elf_path: PathBuf::from("test-bin/rust-bin/fib/fib-ima"),
            stdin: Vec::new(),
        },
        Benchmark {
            name: "echo_ima",
            elf_path: PathBuf::from("test-bin/rust-bin/echo/echo-ima"),
            stdin: b"hello from rax-perf\n".to_vec(),
        },
        Benchmark {
            name: "exec_block_ima",
            elf_path: PathBuf::from("test-bin/rust-bin/exec-block/exec-block-ima"),
            stdin: exec_block_input()?,
        },
    ])
}

fn exec_block_input() -> PerfResult<Vec<u8>> {
    Ok(fs::read("examples/exec-block.input")?)
}
