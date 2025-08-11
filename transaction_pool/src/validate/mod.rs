//! Implement Validation part.
//! Only validated transactions can be inserted into the [TxPool](crate::pool::txpool)
pub mod pint;
pub mod task;

use primitives::types::TxHash;
use transaction::U256;

use crate::{
    error::InvalidPoolTransactionError,
    identifier::{SenderId, TransactionId},
    traits::{PoolTransaction, TransactionOrigin},
};

// A valid transaction in the pool.
// ValidPoolTransaction<T>: a verified transaction
#[derive(Debug, Clone)]
pub struct ValidPoolTransaction<T: PoolTransaction> {
    pub transaction: T,
    pub transaction_id: TransactionId,
    pub origin: TransactionOrigin,
    pub timestamp: std::time::Instant,
}

impl<T: PoolTransaction> ValidPoolTransaction<T> {
    pub fn id(&self) -> &TransactionId {
        &self.transaction_id
    }

    pub fn sender_id(&self) -> SenderId {
        self.transaction_id.sender
    }

    pub fn hash(&self) -> TxHash {
        self.transaction.hash()
    }

    pub fn nonce(&self) -> u64 {
        self.transaction.nonce()
    }

    // True when other is underpriced
    pub fn is_underpriced(&self, other: &Self) -> bool {
        self.transaction.cost() > other.transaction.cost()
    }

    pub fn cost(&self) -> U256 {
        self.transaction.cost()
    }
}

/// Transaction Validator
pub trait TransactionValidator: Send + Sync {
    type Transaction: PoolTransaction;

    fn validate_transaction(
        &self,
        origin: TransactionOrigin,
        transaction: Self::Transaction,
    ) -> impl Future<Output = TransactionValidationOutcome<Self::Transaction>> + Send;
}

/// Transaction Validator Outcome
#[derive(Debug)]
pub enum TransactionValidationOutcome<T: PoolTransaction> {
    Valid {
        transaction: T,
        balance: U256,
        nonce: u64,
    },
    Invalid(T, InvalidPoolTransactionError),
    Error(TxHash, Box<dyn core::error::Error + Send + Sync>),
}

impl<T: PoolTransaction> TransactionValidationOutcome<T> {
    pub const fn is_valid(&self) -> bool {
        matches!(self, Self::Valid { .. })
    }
}
