//! Example binary that runs the prebuilt RISC‑V `exec-block` guest using the local `riscv` VM.
//!
//! Usage:
//!  - Run this example (from the `riscv` crate root):
//!      cargo run -p riscv --example exec_block_ima --release
//!
//! Note: enable the `trace` feature on the `riscv` crate if you want full tracing output.
use std::fs;
use std::path::Path;

use riscv::{Runner, init_from_elf};

#[path = "perf_stat.rs"]
mod perf_stat;

const EXEC_BLOCK_BINARY: &str = "test-bin/rust-bin/exec-block/exec-block-ima";

fn main() {
    println!(
        "RISC-V exec-block example: loading ELF: {}",
        EXEC_BLOCK_BINARY
    );

    if !Path::new(EXEC_BLOCK_BINARY).exists() {
        eprintln!(
            "guest binary not found: {}\n\
             build the guest first (see README or run the rust-bin/exec-block build target).",
            EXEC_BLOCK_BINARY
        );
        return;
    }

    // Construct a VM using the FullTracer tracer implementation (same as original main).
    let mut vm = init_from_elf(EXEC_BLOCK_BINARY.to_string());

    let input_hex_string = fs::read_to_string("examples/exec-block.input").unwrap();
    let input_hex_string = input_hex_string.trim();
    let bytes = hex::decode(input_hex_string).unwrap();

    let mut runner = Runner::new();
    runner.set_input_stream(bytes);

    println!("Running exec-block program IMA...\n");

    // In the original project this run was sometimes commented out; we call
    // `run_with_timing` to match the fib example and produce timing output.
    runner.run_with_timing(&mut vm);

    println!("\nexit_code: {}", vm.exit_code());

    perf_stat::print_perf_stat(&runner, &vm, "exec_block_ima");

    assert_eq!(runner.cycles(), 2165224867);
}
