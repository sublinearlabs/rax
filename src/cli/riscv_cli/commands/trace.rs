//! RISC-V trace command - Execution trace generation

use crate::cli::common::{check_file_exists, print_header, print_info, CliError, CliResult};
use crate::trace::NoopTracer;
use crate::{init_from_elf, Runner};
use colored::*;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

/// Trace result data
#[derive(Debug)]
pub struct TraceResult {
    pub total_cycles: u64,
    pub exit_code: u64,
    pub elapsed_ms: f64,
}

/// Execute the trace command
pub fn execute_trace(
    binary: &str,
    _filter: Option<&str>,
    format: &str,
    output: Option<&str>,
) -> CliResult<()> {
    print_header("RISC-V CLI - Trace Generation Command");

    // Check file exists
    check_file_exists(binary)?;
    print_info(&format!("Loading ELF: {}", binary));

    // Load ELF and execute with timing
    // TODO: use FullTracer once trace structure is finalized
    let mut vm = init_from_elf::<NoopTracer>(binary);
    print_info("Starting execution analysis...");

    let mut runner = Runner::new();
    let start = Instant::now();
    runner.run(&mut vm);
    let elapsed = start.elapsed();

    // Collect results
    let result = TraceResult {
        total_cycles: runner.cycles(),
        exit_code: vm.exit_code(),
        elapsed_ms: elapsed.as_secs_f64() * 1000.0,
    };

    // Display results
    print_trace_summary(&result)?;

    // Format output as string
    let output_text = format_trace_output(&result, format)?;

    // Display to stdout
    println!("{}", output_text);

    // Save to file if requested
    if let Some(file_path) = output {
        write_trace_to_file(file_path, &output_text)?;
        print_info(&format!("Trace output written to: {}", file_path));
    }

    Ok(())
}

/// Print summary of trace results
fn print_trace_summary(result: &TraceResult) -> CliResult<()> {
    println!("\n{}", "Execution Analysis Summary".bold());
    println!("{}", "-".repeat(60));
    println!("  Exit Code:          {}", result.exit_code);
    println!("  Total Cycles:       {}", result.total_cycles);
    println!("  Elapsed Time:       {:.2} ms", result.elapsed_ms);
    println!(
        "  Cycles/ms:          {:.2}",
        result.total_cycles as f64 / result.elapsed_ms
    );
    println!("{}", "-".repeat(60));
    Ok(())
}

/// Format trace output based on format type
fn format_trace_output(result: &TraceResult, format: &str) -> CliResult<String> {
    match format {
        "text" => format_trace_text(result),
        "json" => format_trace_json(result),
        "csv" => format_trace_csv(result),
        _ => Err(CliError::new(format!(
            "Unknown output format: '{}'. Use: text, json, csv",
            format
        ))),
    }
}

/// Format trace output as human-readable text
fn format_trace_text(_result: &TraceResult) -> CliResult<String> {
    Ok(format!(
        "{}\n{}\n{}",
        "Execution Analysis".bold(),
        "-".repeat(100),
        "Detailed instruction-level tracing available via library FullTracer API"
    ))
}

/// Format trace output as JSON
fn format_trace_json(result: &TraceResult) -> CliResult<String> {
    let json = serde_json::json!({
        "execution_analysis": {
            "exit_code": result.exit_code,
            "total_cycles": result.total_cycles,
            "elapsed_ms": result.elapsed_ms,
        },
        "note": "Full instruction-level tracing is available via the FullTracer in the library API"
    });

    Ok(serde_json::to_string_pretty(&json).unwrap())
}

/// Format trace output as CSV
fn format_trace_csv(result: &TraceResult) -> CliResult<String> {
    Ok(format!(
        "metric,value\nexit_code,{}\ntotal_cycles,{}\nelapsed_ms,{:.2}",
        result.exit_code, result.total_cycles, result.elapsed_ms
    ))
}

/// Write trace output to a file
fn write_trace_to_file(file_path: &str, content: &str) -> CliResult<()> {
    let mut file = File::create(file_path).map_err(|e| {
        CliError::new(format!(
            "Failed to create output file '{}': {}",
            file_path, e
        ))
    })?;

    file.write_all(content.as_bytes())
        .map_err(|e| CliError::new(format!("Failed to write to file '{}': {}", file_path, e)))?;

    Ok(())
}
