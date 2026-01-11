//! Example binary that runs the prebuilt RISC‑V `fib` guest using the local `riscv` VM.
//!
//! Usage:
//!  - Run this example (from the `riscv` crate root):
//!      cargo run -p riscv --example fib_ima --release
//!
//! Note: enable the `trace` feature on the `riscv` crate if you want full tracing output
//! (depends on how your workspace/crate features are configured).
use std::path::Path;

use riscv::VM;
use riscv::trace::FullTracer;

/// Path to the prebuilt guest ELF produced by the `rust-bin/fib` crate.
const FIB_BINARY: &str = "test-bin/rust-bin/fib/fib-ima";


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

    // Construct a VM using the FullTracer tracer implementation (same as original main).
    let mut vm = VM::<FullTracer>::init_from_elf(FIB_BINARY.to_string());

    println!("Running fibonacci program IMA...\n");

    vm.run_with_timing();

    println!("\nexit_code: {}", vm.exit_code());
}
