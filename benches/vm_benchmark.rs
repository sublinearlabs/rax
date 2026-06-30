//! Benchmark the fibonacci program with the interpreter.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use riscv::{init_from_elf, Runner};

/// Path to the fibonacci binary
const FIB_BINARY: &str = "test-bin/rust-bin/fib/fib-ima";

/// Benchmark running the fibonacci program with the interpreter.
fn bench_fib(c: &mut Criterion) {
    c.bench_function("fib_interpreter", |b| {
        b.iter(|| {
            let mut vm = init_from_elf(FIB_BINARY.to_string());
            let mut runner = Runner::new();
            runner.run(&mut vm);
            black_box(vm.exit_code())
        });
    });
}

/// Benchmark that isolates just the VM execution (excluding ELF loading).
fn bench_execution_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_execution_only");

    group.bench_function("execution", |b| {
        b.iter_batched(
            || init_from_elf(FIB_BINARY.to_string()),
            |mut vm| {
                let mut runner = Runner::new();
                runner.run(&mut vm);
                black_box(vm.exit_code())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_fib, bench_execution_only);

criterion_main!(benches);
