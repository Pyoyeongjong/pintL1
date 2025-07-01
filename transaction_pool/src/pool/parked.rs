//! Implementation of [ParkedPool] and [ParkedPoolTransaction]
use std::{collections::BTreeMap, sync::Arc};

use crate::{
    identifier::TransactionId, ordering::TransactionOrdering, validate::ValidPoolTransaction,
};

/// PintL1 ParkedPool: It should be same as queue_pool!
pub struct ParkedPool<T: TransactionOrdering> {
    // Keeps track of when transaction was inserted in this pool by id
    submission_id: u64,
    // All Transactions that are currently inside the pool grouped by their
    // identifier.
    by_id: BTreeMap<TransactionId, ParkedPoolTransaction<T>>,
}

impl<T: TransactionOrdering> ParkedPool<T> {
    pub fn add_transaction(&mut self, tx: Arc<ValidPoolTransaction<T::Transaction>>) {
        let id = *tx.id();
        let submission_id = self.next_id();
        let tx = ParkedPoolTransaction {
            submission_id,
            transaction: tx,
        };

        self.by_id.insert(id, tx);
    }

    pub fn remove_transaction(
        &mut self,
        id: &TransactionId,
    ) -> Option<Arc<ValidPoolTransaction<<T as TransactionOrdering>::Transaction>>> {
        let tx = self.by_id.remove(id)?;
        Some(tx.transaction)
    }

    const fn next_id(&mut self) -> u64 {
        let id = self.submission_id;
        self.submission_id = self.submission_id.wrapping_add(1);
        id
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }
}

impl<T: TransactionOrdering> Default for ParkedPool<T> {
    fn default() -> Self {
        Self {
            submission_id: 0,
            by_id: Default::default(),
        }
    }
}

/// ParkedPoolTransaction
struct ParkedPoolTransaction<T: TransactionOrdering> {
    transaction: Arc<ValidPoolTransaction<T::Transaction>>,
    submission_id: u64,
}
