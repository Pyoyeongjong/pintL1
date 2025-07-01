//! [PoolInner] implementation
use parking_lot::{RwLock, RwLockWriteGuard};
use primitives::types::TxHash;
use std::sync::Arc;

use crate::{
    config::PoolConfig,
    error::PoolResult,
    ordering::TransactionOrdering,
    pool::txpool::TxPool,
    traits::{PoolTransaction, TransactionOrigin},
    validate::{TransactionValidationOutcome, TransactionValidator, ValidPoolTransaction},
};

pub mod parked;
pub mod pending;
pub mod state;
pub mod txpool;

/// PoolInner
pub struct PoolInner<V, T>
where
    T: TransactionOrdering,
{
    validator: V,
    pool: RwLock<TxPool<T>>,
}

impl<V, T> PoolInner<V, T>
where
    V: TransactionValidator,
    T: TransactionOrdering,
{
    pub fn new(validator: V, ordering: T, config: PoolConfig) -> Self {
        Self {
            validator: validator,
            pool: RwLock::new(TxPool::new(ordering, config)),
        }
    }

    pub fn validator(&self) -> &V {
        &self.validator
    }

    // Adds all transactions in the iterator to the pool, returning a list of results.
    pub fn add_transactions(
        &self,
        origin: TransactionOrigin,
        transactions: impl IntoIterator<Item = TransactionValidationOutcome<T::Transaction>>,
    ) -> Vec<PoolResult<TxHash>> {
        // Add the transactions

        let mut pool = self.pool.write();
        let added = transactions
            .into_iter()
            .map(|tx| self.add_transaction(&mut pool, origin, tx))
            .collect();

        added
    }

    fn add_transaction(
        &self,
        pool: &mut RwLockWriteGuard<'_, TxPool<T>>,
        origin: TransactionOrigin,
        tx: TransactionValidationOutcome<T::Transaction>,
    ) -> PoolResult<TxHash> {
        todo!()
    }
}

// Represents a transaction that was added into the pool and its state
#[derive(Debug)]
pub enum AddedTransaction<T: PoolTransaction> {
    // Transaction was successfully added and moved to the pending pool
    Pending(AddedPendingTransaction<T>),
    // Successfully added but not yet ready for processing -> so it moved to the parked pool
    Parked {
        transaction: Arc<ValidPoolTransaction<T>>,
    },
}

// Tracks an added transaction
#[derive(Debug)]
pub struct AddedPendingTransaction<T: PoolTransaction> {
    transaction: Arc<ValidPoolTransaction<T>>,
}
