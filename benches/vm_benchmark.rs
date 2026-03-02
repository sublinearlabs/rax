//! Benchmark VM execution performance.
//!
//! Run with: `cargo bench`

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use riscv::{Runner, init_from_elf};

/// Path to the fibonacci binary
const FIB_BINARY: &str = "test-bin/rust-bin/fib/fib-ima"; // I intend to change to the block exec program... I trust that data more :)

/// Benchmark running the fibonacci program.
fn bench_fib(c: &mut Criterion) {
    c.bench_function("fib", |b| {
        b.iter(|| {
            let mut vm = init_from_elf(FIB_BINARY.to_string());
            let mut runner = Runner::new();
            runner.run(&mut vm);
            black_box(vm.exit_code())
        });
    });
}

/// Benchmark that isolates just the VM execution (excluding ELF loading).
///
/// This provides more accurate measurements of the actual execution
/// performance by loading the ELF once and cloning the initial state.
fn bench_execution_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_execution_only");

    // Pre-load the ELF to isolate execution time
    group.bench_function(BenchmarkId::new("execution", "run"), |b| {
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

criterion_group!(benches, bench_fib, bench_execution_only,);

criterion_main!(benches);
