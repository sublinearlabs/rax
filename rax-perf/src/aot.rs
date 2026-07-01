use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use rax_aot::compiler::compile_elf_file_with_stats;
use rax_interpreter::init_from_elf;
use serde::{Deserialize, Serialize};

use crate::benchmarks::{aot_benchmarks, Benchmark};
use crate::timing::{duration_ns, median};
use crate::PerfResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct PerfSuite {
    pub kind: String,
    pub runs: usize,
    pub warmups: usize,
    pub benchmarks: Vec<BenchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchResult {
    pub name: String,
    pub elf_path: String,
    pub jit_exit_code: u64,
    pub aot_exit_code: i32,
    pub guest_instructions: u64,
    pub compile_ns: u64,
    pub native_run_ns_median: u64,
    pub native_run_ns_min: u64,
    pub native_run_ns_samples: Vec<u64>,
    pub effective_guest_mhz: f64,
    pub riscv_static_instructions: u64,
    pub x86_static_instructions: u64,
    pub x86_per_riscv: f64,
    pub x86_code_bytes: u64,
    pub jump_table_bytes: u64,
    pub output_size: u64,
    pub stdout_matches: bool,
    pub stderr_matches: bool,
}

struct GuestRun {
    exit_code: u64,
    guest_instructions: u64,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

struct NativeRun {
    exit_code: i32,
    elapsed_ns: u64,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

pub fn run_suite(runs: usize, warmups: usize) -> PerfResult<PerfSuite> {
    if runs == 0 {
        return Err("--runs must be greater than zero".into());
    }

    let artifact_dir = PathBuf::from("target/perf/aot-artifacts");
    fs::create_dir_all(&artifact_dir)?;

    let mut results = Vec::new();
    for benchmark in aot_benchmarks()? {
        println!("benchmarking {}", benchmark.name);
        results.push(run_benchmark(&benchmark, &artifact_dir, runs, warmups)?);
    }

    Ok(PerfSuite {
        kind: "aot".to_string(),
        runs,
        warmups,
        benchmarks: results,
    })
}

fn run_benchmark(
    benchmark: &Benchmark,
    artifact_dir: &Path,
    runs: usize,
    warmups: usize,
) -> PerfResult<BenchResult> {
    let guest = run_guest_counter(benchmark)?;
    let native_path = artifact_dir.join(format!("{}-{}", benchmark.name, std::process::id()));

    let compile_start = Instant::now();
    let compile_stats = compile_elf_file_with_stats(&benchmark.elf_path, &native_path)?;
    let compile_ns = duration_ns(compile_start.elapsed());
    let output_size = fs::metadata(&native_path)?.len();

    for _ in 0..warmups {
        let _ = run_native(&native_path, &benchmark.stdin)?;
    }

    let mut samples = Vec::with_capacity(runs);
    let mut aot_exit_code = 0;
    let mut stdout_matches = true;
    let mut stderr_matches = true;

    for _ in 0..runs {
        let native = run_native(&native_path, &benchmark.stdin)?;
        aot_exit_code = native.exit_code;
        stdout_matches &= native.stdout == guest.stdout;
        stderr_matches &= native.stderr == guest.stderr;
        samples.push(native.elapsed_ns);
    }

    let _ = fs::remove_file(&native_path);

    let native_run_ns_median = median(&samples).expect("runs checked above");
    let native_run_ns_min = samples.iter().copied().min().expect("runs checked above");
    let effective_guest_mhz =
        guest.guest_instructions as f64 * 1000.0 / native_run_ns_median as f64;

    Ok(BenchResult {
        name: benchmark.name.to_string(),
        elf_path: benchmark.elf_path.display().to_string(),
        jit_exit_code: guest.exit_code,
        aot_exit_code,
        guest_instructions: guest.guest_instructions,
        compile_ns,
        native_run_ns_median,
        native_run_ns_min,
        native_run_ns_samples: samples,
        effective_guest_mhz,
        riscv_static_instructions: compile_stats.riscv_instruction_count as u64,
        x86_static_instructions: compile_stats.x86_instruction_count as u64,
        x86_per_riscv: compile_stats.x86_instructions_per_riscv_instruction(),
        x86_code_bytes: compile_stats.x86_code_bytes as u64,
        jump_table_bytes: compile_stats.jump_table_bytes as u64,
        output_size,
        stdout_matches,
        stderr_matches,
    })
}

fn run_guest_counter(benchmark: &Benchmark) -> PerfResult<GuestRun> {
    let mut vm = init_from_elf(&benchmark.elf_path);
    let mut runner = rax_jit::Runner::new();
    runner.set_input_stream(benchmark.stdin.clone());
    runner.set_capture_output(true);
    runner.run(&mut vm);

    Ok(GuestRun {
        exit_code: vm.exit_code(),
        guest_instructions: runner.cycles(),
        stdout: runner.stdout().to_vec(),
        stderr: runner.stderr().to_vec(),
    })
}

fn run_native(path: &Path, stdin: &[u8]) -> PerfResult<NativeRun> {
    let start = Instant::now();
    let mut child = Command::new(path)
        .stdin(if stdin.is_empty() {
            Stdio::null()
        } else {
            Stdio::piped()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if !stdin.is_empty() {
        if let Some(mut child_stdin) = child.stdin.take() {
            if let Err(err) = child_stdin.write_all(stdin) {
                if err.kind() != io::ErrorKind::BrokenPipe {
                    return Err(Box::new(err));
                }
            }
        }
    }

    let output = child.wait_with_output()?;
    let elapsed_ns = duration_ns(start.elapsed());

    Ok(NativeRun {
        exit_code: output.status.code().unwrap_or(-1),
        elapsed_ns,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}
