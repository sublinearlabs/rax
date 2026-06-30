//! RISC-V VM command-line interface
//!
//! Provides commands for executing and analyzing RISC-V ELF binaries

use clap::{Parser, Subcommand};

/// RISC-V CLI tool for executing and analyzing RISC-V binaries
#[derive(Parser, Debug)]
#[command(name = "riscv-cli")]
#[command(about = "RISC-V VM executor and analyzer", long_about = None)]
#[command(version)]
pub struct RiscvCli {
    #[command(subcommand)]
    pub command: RiscvCommand,

    /// Verbosity level (0=quiet, 1=normal, 2=verbose, 3=debug)
    #[arg(global = true, short = 'v', long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Subcommand, Debug)]
pub enum RiscvCommand {
    /// Run a RISC-V ELF binary
    Run {
        /// Path to the ELF binary
        #[arg(value_name = "FILE")]
        binary: String,

        /// Enable instruction tracing
        #[arg(short, long)]
        trace: bool,

        /// Output format (text, json, csv)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Save output to file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Compile a RISC-V ELF to a native x86-64 executable
    Compile {
        /// Path to the input RISC-V ELF
        #[arg(value_name = "input")]
        input_path: String,

        /// Path to write the output x86-64 ELF
        #[arg(value_name = "output")]
        output_path: String,
    },
}
