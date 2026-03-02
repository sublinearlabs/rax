//! Example binary that runs the prebuilt RISC‑V `fib` guest using the local `riscv` VM.
//!
//! Usage:
//!  - Run this example (from the `riscv` crate root):
//!      cargo run -p riscv --example fib_imac --release
//!
use std::path::Path;

use riscv::{init_from_elf, Runner};

#[path = "perf_stat.rs"]
mod perf_stat;

/// Path to the prebuilt guest ELF produced by the `rust-bin/fib` crate.
const FIB_BINARY: &str = "test-bin/rust-bin/fib/fib-imac";

fn main() {
    println!("RISC-V fib example: loading ELF: {}", FIB_BINARY);

    if !Path::new(FIB_BINARY).exists() {
        eprintln!(
            "guest binary not found: {}\n\
             build the guest first (see README or run the rust-bin/fib build target).",
            FIB_BINARY
        );
        return;
    }

    let mut vm = init_from_elf(FIB_BINARY.to_string());

    println!("Running fibonacci program IMAC...\n");

    let mut runner = Runner::new();
    runner.run_with_timing(&mut vm);

    println!("\nexit_code: {}", vm.exit_code());

    perf_stat::print_perf_stat(&runner, &vm, "fib_imac");

    assert_eq!(runner.cycles(), 72000006);
}
