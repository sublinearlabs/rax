//! Production Integration Example: Generate full execution witness from block
//!
//! This example demonstrates the complete pipeline:
//! 1. Fetches real block from Ethereum RPC
//! 2. Traces block execution to generate state changes
//! 3. Builds complete execution witness
//! 4. Serializes and saves for guest validator
//! 5. Runs exec-block test binary with the witness

use riscv::trace::NoopTracer;
use riscv::{
    Runner,
    eth::{BlockTracer, EthFetcher},
    init_from_elf,
};
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() {
    let witness_file_path = Some("examples/exec-live-block.input");
    // let witness_file_path: Option<&str> = None;
    let block_number = 24628522u64;

    match witness_file_path {
        Some(file_path) => {
            println!("Serialized witness available at: {}", file_path);
            println!("Reading serialized witness");

            let input_packet = fs::read(file_path).expect("Failed to read block input");

            let decoded_input =
                hex::decode(input_packet).expect("Failed to decode witness input data");

            execute_light_client_guest(decoded_input);
        }
        None => {
            // Step 1: Fetch real block from Ethereum
            println!("Step 1: Fetching block from Ethereum RPC...");
            let rpc_url = std::env::var("ETH_MAINNET_RPC_URL")
                .unwrap_or_else(|_| "https://eth-mainnet.g.alchemy.com/v2/{YOUR_KEY}".to_string());

            let fetcher = match EthFetcher::new(rpc_url.as_str()) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("⚠ Warning: Could not connect to RPC: {}", e);
                    eprintln!("  Using synthetic block data instead");
                    println!(
                        "  (To use real data, set ETH_MAINNET_RPC_URL environment variable)\n"
                    );

                    return generate_synthetic_witness_and_save().await;
                }
            };

            match fetcher.fetch_block_data(block_number).await {
                Ok(block_data) => {
                    println!("✓ Block #{} fetched successfully", block_number);
                    println!("  State root: {}", hex::encode(block_data.state_root));
                    println!("  Transactions: {}\n", block_data.transactions.len());

                    // Step 2: Generate execution trace
                    println!("Step 2: Generating execution trace...");
                    let block_trace = match BlockTracer::trace_block(&block_data) {
                        Ok(trace) => {
                            println!("✓ Trace generated successfully");
                            println!("  Transactions traced: {}\n", trace.transactions.len());
                            trace
                        }
                        Err(e) => {
                            eprintln!("Error generating trace: {}", e);
                            eprintln!("  Using synthetic block data instead");
                            return generate_synthetic_witness_and_save().await;
                        }
                    };

                    // Step 3: Generate execution witness
                    println!("Step 3: Generating execution witness structure...");
                    let witness_json =
                        match BlockTracer::generate_witness(&block_data, &block_trace) {
                            Ok(w) => {
                                println!("✓ Witness structure generated");
                                w
                            }
                            Err(e) => {
                                eprintln!("Error generating witness: {}", e);
                                eprintln!("  Using synthetic block data instead");
                                return generate_synthetic_witness_and_save().await;
                            }
                        };

                    // Step 4: Serialize to JSON
                    println!("Step 4: Serializing witness to JSON...");
                    let witness_bytes = serde_json::to_vec_pretty(&witness_json)
                        .expect("Failed to serialize witness");
                    println!("✓ Witness serialized: {} bytes\n", witness_bytes.len());

                    // Step 5: Save to file
                    let state_root_array: [u8; 32] = block_data.state_root.into();

                    // Step 6: Save wwitness to file (hex-encoded for exec-block)
                    save_witness(
                        &witness_bytes,
                        block_number,
                        state_root_array,
                        "examples/exec-live-block.input",
                    )
                    .await;

                    let input_packet = fs::read("examples/exec-live-block.input")
                        .expect("Failed to read block input");

                    let decoded_input =
                        hex::decode(input_packet).expect("Failed to decode witness input data");

                    // Step 7: Run exec-block test binary with the witness
                    execute_light_client_guest(decoded_input);
                }
                Err(e) => {
                    eprintln!("Error fetching block: {}", e);
                    eprintln!("Falling back to synthetic data...\n");
                    generate_synthetic_witness_and_save().await;
                }
            }
        }
    }
}

async fn generate_synthetic_witness_and_save() {
    println!("Step 1: Generating synthetic witness...");
    let block_number = 19000000u64;
    let state_root =
        hex::decode("81dce89407da8ea6049dee8443b625fa08cf1edd036b7e77372b8fbda7ae763a")
            .expect("Failed to decode state root")
            .try_into()
            .expect("State root must be 32 bytes");

    let witness_data = serde_json::json!({
        "block": {
            "number": block_number,
            "state_root": "0x81dce89407da8ea6049dee8443b625fa08cf1edd036b7e77372b8fbda7ae763a",
        },
        "accounts": {},
        "witness": {
            "codes": [],
            "state": [],
            "keys": [],
        }
    });

    let witness_bytes =
        serde_json::to_vec_pretty(&witness_data).expect("Failed to serialize synthetic witness");
    println!("✓ Synthetic witness: {} bytes\n", witness_bytes.len());

    save_witness(
        &witness_bytes,
        block_number,
        state_root,
        "examples/exec-live-block.input",
    )
    .await;
}

async fn save_witness(
    witness_bytes: &[u8],
    block_number: u64,
    state_root: [u8; 32],
    file_path: &str,
) {
    // The witness_bytes should be JSON from the block trace
    // But exec-block expects it as the direct input
    let mut input_packet = Vec::new();
    input_packet.extend_from_slice(witness_bytes);

    println!("✓ Prepared input packet");
    println!("  Block number: {}", block_number);
    println!("  State root: {}", hex::encode(&state_root));
    println!("  Witness data length: {} bytes", witness_bytes.len());
    println!("  Total packet size: {} bytes\n", input_packet.len());

    let packet_hex = hex::encode(&input_packet);
    fs::write(file_path, &packet_hex).expect("Failed to write witness to exec-live-block.input");

    println!("Step 6: Saved witness packet to 'examples/exec-live-block.input'");
    println!("  Hex length: {} chars\n", packet_hex.len());
}

pub fn execute_light_client_guest(input_packet: Vec<u8>) {
    println!("Step 7: Running exec-block binary with generated witness...");

    const EXEC_BLOCK_BINARY: &str = "test-bin/rust-bin/exec-block/exec-block-imac";

    if !Path::new(EXEC_BLOCK_BINARY).exists() {
        println!("⚠ exec-block binary not found at {}", EXEC_BLOCK_BINARY);
        println!("  Skipping execution test");
        println!("  To run: cargo run --example exec_block_imac --release\n");
    } else {
        println!("  Loading: {}", EXEC_BLOCK_BINARY);

        let mut vm = init_from_elf::<NoopTracer>(EXEC_BLOCK_BINARY.to_string());
        let mut runner = Runner::new();

        runner.set_input_stream(input_packet);

        println!("  Running...\n");
        runner.run_with_timing(&mut vm);

        println!("\n  Exit code: {}", vm.exit_code());
    };
}
