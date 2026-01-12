//! Example binary that runs the prebuilt RISC‑V `echo` guest using the local `riscv` VM.
//!
//! Usage:
//!  - Run this example (from the `riscv` crate root):
//!      cargo run -p riscv --example echo_gc --release
//!
//! Note: enable the `trace` feature on the `riscv` crate if you want full tracing output
//! (depends on how your workspace/crate features are configured).
use std::path::Path;

use riscv::VM;
use riscv::trace::NoopTracer;

/// Path to the prebuilt guest ELF produced by the `rust-bin/echo` crate.
const ECHO_BINARY: &str = "test-bin/rust-bin/echo/echo-gc";

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
    let mut vm = VM::<NoopTracer>::init_from_elf(ECHO_BINARY.to_string());
    vm.input_stream = "Hola Riscv, buenos días".as_bytes().to_vec();
    vm.input_cursor = 0;

    println!("Running echo program GC...\n");

    vm.run_with_timing();

    println!("\nexit_code: {}", vm.exit_code());
}
