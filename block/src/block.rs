use core::time;
use std::sync::OnceLock;
use primitives::types::{BlockHash, TxHash, B256};
use sha2::{Sha256, Digest};

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

// For Disk Storage
#[derive(Debug)]
pub struct Block<T, H = Header> {
    pub header: H,
    pub body: BlockBody<T>
}

impl<T, H> Block<T, H> {
    pub const fn new(header: H, body: BlockBody<T>) -> Self {
        Self { header, body } 
    }

    pub fn into_header(self) -> H {
        self.header
    }

    pub fn into_body(self) -> BlockBody<T> {
        self.body
    }
}


// TODO: Implement SealedHeader
// Runtime Memory Cache Structure
pub struct SealedHeader<H = Header> {
    hash: OnceLock<BlockHash>,
    header: H,
}

#[derive(Debug)]
pub struct BlockBody<T> {
    pub transaction: Vec<T>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_block() {
        
    }
}
