//! Example binary that runs the prebuilt RISC‑V `echo` guest using the local `riscv` VM.
//!
//! Usage:
//!  - Run this example (from the `riscv` crate root):
//!      cargo run -p riscv --example echo_ima --release
//!
use std::path::Path;

use riscv::{Runner, init_from_elf};

#[path = "perf_stat.rs"]
mod perf_stat;

/// Path to the prebuilt guest ELF produced by the `rust-bin/echo` crate.
const ECHO_BINARY: &str = "test-bin/rust-bin/echo/echo-ima";

fn main() {
    println!("RISC-V echo example: loading ELF: {}", ECHO_BINARY);

    if !Path::new(ECHO_BINARY).exists() {
        eprintln!(
            "guest binary not found: {}\n\
             build the guest first (see README or run the rust-bin/echo build target).",
            ECHO_BINARY
        );
        return;
    }

    let mut vm = init_from_elf(ECHO_BINARY.to_string());
    let mut runner = Runner::new();
    runner.set_input_stream("Hola Riscv, buenos días".as_bytes().to_vec());

    println!("Running echo program IMA...\n");

    runner.run_with_timing(&mut vm);

    println!("\nexit_code: {}", vm.exit_code());

    perf_stat::print_perf_stat(&runner, &vm, "echo_ima");

    assert_eq!(runner.cycles(), 112);
}
