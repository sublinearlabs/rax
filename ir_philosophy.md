# IR Philosophy

## Purpose
The IR represents a RISC-V basic block as a typed SSA graph that is cheap to
interpret and easy to lower into faster execution forms. The goal is to make
state transitions explicit and optimize around them.

## Core Model
- A function is a set of basic blocks.
- A basic block is a linear sequence of ops ending in a terminator.
- Values are immutable SSA IDs with explicit types.
- Pure ops operate only on SSA values.
- Effect ops are the only way to read or mutate VM state.

## Minimum Pure Ops
These are total functions on SSA values with no observable side effects:
- Constants: const i64
- Integer arithmetic: add, sub, mul, div, rem
- Bitwise: and, or, xor
- Shifts: shl, shr, sar
- Comparisons: eq, ne, lt, ltu, ge, geu
- Type ops: sext, zext, trunc
- Select: select(cond, t, f)

Everything else is reducible to this set.

## Minimum Effect Ops
These are the only operations that may read or modify VM state:
- Integer registers: get_reg, set_reg
- PC: get_pc, set_pc
- Memory: load/store with width and sign/zero semantics
- CSR: get_csr, set_csr
- Atomics/reservation: lr/sc/amo
- Traps/exit: ecall, ebreak, halt

No other op may touch VM state.

## Control Flow Invariants
- Every block ends in a terminator (br, cbr, ret).
- No ops are allowed after a terminator.
- Effect ops preserve program order.
- Block arguments are typed and must match on all incoming edges.
- IMA blocks have at most two successors (br or cbr). Jump instructions end the
  block.

## NOP Semantics
"No state change" means no effect ops, but PC is state. An ISA NOP still maps
to a PC update (e.g., set_pc(next_pc)) or an equivalent block-level policy.

## Builder Responsibilities
- Enforce typing and block argument correctness.
- Enforce terminator presence and placement.
- Provide small, transparent helpers (addr, pc_plus, shamt) that expand into
  core ops without hiding effects.

## Linear Composition
Sequential composition is explicit and conservative:
- An IR fragment is mergeable only if it has exactly one leaf block.
- A leaf block is a block whose terminator is Ret.
- Appending B after A is allowed only when A has one leaf; otherwise the merge
  fails and the user must add a join block or mark the instruction as
  terminating.
- The builder must not pick a "last switched" block implicitly when multiple
  leaves exist.

## Extensions
Higher-level compound ops are allowed only as builder sugar that lowers to the
minimum pure/effect op set. New effect ops are acceptable only if they represent
an explicit VM state transition not already expressible.
