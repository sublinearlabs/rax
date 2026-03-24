//! Ethereum CLI binary entry point

use clap::Parser;
use eth_utils::cli::{EthCli, EthCommand};
use riscv::cli::common::{print_error, print_info};
use tracing_subscriber::EnvFilter;

fn main() {
    let cli = EthCli::parse();
    init_tracing(cli.verbose);

    if let Some(ref rpc_url) = cli.rpc_url {
        print_info(&format!("Using RPC endpoint: {}", rpc_url));
    }

    let result = match cli.command {
        EthCommand::Fetch {
            block,
            format,
            output,
        } => {
            print_info(&format!("Fetching block: {}", block));
            eth_utils::cli::commands::execute_fetch(
                &block,
                cli.rpc_url.as_deref(),
                &format,
                output.as_deref(),
            )
        }

        EthCommand::Inspect {
            block,
            detailed,
            format,
        } => {
            print_info(&format!("Inspecting block: {}", block));
            todo!()
        }

        EthCommand::Trace {
            block,
            format,
            output,
        } => {
            print_info(&format!("Tracing block: {}", block));
            todo!()
        }

        EthCommand::Stats {
            range,
            format,
            output,
        } => {
            print_info(&format!("Generating statistics for range: {}", range));
            todo!()
        }
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
