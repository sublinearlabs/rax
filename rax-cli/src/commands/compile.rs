//! RISC-V compile command

use crate::common::{check_file_exists, print_info, print_success, CliResult};
use rax_aot::compiler::compile_elf_file_with_stats;
use std::time::Instant;

const LABEL_WIDTH: usize = 8;

/// Execute the compile command
pub fn execute_compile(input_path: &str, output_path: &str) -> CliResult<()> {
    check_file_exists(input_path)?;
    print_info(&format!("Compiling {} to {}", input_path, output_path));

    let start = Instant::now();
    let stats = compile_elf_file_with_stats(input_path, output_path)?;
    let compile_elapsed = start.elapsed();
    let output_size = std::fs::metadata(output_path)?.len();

    print_success(&format!("Output written to {}", output_path));
    println!(
        "{}",
        format_compile_stats(&stats, compile_elapsed, output_size)
    );
    Ok(())
}

fn format_compile_stats(
    stats: &rax_aot::compiler::AotCompileStats,
    compile_elapsed: std::time::Duration,
    output_size: u64,
) -> String {
    let mut output = String::new();
    output.push_str("\nCompilation Stats\n");
    output.push_str(&format!("{}\n", "-".repeat(60)));
    output.push_str(&row("metric", "value".to_string()));
    output.push_str(&row("compile", format_duration(compile_elapsed)));
    output.push_str(&row(
        "riscv",
        format_integer(stats.riscv_instruction_count as u64),
    ));
    output.push_str(&row(
        "x86",
        format_integer(stats.x86_instruction_count as u64),
    ));
    output.push_str(&row(
        "x86/rv",
        format!("{:.2}", stats.x86_instructions_per_riscv_instruction()),
    ));
    output.push_str(&row("code", format_bytes(stats.x86_code_bytes as u64)));
    output.push_str(&row("jtable", format_bytes(stats.jump_table_bytes as u64)));
    output.push_str(&row("output", format_bytes(output_size)));
    output
}

fn row(label: &str, value: String) -> String {
    format!("{label:<LABEL_WIDTH$}{value}\n")
}

fn format_duration(duration: std::time::Duration) -> String {
    let ns = duration.as_nanos();
    if ns >= 1_000_000_000 {
        format!("{:.3}s", duration.as_secs_f64())
    } else if ns >= 1_000_000 {
        format!("{:.3}ms", ns as f64 / 1_000_000.0)
    } else if ns >= 1_000 {
        format!("{:.3}us", ns as f64 / 1_000.0)
    } else {
        format!("{ns}ns")
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2}MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2}KiB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes}B")
    }
}

fn format_integer(value: u64) -> String {
    let digits = value.to_string();
    let mut output = String::with_capacity(digits.len() + digits.len() / 3);

    for (idx, ch) in digits.chars().enumerate() {
        if idx > 0 && (digits.len() - idx) % 3 == 0 {
            output.push(',');
        }
        output.push(ch);
    }

    output
}
