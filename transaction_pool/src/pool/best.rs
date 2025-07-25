use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    sync::Arc,
};

use crate::{
    identifier::{SenderId, TransactionId},
    ordering::TransactionOrdering,
    pool::pending::PendingTransaction,
    validate::ValidPoolTransaction,
};

/// Iterator that returns transactions that can be executed on the current state (best)
pub struct BestTransactions<T: TransactionOrdering> {
    // Contains a copy of all transaction in time this was created
    pub(crate) all: BTreeMap<TransactionId, PendingTransaction<T>>,
    // Transactions that can be executed right away
    pub(crate) independent: BTreeSet<PendingTransaction<T>>,
    // Invalid Transactions
    pub(crate) invalid: HashSet<SenderId>,
}

impl<T: TransactionOrdering> BestTransactions<T> {
    fn pop_best(&mut self) -> Option<PendingTransaction<T>> {
        let res = self.independent.pop_last().inspect(|best| {
            self.all.remove(best.transaction.id());
        });
        res
    }
}

impl<T: TransactionOrdering> crate::traits::BestTransactions for BestTransactions<T> {}

impl<T: TransactionOrdering> Iterator for BestTransactions<T> {
    type Item = Arc<ValidPoolTransaction<T::Transaction>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let best = match self.pop_best() {
                Some(best) => best,
                None => {
                    return None;
                }
            };
            let sender_id = best.transaction.sender_id();

            if self.invalid.contains(&sender_id) {
                continue;
            }

            return Some(best.transaction.clone());
        }
    }
}
