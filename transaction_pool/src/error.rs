//! Errors for Transaction Pool
use crate::{traits::PoolTransaction, validate::ValidPoolTransaction};
use primitives::types::TxHash;
use std::sync::Arc;

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
    ImportError,
}

pub enum InsertErr<T: PoolTransaction> {
    Underpriced {
        transaction: Arc<ValidPoolTransaction<T>>,
    },
    InvalidTransaction {
        transaction: Arc<ValidPoolTransaction<T>>,
    },
}

#[derive(Debug)]
pub enum InvalidPoolTransactionError {
    TxTypeNotSupported,
    NotEnoughFee,
    NonceNotConsistent,
}

#[derive(Debug)]
pub enum TransactionValidatoneError {
    ValidationServiceUnreachable,
}
