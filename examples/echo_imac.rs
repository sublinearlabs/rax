//! Example binary that runs the prebuilt RISC‑V `echo` guest using the local `riscv` VM.
//!
//! Usage:
//!  - Run this example (from the `riscv` crate root):
//!      cargo run -p riscv --example echo_imac --release
//!
//! Note: enable the `trace` feature on the `riscv` crate if you want full tracing output
//! (depends on how your workspace/crate features are configured).
use std::path::Path;

use riscv::trace::NoopTracer;
use riscv::{init_from_elf, Runner};

#[path = "perf_stat.rs"]
mod perf_stat;

/// Path to the prebuilt guest ELF produced by the `rust-bin/echo` crate.
const ECHO_BINARY: &str = "test-bin/rust-bin/echo/echo-imac";

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

    // Construct a VM using the FullTracer tracer implementation (same as original main).
    let mut vm = init_from_elf::<NoopTracer>(ECHO_BINARY.to_string());
    let mut runner = Runner::new();
    runner.set_input_stream("Hola Riscv, buenos días".as_bytes().to_vec());

    println!("Running echo program IMAC...\n");

    runner.run_with_timing(&mut vm);

    println!("\nexit_code: {}", vm.exit_code());

    perf_stat::print_perf_stat(&runner, &vm, "echo_imac");

    assert_eq!(runner.cycles(), 98);
}
