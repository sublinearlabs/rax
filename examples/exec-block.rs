//! Example binary that runs the prebuilt RISC‑V `exec-block` guest using the local `riscv` VM.
//!
//! Usage:
//!  - Build the guest first (from repo root):
//!      cargo +nightly build -p rust-bin/exec-block --release --target rust-bin/exec-block/riscv64ima-unknown-none-elf.json
//!  - Run this example (from the `riscv` crate root):
//!      cargo run -p riscv --example exec-block --release
//!
//! Note: enable the `trace` feature on the `riscv` crate if you want full tracing output.
use std::fs;
use std::path::Path;

use riscv::VM;
use riscv::trace::NoopTracer;

const EXEC_BLOCK_BINARY: &str =
    "rust-bin/exec-block/target/riscv64ima-unknown-none-elf/release/exec-block";

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
    let mut vm = VM::<NoopTracer>::init_from_elf(EXEC_BLOCK_BINARY.to_string());

    let input_hex_string = fs::read_to_string("examples/exec-block.input").unwrap();
    let input_hex_string = input_hex_string.trim();
    let bytes = hex::decode(input_hex_string).unwrap();

    vm.set_input_stream(bytes);

    println!("Running exec-block program...\n");

    // In the original project this run was sometimes commented out; we call
    // `run_with_timing` to match the fib example and produce timing output.
    vm.run_with_timing();

    println!("\nexit_code: {}", vm.exit_code());
}
