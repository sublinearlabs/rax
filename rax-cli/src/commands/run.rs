//! RISC-V run command

use crate::common::{check_file_exists, print_header, print_info, CliError, CliResult};
use colored::*;
use rax_interpreter::init_from_elf;
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
pub fn execute_run(binary: &str, jit: bool, format: &str, output: Option<&str>) -> CliResult<()> {
    print_header("RISC-V CLI - Run Command");

    // Check file exists
    check_file_exists(binary)?;
    print_info(&format!("Loading ELF: {}", binary));

    // Load ELF file
    let mut vm = init_from_elf(binary);

    print_info(if jit {
        "Starting JIT execution..."
    } else {
        "Starting interpreter execution..."
    });

    let start = Instant::now();
    let cycles = if jit {
        let mut runner = rax_jit::Runner::new();
        runner.set_input_from_host();
        runner.run(&mut vm);
        runner.cycles()
    } else {
        let mut runner = rax_interpreter::Runner::new();
        runner.set_input_from_host();
        runner.run(&mut vm);
        runner.cycles()
    };
    let elapsed = start.elapsed();

    // Collect results
    let result = ExecutionResult {
        exit_code: vm.exit_code(),
        cycles,
        elapsed_ms: elapsed.as_secs_f64() * 1000.0,
        instructions_per_cycle: cycles as f64 / elapsed.as_micros() as f64,
    };

    // Format output as string
    let output_text = format_output(format, &result)?;

    // Display to stdout
    println!("{}", output_text);

    // Save to file if requested
    if let Some(file_path) = output {
        write_run_to_file(file_path, &output_text)?;
        print_info(&format!("Run output written to: {}", file_path));
    }

    Ok(())
}

/// Format and display execution results
fn format_output(format: &str, result: &ExecutionResult) -> CliResult<String> {
    match format {
        "text" => {
            let mut output = String::new();
            output.push_str(&format!("\n{}\n", "Execution Results".bold()));
            output.push_str(&format!("{}\n", "-".repeat(60)));
            output.push_str(&format!("  Exit Code:            {}\n", result.exit_code));
            output.push_str(&format!("  Total Cycles:         {}\n", result.cycles));
            output.push_str(&format!(
                "  Elapsed Time:         {:.2} ms\n",
                result.elapsed_ms
            ));
            output.push_str(&format!(
                "  Instructions/Cycle:   {:.2} MHz\n",
                result.instructions_per_cycle
            ));
            output.push_str(&format!("{}\n", "-".repeat(60)));
            Ok(output)
        }
        "json" => {
            let json = serde_json::json!({
                "exit_code": result.exit_code,
                "cycles": result.cycles,
                "elapsed_ms": result.elapsed_ms,
                "instructions_per_cycle": result.instructions_per_cycle,
            });
            Ok(serde_json::to_string_pretty(&json).unwrap())
        }
        "csv" => {
            let mut output = String::new();
            output.push_str("exit_code,cycles,elapsed_ms,instructions_per_cycle\n");
            output.push_str(&format!(
                "{},{},{:.2},{:.2}\n",
                result.exit_code, result.cycles, result.elapsed_ms, result.instructions_per_cycle
            ));
            Ok(output)
        }
        _ => Err(CliError::new(format!(
            "Unknown output format: '{}'. Use: text, json, csv",
            format
        ))),
    }
}

/// Write run output to a file
fn write_run_to_file(file_path: &str, content: &str) -> CliResult<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(file_path).map_err(|e| {
        CliError::new(format!(
            "Failed to create output file '{}': {}",
            file_path, e
        ))
    })?;

    file.write_all(content.as_bytes())
        .map_err(|e| CliError::new(format!("Failed to write to file '{}': {}", file_path, e)))?;

    Ok(())
}
