use std::sync::OnceLock;

use crate::types::{BlockHash, TxHash};
use alloy_primitives::B256;
use k256::sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct Header {
    pub previous_hash: TxHash,
    pub transaction_root: B256,
    pub state_root: B256,
    pub timestamp: u64,
}

impl Header {
    pub fn hash_slow(&self) -> BlockHash {
        let mut hasher = Sha256::new();
        hasher.update(self.previous_hash.to_string().as_bytes());
        hasher.update(self.transaction_root.to_string().as_bytes());
        hasher.update(self.state_root.to_string().as_bytes());
        hasher.update(self.timestamp.to_string().as_bytes());
        B256::from_slice(&hasher.finalize())
    }
}

impl crate::block::traits::BlockHeader for Header {}

// TODO: Implement SealedHeader
// Runtime Memory Cache Structure for block header with block hash
#[derive(Clone)]
pub struct SealedHeader<H = Header> {
    hash: OnceLock<BlockHash>,
    header: H,
}

impl SealedHeader {
    pub fn hash(&self) -> BlockHash {
        *self.hash.get_or_init(|| self.header.hash_slow())
    }
}
