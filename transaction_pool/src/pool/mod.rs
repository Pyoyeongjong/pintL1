//! [PoolInner] implementation
use parking_lot::{RwLock, RwLockWriteGuard};
use primitives::types::{Address, TxHash};
use std::{sync::Arc, time::Instant};

use crate::{
    config::PoolConfig,
    error::{PoolError, PoolResult},
    identifier::{SenderId, SenderIdentifiers, TransactionId},
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
    identifiers: RwLock<SenderIdentifiers>,
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
            identifiers: Default::default(),
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

    pub fn get_sender_id(&self, addr: Address) -> SenderId {
        self.identifiers.write().sender_id_or_create(addr)
    }

    fn add_transaction(
        &self,
        pool: &mut RwLockWriteGuard<'_, TxPool<T>>,
        origin: TransactionOrigin,
        tx: TransactionValidationOutcome<T::Transaction>,
    ) -> PoolResult<TxHash> {
        match tx {
            TransactionValidationOutcome::Valid {
                transaction,
                balance,
                nonce,
                propagate,
            } => {
                let sender_id = self.get_sender_id(transaction.sender());
                let transaction_id: TransactionId = TransactionId::new(sender_id, nonce);
                let tx = ValidPoolTransaction {
                    transaction,
                    transaction_id,
                    origin,
                    timestamp: Instant::now(),
                };

                let added = pool.add_transaction(tx, balance, nonce)?;
                Ok(added.hash())
            }
            TransactionValidationOutcome::Invalid(tx, _) => {
                let pool_error = PoolError {
                    hash: tx.hash(),
                    kind: crate::error::PoolErrorKind::InvalidTransaction,
                };
                return Err(pool_error);
            }
            TransactionValidationOutcome::Error(tx_hash, _) => {
                let pool_error = PoolError {
                    hash: tx_hash,
                    kind: crate::error::PoolErrorKind::ImportError,
                };
                return Err(pool_error);
            }
        }
    }

    pub fn get(&self, tx_hash: &TxHash) -> Option<Arc<ValidPoolTransaction<T::Transaction>>> {
        self.get_pool_data().get(tx_hash)
    }

    fn get_pool_data(
        &self,
    ) -> parking_lot::lock_api::RwLockReadGuard<'_, parking_lot::RawRwLock, TxPool<T>> {
        self.pool.read()
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

impl<T: PoolTransaction> AddedTransaction<T> {
    pub fn hash(&self) -> TxHash {
        match self {
            Self::Pending(tx) => tx.transaction.hash(),
            Self::Parked { transaction, .. } => transaction.hash(),
        }
    }
}

// Tracks an added transaction
#[derive(Debug)]
pub struct AddedPendingTransaction<T: PoolTransaction> {
    transaction: Arc<ValidPoolTransaction<T>>,
}
