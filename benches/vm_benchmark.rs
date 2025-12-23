//! Benchmark comparing VM execution with and without tracing.
//!
//! This benchmark measures the overhead of the tracing system by running
//! the fibonacci program with both `NoopTracer` (zero-cost) and `FullTracer`
//! (captures full execution trace).
//!
//! Run with: `cargo bench`

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use riscv::VM;
use riscv::trace::{FullTracer, NoopTracer};

/// Path to the fibonacci binary
const FIB_BINARY: &str = "rust-bin/fib/target/riscv64ima-unknown-none-elf/release/fib"; // I intend to change to the block exec program... I trust that data more :)

/// Benchmark running the fibonacci program without tracing (NoopTracer).
///
/// This represents the fastest possible execution as all tracing calls
/// are optimized away at compile time.
fn bench_fib_no_tracer(c: &mut Criterion) {
    c.bench_function("fib_no_tracer", |b| {
        b.iter(|| {
            let mut vm = VM::<NoopTracer>::init_from_elf(FIB_BINARY.to_string());
            vm.run();
            black_box(vm.exit_code())
        });
    });
}

/// Benchmark running the fibonacci program with full tracing (FullTracer).
///
/// This captures a complete execution trace including all register states,
/// memory operations, and instruction metadata for each cycle.
fn bench_fib_with_tracer(c: &mut Criterion) {
    c.bench_function("fib_full_tracer", |b| {
        b.iter(|| {
            let mut vm = VM::<FullTracer>::init_from_elf(FIB_BINARY.to_string());
            vm.run();
            let trace = vm.take_trace();
            black_box(trace)
        });
    });
}

/// Comparative benchmark showing both tracers side by side.
///
/// This benchmark group makes it easy to compare the overhead
/// introduced by the full tracing system.
fn bench_tracer_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_tracer_comparison");

    group.bench_function(BenchmarkId::new("tracer", "noop"), |b| {
        b.iter(|| {
            let mut vm = VM::<NoopTracer>::init_from_elf(FIB_BINARY.to_string());
            vm.run();
            black_box(vm.exit_code())
        });
    });

    group.bench_function(BenchmarkId::new("tracer", "full"), |b| {
        b.iter(|| {
            let mut vm = VM::<FullTracer>::init_from_elf(FIB_BINARY.to_string());
            vm.run();
            let trace = vm.take_trace();
            black_box(trace)
        });
    });

    group.finish();
}

/// Benchmark that isolates just the VM execution (excluding ELF loading).
///
/// This provides more accurate measurements of the actual execution
/// performance by loading the ELF once and cloning the initial state.
fn bench_execution_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm_execution_only");

    // Pre-load the ELF to isolate execution time
    group.bench_function(BenchmarkId::new("execution", "noop"), |b| {
        b.iter_batched(
            || VM::<NoopTracer>::init_from_elf(FIB_BINARY.to_string()),
            |mut vm| {
                vm.run();
                black_box(vm.exit_code())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function(BenchmarkId::new("execution", "full"), |b| {
        b.iter_batched(
            || VM::<FullTracer>::init_from_elf(FIB_BINARY.to_string()),
            |mut vm| {
                vm.run();
                black_box(vm.take_trace())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_fib_no_tracer,
    bench_fib_with_tracer,
    bench_tracer_comparison,
    bench_execution_only,
);

criterion_main!(benches);
