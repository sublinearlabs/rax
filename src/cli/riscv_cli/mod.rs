//! RISC-V VM command-line interface
//!
//! Provides commands for executing and analyzing RISC-V ELF binaries

use clap::{Parser, Subcommand};

pub mod commands;

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

    /// Inspect an ELF binary structure
    Inspect {
        /// Path to the ELF binary
        #[arg(value_name = "FILE")]
        binary: String,

        /// Output format (text, json, csv)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Save output to file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Trace instruction execution
    Trace {
        /// Path to the ELF binary
        #[arg(value_name = "FILE")]
        binary: String,

        /// Filter by instruction type
        #[arg(short, long)]
        filter: Option<String>,

        /// Output format (text, json, csv)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Save output to file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Run performance benchmarks
    Benchmark {
        /// Path to the ELF binary
        #[arg(value_name = "FILE")]
        binary: String,

        /// Number of iterations
        #[arg(short, long, default_value = "1")]
        iterations: usize,

        /// Output format (text, json, csv)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Save output to file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Verify Ethereum block execution on RISC-V VM
    VerifyBlock {
        /// Ethereum block number
        #[arg(value_name = "BLOCK")]
        block: String,

        /// Path to RISC-V verifier binary
        #[arg(short, long)]
        binary: String,

        /// Ethereum RPC endpoint
        #[arg(short, long)]
        rpc_url: String,

        /// Path to save the witness file
        #[arg(short, long)]
        witness: Option<String>,

        /// Output format (text, json, csv)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Save output to file
        #[arg(short, long)]
        output: Option<String>,
    },
}
