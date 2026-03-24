//! RISC-V run command

use crate::cli::common::{check_file_exists, print_header, print_info, CliError, CliResult};
use crate::{init_from_elf, trace::NoopTracer, Runner, VM};
use colored::*;
use std::time::Instant;

/// Execution result data
#[derive(Debug)]
pub struct ExecutionResult {
    pub exit_code: u64,
    pub cycles: u64,
    pub elapsed_ms: f64,
    pub instructions_per_cycle: f64,
}

/// Execute the run command
pub fn execute_run(
    binary: &str,
    _trace: bool,
    format: &str,
    _output: Option<&str>,
) -> CliResult<()> {
    print_header("RISC-V CLI - Run Command");

    // Check file exists
    check_file_exists(binary)?;
    print_info(&format!("Loading ELF: {}", binary));

    // Load ELF file
    let mut vm: VM<NoopTracer> = init_from_elf(binary);

    // Create runner and execute
    print_info("Starting execution...");
    let mut runner = Runner::new();
    let start = Instant::now();
    runner.run(&mut vm);
    let elapsed = start.elapsed();

    // Collect results
    let result = ExecutionResult {
        exit_code: vm.exit_code(),
        cycles: runner.cycles(),
        elapsed_ms: elapsed.as_secs_f64() * 1000.0,
        instructions_per_cycle: runner.cycles() as f64 / elapsed.as_micros() as f64,
    };

    // Display results based on format
    format_output(format, &result)?;

    Ok(())
}

/// Format and display execution results
fn format_output(format: &str, result: &ExecutionResult) -> CliResult<()> {
    match format {
        "text" => {
            println!("\n{}", "Execution Results".bold());
            println!("{}", "-".repeat(60));
            println!("  Exit Code:            {}", result.exit_code);
            println!("  Total Cycles:         {}", result.cycles);
            println!("  Elapsed Time:         {:.2} ms", result.elapsed_ms);
            println!(
                "  Instructions/Cycle:   {:.2} MHz",
                result.instructions_per_cycle
            );
            println!("{}", "-".repeat(60));
        }
        "json" => {
            let json = serde_json::json!({
                "exit_code": result.exit_code,
                "cycles": result.cycles,
                "elapsed_ms": result.elapsed_ms,
                "instructions_per_cycle": result.instructions_per_cycle,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        "csv" => {
            println!("exit_code,cycles,elapsed_ms,instructions_per_cycle");
            println!(
                "{},{},{:.2},{:.2}",
                result.exit_code, result.cycles, result.elapsed_ms, result.instructions_per_cycle
            );
        }
        _ => {
            return Err(CliError::new(format!(
                "Unknown output format: '{}'. Use: text, json, csv",
                format
            )));
        }
    }
    Ok(())
}
