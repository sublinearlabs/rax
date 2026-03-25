## RISC-V 64GC ISA
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
cargo build --release --bin riscv-cli
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

##### `verify-block` - Verify Ethereum block on RISC-V VM
```bash
riscv-cli verify-block <BLOCK_NUMBER> [OPTIONS]

Options:
  -b, --binary <BINARY>        Path to RISC-V verifier binary (required)
  -r, --rpc-url <RPC_URL>      Ethereum RPC endpoint (required)
  -w, --witness <WITNESS>      Path to witness file (cached if exists)
  -f, --format <FORMAT>        Output format: text, json, csv [default: text]
  -o, --output <FILE>          Save output to file
  -v, --verbose...             Verbosity level (0-3)
```

**Features:**
- Fetches Ethereum block data via RPC
- Generates execution witness using `eth-cli`
- Caches witness files for reuse (skips RPC fetch on repeat runs)
- Executes witness on RISC-V VM with cycle/timing measurements
- Supports multiple output formats

**Example:**
```bash
# Generate and verify block, save witness for later use
riscv-cli verify-block 24628522 \
  --binary test-bin/rust-bin/exec-block/exec-block-imac \
  --rpc-url "https://eth.llamarpc.com" \
  --witness /tmp/block_24628522.json \
  --format json \
  --output verification_result.json

# Reuse cached witness (no RPC call needed)
riscv-cli verify-block 24628522 \
  --binary test-bin/rust-bin/exec-block/exec-block-imac \
  --rpc-url "https://eth.llamarpc.com" \
  --witness /tmp/block_24628522.json
```

### Ethereum CLI (`eth-cli`)

Fetch and analyze Ethereum block data.

#### Building
```bash
cargo build --release -p eth_utils --bin eth-cli
```

#### Commands

##### `fetch` - Fetch Ethereum block data
```bash
eth-cli fetch <BLOCK_NUMBER> --rpc-url <RPC_URL> [OPTIONS]

Options:
  --rpc-url <RPC_URL>      Ethereum RPC endpoint (required)
  -f, --format <FORMAT>    Output format: text, json, csv [default: text]
  -o, --output <FILE>      Save output to file
  -v, --verbose...         Verbosity level (0-3)
```

**Example:**
```bash
eth-cli fetch 24628522 --rpc-url "https://eth.llamarpc.com" --format json
```

##### `generate-witness` - Generate execution witness for a block
```bash
eth-cli generate-witness <BLOCK_NUMBER> --rpc-url <RPC_URL> [OPTIONS]

Options:
  --rpc-url <RPC_URL>      Ethereum RPC endpoint (required)
  -f, --format <FORMAT>    Output format: text, json, csv [default: text]
  -o, --output <FILE>      Save witness to file (hex-encoded)
  -v, --verbose...         Verbosity level (0-3)
```

**Features:**
- Fetches block from RPC
- Traces execution to generate state changes
- Builds complete execution witness structure
- When saving to file: outputs hex-encoded witness bytes (compatible with RISC-V VM input)
- When displaying: outputs formatted text/json/csv

**Example:**
```bash
# Generate witness and save for reuse
eth-cli generate-witness 24628522 \
  --rpc-url "https://eth.llamarpc.com" \
  --output /tmp/witness.hex

# Display witness (stdout)
eth-cli generate-witness 24628522 \
  --rpc-url "https://eth.llamarpc.com" \
  --format json
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

Run the Ethereum stateless block execution example:
```bash
cargo run --example live-block --release
```

This example demonstrates:
1. Fetching real block data from Ethereum RPC
2. Tracing block execution to generate state changes
3. Building complete execution witness
4. Serializing witness for guest validator
5. Running exec-block test binary with the witness

## Benchmarks
## Resources
- https://www.cs.sfu.ca/~ashriram/Courses/CS295/assets/notebooks/RISCV/RISCV_CARD.pdf
- https://docs.riscv.org/reference/isa/_attachments/riscv-unprivileged.pdf
- https://msyksphinz-self.github.io/riscv-isadoc/html/rvfd.html

