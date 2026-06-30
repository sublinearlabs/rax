# RISC-V 64GC ISA

This is an implementation of the RISC-V RV64GC ISA which incorporates:
- **I** - Base Integer Instructions
- **M** - Multiply/Divide Instructions
- **A** - Atomic Instructions
- **F** - Single-Precision Floating-Point Instructions
- **D** - Double-Precision FLoating-Point Instructions
- **C** - Compressed Instructions(16-bit)

It supports trace generation for the execution of programs that can be compiled down to ELF.

## CLI Tools

### RISC-V CLI (`riscv-cli`)

Execute and analyze RISC-V ELF binaries.

#### Building

```bash
cargo build --release -p riscv-cli
```

#### Commands

##### `run` - Execute a RISC-V ELF binary

```bash
riscv-cli run <BINARY> [OPTIONS]

Options:
  -t, --trace              Enable instruction tracing
  -f, --format <FORMAT>    Output format: text, json, csv [default: text]
  -o, --output <FILE>      Save output to file
  -v, --verbose...         Verbosity level (0-3)
```

**Example:**

```bash
riscv-cli run test-bin/rust-bin/fib/fib-imac --format json --output results.json
```

##### `compile` - Compile a RISC-V ELF to a native x86-64 executable

```bash
riscv-cli compile <input> <output>
```

**Example:**

```bash
riscv-cli compile test-bin/rust-bin/echo/echo-ima ./echo-native
echo "Hola" | ./echo-native
```

## Tracing

Direct API usage for tracing:

```rust
let mut vm = VM::<FullTracer>::init_from_elf(<path-to-elf>);
vm.run_with_timing();
```

Without tracing:

```rust
let mut vm = VM::<NoopTracer>::init_from_elf(<path-to-elf>);
vm.run_with_timing();
```

## Examples

Run the bundled ELF examples through the CLI:

```bash
cargo run --bin riscv-cli -- run test-bin/rust-bin/fib/fib-imac
```

## Benchmarks

## Resources

- https://www.cs.sfu.ca/~ashriram/Courses/CS295/assets/notebooks/RISCV/RISCV_CARD.pdf
- https://docs.riscv.org/reference/isa/_attachments/riscv-unprivileged.pdf
- https://msyksphinz-self.github.io/riscv-isadoc/html/rvfd.html
