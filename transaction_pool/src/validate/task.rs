use std::sync::Arc;

use primitives::types::TxHash;
use tokio::sync::{self, oneshot};

use crate::{
    error::TransactionValidatoneError,
    traits::PoolTransaction,
    validate::{TransactionValidationOutcome, TransactionValidator},
};

pub struct TransactionValidationTaskExecutor<V> {
    pub validator: V,
}

impl<V: TransactionValidator> TransactionValidationTaskExecutor<V> {
    pub fn new(validator: V) -> Self {
        Self { validator }
    }
}

impl<V> TransactionValidator for TransactionValidationTaskExecutor<V>
where
    V: TransactionValidator + Clone,
{
    type Transaction = <V as TransactionValidator>::Transaction;

    async fn validate_transaction(
        &self,
        origin: crate::traits::TransactionOrigin,
        transaction: Self::Transaction,
    ) -> TransactionValidationOutcome<Self::Transaction> {
        {
            let res = self
                .validator
                .validate_transaction(origin, transaction)
                .await;
            res
        }
    }
}
