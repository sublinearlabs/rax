## RISC-V 64GC ISA
This is an implementation of the RISC-V RV64GC ISA which incorporates:
- **I** - Base Integer Instructions
- **M** - Multiply/Divide Instructions
- **A** - Atomic Instructions
- **F** - Single-Precision Floating-Point Instructions
- **D** - Double-Precision FLoating-Point Instructions
- **C** - Compressed Instructions(16-bit)

It supports trace generation for the execution of programs that can be compiled down to ELF.

Tracing
```rust
let mut vm = VM::<FullTracer>::init_from_elf(<path-to-elf>);
vm.run_with_timing();
```

No Tracing
```rust
let mut vm = VM::<NoopTracer>::init_from_elf(<path-to-elf>);
vm.run_with_timing();
```
## Benchmarks
## Resources
- https://www.cs.sfu.ca/~ashriram/Courses/CS295/assets/notebooks/RISCV/RISCV_CARD.pdf
- https://docs.riscv.org/reference/isa/_attachments/riscv-unprivileged.pdf
- https://msyksphinz-self.github.io/riscv-isadoc/html/rvfd.html
