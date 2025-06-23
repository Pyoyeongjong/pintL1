use std::sync::Arc;

use primitives::types::TxHash;

use crate::{traits::PoolTransaction, validate::ValidPoolTransaction};

pub type PoolResult<T> = Result<T, PoolError>;

#[derive(Debug)]
pub struct PoolError {
    pub hash: TxHash,
    pub kind: PoolErrorKind,
}

impl PoolError {
    pub fn new(hash: TxHash, kind: PoolErrorKind) -> Self {
        Self { hash, kind }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PoolErrorKind {
    AlreadyImported,
    InvalidTransaction,
    RelpacementUnderpriced,
}

pub enum InsertErr<T: PoolTransaction> {
    Underpriced {
        transaction: Arc<ValidPoolTransaction<T>>,
    },
    InvalidTransaction {
        transaction: Arc<ValidPoolTransaction<T>>,
    },
}
