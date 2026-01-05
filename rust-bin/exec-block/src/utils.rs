use alloc::vec::Vec;
use reth_stateless::{StatelessInput, UncompressedPublicKey};
use reth_ethereum_primitives::TransactionSigned;

use crate::exec::RethStatelessValidatorInput;

pub fn to_reth_stateless_input(s_inout: &StatelessInput) -> RethStatelessValidatorInput {
    let signers = recover_signers(&s_inout.block.body.transactions).unwrap();
    RethStatelessValidatorInput {
        stateless_input: s_inout.clone(),
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