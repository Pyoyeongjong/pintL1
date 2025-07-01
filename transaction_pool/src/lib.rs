//! Implementation of the Transaction Pool [Pool]
//! [Pool] is the top level structure for transaction pool
//! It manages mempool & validation part.
use std::sync::Arc;

use primitives::types::TxHash;

use crate::{
    config::PoolConfig,
    error::PoolResult,
    ordering::TransactionOrdering,
    pool::PoolInner,
    traits::{PoolTransaction, TransactionOrigin, TransactionPool},
    validate::{TransactionValidationOutcome, TransactionValidator},
};

mod config;
mod error;
mod identifier;
mod ordering;
mod pool;
mod test_utils;
mod traits;
mod validate;

pub struct Pool<V, T: TransactionOrdering> {
    pool: Arc<PoolInner<V, T>>,
}

impl<V, T> Pool<V, T>
where
    V: TransactionValidator,
    T: TransactionOrdering,
{
    pub fn new(validator: V, ordering: T, config: PoolConfig) -> Self {
        Self {
            pool: Arc::new(PoolInner::new(validator, ordering, config)),
        }
    }

    async fn validate(
        &self,
        origin: TransactionOrigin,
        transaction: V::Transaction,
    ) -> (TxHash, TransactionValidationOutcome<V::Transaction>) {
        let hash = transaction.hash().clone();
        let outcome = self
            .pool
            .validator()
            .validate_transaction(origin, transaction)
            .await;

        (hash, outcome)
    }
}

impl<V, T> TransactionPool for Pool<V, T>
where
    V: TransactionValidator,
    <V as TransactionValidator>::Transaction: PoolTransaction,
    T: TransactionOrdering<Transaction = <V as TransactionValidator>::Transaction> + Send + Sync,
{
    type Transaction = T::Transaction;

    // In trait, no async but implementation trait, async is sugar!
    async fn add_transaction(
        &self,
        origin: TransactionOrigin,
        transaction: Self::Transaction,
    ) -> PoolResult<TxHash> {
        todo!()
    }
}
