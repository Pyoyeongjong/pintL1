use primitives::types::TxHash;

use crate::{identifier::{SenderId, TransactionId}, traits::{PoolTransaction, TransactionOrigin}};

// A valid transaction in the pool.
#[derive(Debug, Clone)]
pub struct ValidPoolTransaction<T: PoolTransaction> {
    pub transaction: T,
    pub transaction_id: TransactionId,
    pub origin: TransactionOrigin,
    pub timestamp: std::time::Instant
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
}
