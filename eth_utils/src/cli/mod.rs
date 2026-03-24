//! Ethereum command-line interface
//!
//! Provides commands for fetching and analyzing Ethereum block data

use clap::{Parser, Subcommand};

pub mod commands;

/// Ethereum CLI tool for block data fetching and analysis
#[derive(Parser, Debug)]
#[command(name = "eth-cli")]
#[command(about = "Ethereum block fetcher and analyzer", long_about = None)]
#[command(version)]
pub struct EthCli {
    #[command(subcommand)]
    pub command: EthCommand,

    /// Ethereum RPC endpoint
    #[arg(global = true, long)]
    pub rpc_url: Option<String>,

    /// Verbosity level (0=quiet, 1=normal, 2=verbose, 3=debug)
    #[arg(global = true, short = 'v', long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Subcommand, Debug)]
pub enum EthCommand {
    /// Fetch Ethereum block data
    Fetch {
        /// Block number or hash
        #[arg(value_name = "BLOCK")]
        block: String,

        /// Output format (text, json, csv)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Save output to file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Inspect block details
    Inspect {
        /// Block number or hash
        #[arg(value_name = "BLOCK")]
        block: String,

        /// Output format (text, json, csv)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Show detailed transaction information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Trace block execution
    Trace {
        /// Block number or hash
        #[arg(value_name = "BLOCK")]
        block: String,

        /// Output format (text, json, csv)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Save output to file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Generate block statistics
    Stats {
        /// Block range (e.g., "100-200" or just "100")
        #[arg(value_name = "RANGE")]
        range: String,

        /// Output format (text, json, csv)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Save output to file
        #[arg(short, long)]
        output: Option<String>,
    },
}
