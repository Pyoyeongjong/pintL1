use std::{collections::BTreeMap, sync::Arc};

use crate::{identifier::TransactionId, ordering::TransactionOrdering, traits::PoolTransaction, validate::ValidPoolTransaction};


struct ParkedPoolTransaction<T: TransactionOrdering> {
    transaction: T,
    submission_id: u64
}
pub struct ParkedPool<T: TransactionOrdering> {
    // Keeps track of when transaction was inserted in this pool by id
    submission_id: u64,
    // All Transactions that are currently inside the pool grouped by their
    // identifier.
    by_id: BTreeMap<TransactionId, ParkedPoolTransaction<T>>,
}

impl<T: TransactionOrdering> ParkedPool<T> {

    pub fn add_transaction(&mut self, tx: Arc<ValidPoolTransaction<T::Transaction>>) {
        compile_error!("Not implemented yet");
    }

    pub fn remove_transaction(
        &mut self,
        id: &TransactionId
    ){
        compile_error!("Not implemented yet");
    }
}

impl<T: TransactionOrdering> Default for ParkedPool<T> {
    fn default() -> Self {
        Self {
            submission_id: 0,
            by_id: Default::default()
        }
    }
}