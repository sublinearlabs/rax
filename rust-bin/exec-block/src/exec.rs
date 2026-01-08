use alloc::{format, sync::Arc, vec::Vec};
use reth_chainspec::ChainSpec;
use reth_evm_ethereum::EthEvmConfig;
use reth_primitives_traits::Block;
use reth_stateless::{
    stateless_validation_with_trie, Genesis, StatelessInput, UncompressedPublicKey,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sparsestate::SparseState;

use crate::{syscalls::sys_println, utils::to_reth_stateless_input};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RethStatelessValidatorInput {
    /// The stateless input for the stateless validation function.
    pub stateless_input: StatelessInput,
    /// The recovered signers for the transactions in the block.
    pub public_keys: Vec<UncompressedPublicKey>,
}
pub type RethStatelessValidatorOutput = ([u8; 32], [u8; 32], bool);

pub fn runner(input_raw: &[u8]) -> [u8; 32] {
    sys_println(&format!("Input have been achieved"));
    let stateless_input = serde_json::from_slice::<StatelessInput>(input_raw).unwrap();
    sys_println(&format!("Deserialization has been done"));
    
    let input = to_reth_stateless_input(stateless_input);
    sys_println(&format!("RETH stateless input (into)"));
    
    let genesis = Genesis {
        config: input.stateless_input.chain_config.clone(),
        ..Default::default()
    };
    let chain_spec: Arc<ChainSpec> = Arc::new(genesis.into());
    let evm_config = EthEvmConfig::new(chain_spec.clone());
    
    sys_println(&format!("Chain config obtained"));
    

    let header = input.stateless_input.block.header().clone();
    let parent_hash = input.stateless_input.block.parent_hash;
    
    sys_println(&format!("Starting stateless validation"));

    let res = stateless_validation_with_trie::<SparseState, _, _>(
        input.stateless_input.block,
        input.public_keys,
        input.stateless_input.witness,
        chain_spec,
        evm_config,
    )
    .map(|(block_hash, _)| block_hash);
    
    sys_println(&format!("Done with block execution"));

    let output: RethStatelessValidatorOutput = match res {
        Ok(block_hash) => (block_hash.0, parent_hash.0, true),
        Err(_err) => (header.hash_slow().0, parent_hash.0, false),
    };
    
    sys_println(&format!("Output have been prepared"));

    let output_serialized = serde_json::to_vec(&output).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&output_serialized);
    let output_hash: [u8; 32] = hasher.finalize().into();
    
    let out_string = format!("VM output: {:?}", output_hash);
    sys_println(&out_string);

    output_hash
}
