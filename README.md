## RISC-V 64GC ISA
This is an implementation of the RISC-V RV64GC ISA which incorporates:
- **I** - Base Integer Instructions
- **M** - Multiply/Divide Instructions
- **A** - Atomic Instructions
- **F** - Single-Precision Floating-Point Instructions
- **D** - Double-Precision FLoating-Point Instructions
- **C** - Compressed Instructions(16-bit)

It runs programs compiled down to ELF.

```rust
let mut vm = init_from_elf(<path-to-elf>);
let mut runner = Runner::new();
runner.run_with_timing(&mut vm);
```

To run the Ethereum stateless block execution program, use this;
```
cargo run -p riscv --example exec-block --release
```

## Benchmarks
## Resources
- https://www.cs.sfu.ca/~ashriram/Courses/CS295/assets/notebooks/RISCV/RISCV_CARD.pdf
- https://docs.riscv.org/reference/isa/_attachments/riscv-unprivileged.pdf
- https://msyksphinz-self.github.io/riscv-isadoc/html/rvfd.html
