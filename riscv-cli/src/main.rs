//! RISC-V CLI binary entry point

use clap::Parser;
use common::{print_error, print_info};
use riscv_cli::{RiscvCli, RiscvCommand};
use tracing_subscriber::EnvFilter;

mod commands;
mod common;
mod riscv_cli;

fn main() {
    let cli = RiscvCli::parse();
    init_tracing(cli.verbose);

    let result = match cli.command {
        RiscvCommand::Run {
            jit,
            binary,
            format,
            output,
        } => {
            print_info(&format!("Running RISC-V binary: {}", binary));
            commands::execute_run(&binary, jit, &format, output.as_deref())
        }
        RiscvCommand::Compile {
            input_path,
            output_path,
        } => commands::execute_compile(&input_path, &output_path),
    };

    match result {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            print_error(&e.to_string());
            std::process::exit(1);
        }
    }
}

fn init_tracing(verbosity: u8) {
    let filter = match verbosity {
        0 => "off",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.parse().unwrap()),
        )
        .try_init();
}
