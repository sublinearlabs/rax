# RISC-V AOT Compiler

A RISC-V to x86-64 ahead-of-time compiler for ELF binaries.

This project focuses on compiling supported RISC-V ELF programs into native x86-64 ELF executables. It also includes interpreter and JIT backends for validation, debugging, and comparison.

## Status

This project is experimental and under active development. The AOT path is the primary focus.

## Highlights

- Compiles supported RISC-V ELF binaries to native x86-64 ELF executables.
- Reports compile-time codegen stats, including static x86/RISC-V instruction ratio.
- Provides interpreter and JIT execution paths for comparison and validation.
- Includes contributor tooling for AOT performance regression checks.

## Supported ISA

The AOT compiler supports RV64IMA. Other ISA extensions exist in the workspace behind feature flags, but the default build is RV64IMA-focused.

## Quick Start

Build the CLI:

```sh
cargo build --release -p riscv-cli
```

Compile a RISC-V ELF to a native x86-64 executable:

```sh
./target/release/riscv compile test-bin/rust-bin/fib/fib-ima -o /tmp/fib-native
```

Run the native executable:

```sh
/tmp/fib-native
```

## AOT Compilation

The main CLI command is `compile`:

```sh
riscv compile <input-riscv-elf> -o <output-x86-elf>
```

Example:

```sh
cargo run -p riscv-cli -- compile test-bin/rust-bin/fib/fib-ima -o /tmp/fib-native
```

Example output:

```text
Compiling test-bin/rust-bin/fib/fib-ima to /tmp/fib-native
Output written to /tmp/fib-native

Compilation Stats
------------------------------------------------------------
metric  value
compile 1.821ms
riscv   27
x86     60
x86/rv  2.22
code    316B
jtable  216B
output  8.73KiB
```

## Compile Stats

The compiler reports static codegen stats for each AOT build:

- `compile`: elapsed AOT compilation time
- `riscv`: static RISC-V instructions in the translated executable segment
- `x86`: static x86 instructions emitted for translated code
- `x86/rv`: average x86 instructions emitted per RISC-V instruction
- `code`: translated x86 code bytes, excluding jump-table data
- `jtable`: runtime jump-table bytes
- `output`: final native ELF size

Jump-table bytes are reported separately and are not included in the x86 instruction count.

## Running RISC-V Programs

The interpreter and JIT are useful for debugging, validation, and comparison.

Run with the interpreter:

```sh
riscv run program.elf
```

Run with the JIT:

```sh
riscv run --jit program.elf
```

When developing from the workspace, use Cargo:

```sh
cargo run -p riscv-cli -- run test-bin/rust-bin/fib/fib-ima
cargo run -p riscv-cli -- run --jit test-bin/rust-bin/fib/fib-ima
```

## Library Usage

Use the AOT compiler directly:

```rust
use riscv::aot::compiler::compile_elf_file;

compile_elf_file("program.elf", "program-native")?;
```

Compile and inspect codegen stats:

```rust
use riscv::aot::compiler::compile_elf_file_with_stats;

let stats = compile_elf_file_with_stats("program.elf", "program-native")?;
println!("x86/rv = {:.2}", stats.x86_instructions_per_riscv_instruction());
```

Run through the interpreter:

```rust
use riscv::{init_from_elf, Runner};

let mut vm = init_from_elf("program.elf");
let mut runner = Runner::new();
runner.run(&mut vm);
```

The JIT backend is optional for library users to avoid pulling Cranelift into default builds:

```toml
riscv = { version = "0.1", features = ["jit"] }
```

## Workspace Crates

- `riscv`: facade crate for common library APIs
- `riscv-core`: instruction decoding and shared utilities
- `riscv-interpreter`: interpreter VM and runner
- `riscv-aot`: RISC-V to x86-64 AOT compiler
- `riscv-jit`: Cranelift-backed JIT runner
- `riscv-elfgen`: ELF analysis and emission helpers
- `riscv-cli`: command-line interface
- `riscv-perf`: contributor-only performance tooling

## Contributor Tooling

For AOT compiler development, the repository includes a branch-to-branch perf tool:

```sh
make perf-aot
```

This command requires a clean working tree. It compares the current branch against `main` by default, writes artifacts under `target/perf/`, and prints a report with compile-time, native run-time, code-size, jump-table-size, and x86/RISC-V instruction-ratio deltas.

You can tune the run count:

```sh
make perf-aot PERF_RUNS=11 PERF_WARMUPS=2
```

Or compare against a different base branch:

```sh
make perf-aot PERF_BASE=origin/main
```

## Limitations

- AOT output currently targets x86-64 ELF.
- The current AOT path is intended for uncompressed RISC-V instruction streams.
- Syscall support is Linux-like and limited.
- The JIT backend is optional because it depends on Cranelift, which increases build times.

## Resources

- https://www.cs.sfu.ca/~ashriram/Courses/CS295/assets/notebooks/RISCV/RISCV_CARD.pdf
- https://docs.riscv.org/reference/isa/_attachments/riscv-unprivileged.pdf
- https://msyksphinz-self.github.io/riscv-isadoc/html/rvfd.html
