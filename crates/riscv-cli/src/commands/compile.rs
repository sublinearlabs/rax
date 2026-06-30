//! RISC-V compile command

use crate::common::{check_file_exists, print_info, print_success, CliResult};
use riscv::aot::compiler::compile_elf_file;

/// Execute the compile command
pub fn execute_compile(input_path: &str, output_path: &str) -> CliResult<()> {
    check_file_exists(input_path)?;
    print_info(&format!("Compiling {} to {}", input_path, output_path));

    compile_elf_file(input_path, output_path)?;

    print_success(&format!("Output written to {}", output_path));
    Ok(())
}
