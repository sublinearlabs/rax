//! RISC-V VM runner binary.
//!
//! Runs the fibonacci program with timing information.
//!
//! Usage: cargo run --release

use riscv::VM;
use riscv::trace::NoopTracer;

/// Path to the fibonacci binary
const FIB_BINARY: &str = "rust-bin/fib/target/riscv64ima-unknown-none-elf/release/fib";

fn main() {
    println!("Loading ELF: {}", FIB_BINARY);

    let mut vm = VM::<NoopTracer>::init_from_elf(FIB_BINARY.to_string());

    println!("Running fibonacci program...\n");

    vm.run_with_timing();

    println!("\nexit_code: {}", vm.exit_code());
}
