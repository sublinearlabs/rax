//! RISC-V VM runner binary.
//!
//! Runs the fibonacci program with timing information.
//!
//! Usage: cargo run --release

use riscv::VM;
use riscv::trace::FullTracer;

/// Path to the fibonacci binary
const FIB_BINARY: &str = "rust-bin/fib/target/riscv64ima-unknown-none-elf/release/fib";

fn main() {
    println!("Loading ELF: {}", FIB_BINARY);

    let mut vm = VM::<FullTracer>::init_from_elf(FIB_BINARY.to_string());

    println!("Running fibonacci program...\n");

    vm.run_with_timing();

    println!("\nexit_code: {}", vm.exit_code());
}



// No tracing
// run took: 2062230ms
// cycles: 72000006
// 34.91 Mhz
// 
// With tracing
// run took: 2062230ms
// cycles: 72000006
// 34.91 Mhz