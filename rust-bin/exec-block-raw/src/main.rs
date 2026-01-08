use std::convert::TryInto;
use std::fs;
use std::sync::Arc;

use reth_chainspec::ChainSpec;
use reth_evm_ethereum::EthEvmConfig;
use reth_primitives_traits::Block;
use reth_stateless::{
    Genesis, StatelessInput, UncompressedPublicKey, stateless_validation_with_trie,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sparsestate::SparseState;

use anyhow::{Context, Result};
use reth_ethereum_primitives::TransactionSigned;


/// Input structure used by the local runner (same shape as in exec.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RethStatelessValidatorInput {
    /// The stateless input for the stateless validation function.
    pub stateless_input: StatelessInput,
    /// The recovered signers for the transactions in the block.
    pub public_keys: Vec<UncompressedPublicKey>,
}
pub type RethStatelessValidatorOutput = ([u8; 32], [u8; 32], bool);

fn to_reth_stateless_input(s_input: StatelessInput) -> RethStatelessValidatorInput {
    let signers = recover_signers(&s_inout.block.body.transactions).unwrap();
    RethStatelessValidatorInput {
        stateless_input: s_input,
        public_keys: signers,
    }
}

/// Recover public keys from transaction signatures.
fn recover_signers<'a, I>(txs: I) -> anyhow::Result<Vec<UncompressedPublicKey>>
where
    I: IntoIterator<Item = &'a TransactionSigned>,
{
    txs.into_iter()
        .enumerate()
        .map(|(i, tx)| {
            tx.signature()
                .recover_from_prehash(&tx.signature_hash())
                .map(|keys| {
                    UncompressedPublicKey(
                        TryInto::<[u8; 65]>::try_into(keys.to_encoded_point(false).as_bytes())
                            .unwrap(),
                    )
                })
                .map_err(|e| anyhow::anyhow!("failed to recover signature for tx #{i}: {e}"))
        })
        .collect::<Result<Vec<UncompressedPublicKey>, _>>()
}

fn runner(input_raw: &[u8]) -> [u8; 32] {
    println!("Input received ({} bytes)", input_raw.len());
    let stateless_input = serde_json::from_slice::<StatelessInput>(input_raw)
        .expect("failed to deserialize StatelessInput");
    println!("Deserialization done");

    let input = to_reth_stateless_input(stateless_input);
    println!("Converted to Reth stateless input");

    let genesis = Genesis {
        config: input.stateless_input.chain_config.clone(),
        ..Default::default()
    };
    let chain_spec: Arc<ChainSpec> = Arc::new(genesis.into());
    let evm_config = EthEvmConfig::new(chain_spec.clone());

    println!("Chain config obtained");

    let header = input.stateless_input.block.header().clone();
    let parent_hash = input.stateless_input.block.parent_hash;

    println!("Starting stateless validation");

    let res = stateless_validation_with_trie::<SparseState, _, _>(
        input.stateless_input.block,
        input.public_keys,
        input.stateless_input.witness,
        chain_spec,
        evm_config,
    )
    .map(|(block_hash, _)| block_hash);

    println!("Done with block execution");

    let output: RethStatelessValidatorOutput = match res {
        Ok(block_hash) => (block_hash.0, parent_hash.0, true),
        Err(_err) => (header.hash_slow().0, parent_hash.0, false),
    };

    println!("Output prepared");

    let output_serialized = serde_json::to_vec(&output).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&output_serialized);
    let output_hash: [u8; 32] = hasher.finalize().into();

    println!("VM output hash: {:?}", output_hash);

    output_hash
}

fn main() -> Result<()> {
    // First CLI arg is the input file path. If absent, use the default exec-block.input next to the crate.
    let input_path = "exec-block.input";

    println!("Reading input file: {}", input_path);
    let input = fs::read_to_string(input_path)
        .with_context(|| format!("failed to read input file '{}'", input_path))?;
    let input = input.trim();
    let input_bytes = hex::decode(input).unwrap();

    if input_bytes.is_empty() {
        println!("Input file '{}' is empty", input_path);
        return Ok(());
    }

    let _out = runner(&input_bytes);
    
    Ok(())
}
