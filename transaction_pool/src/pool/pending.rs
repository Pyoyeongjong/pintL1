//! Implementation of [PendingPool] and [PendingPoolTransaction]
use std::cmp::Ordering;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::broadcast;

use crate::identifier::{SenderId, TransactionId};
use crate::ordering::Priority;
use crate::pool::best::BestTransactions;
use crate::{ordering::TransactionOrdering, validate::ValidPoolTransaction};
#[derive(Clone)]
pub struct PendingPool<T: TransactionOrdering> {
    // How to order transactions.
    ordering: T,
    // Keeps track of when transaction was inserted in this pool by id
    submission_id: u64,
    // All Transactions that are currently inside the pool grouped by their
    // identifier.
    by_id: BTreeMap<TransactionId, PendingTransaction<T>>,
    // Independent transactions that can be included directly and don't require other transactions.
    independent: BTreeMap<SenderId, PendingTransaction<T>>,
    // Used to broadcast new transactions that have been added to the
    // `PendingPool` to `static_subscriber(files)` of this pool
    new_transaction_notifier: broadcast::Sender<PendingTransaction<T>>,
}

impl<T: TransactionOrdering> PendingPool<T> {
    pub fn with_buffer(ordering: T, buffer_capacity: usize) -> Self {
        let (new_transaction_notifier, _) = broadcast::channel(buffer_capacity);
        Self {
            ordering,
            submission_id: 0,
            by_id: Default::default(),
            independent: Default::default(),
            new_transaction_notifier,
        }
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn add_transaction(
        &mut self,
        tx: Arc<ValidPoolTransaction<T::Transaction>>,
        // Base fee of blocks. If Tx fee is under this, It should rejected!
        base_fee: u64,
    ) {
        assert!(
            !self.contains(tx.id()),
            "transaction already included {:?}",
            self.get(tx.id()).unwrap().transaction
        );

        let tx_id = *tx.id();
        let submission_id = self.next_id();
        let priority = self.ordering.priority(&tx.transaction);

        let tx = PendingTransaction {
            submission_id,
            transaction: tx,
            priority,
        };

        if self.new_transaction_notifier.receiver_count() > 0 {
            let _ = self.new_transaction_notifier.send(tx.clone());
        }

        self.by_id.insert(tx_id, tx.clone());
        self.independent.insert(tx_id.sender, tx);
    }

    pub fn remove_transaction(
        &mut self,
        id: &TransactionId,
    ) -> Option<Arc<ValidPoolTransaction<T::Transaction>>> {
        let tx = self.by_id.remove(id)?;
        Some(tx.transaction)
    }

    const fn next_id(&mut self) -> u64 {
        let id = self.submission_id;
        self.submission_id = self.submission_id.wrapping_add(1);
        id
    }

    fn get(&self, id: &TransactionId) -> Option<&PendingTransaction<T>> {
        self.by_id.get(id)
    }

    fn contains(&self, id: &TransactionId) -> bool {
        self.by_id.contains_key(id)
    }

    // independent.values().cloned().collect() collects into a collection 'BtreeSet'
    // using 'Ord" to sort and determine uniqueness.
    pub fn best(&self) -> BestTransactions<T> {
        BestTransactions {
            all: self.by_id.clone(),
            independent: self.independent.values().cloned().collect(),
            invalid: Default::default(),
        }
    }
}

// A transaction that is ready to be included in a block.
// pub(crate): is public inside this crate ( can't use this outside! )
#[derive(Debug)]
pub(crate) struct PendingTransaction<T: TransactionOrdering> {
    pub(crate) submission_id: u64,
    pub(crate) transaction: Arc<ValidPoolTransaction<T::Transaction>>,
    pub(crate) priority: Priority<T::PriorityValue>,
}

impl<T: TransactionOrdering> Clone for PendingTransaction<T> {
    fn clone(&self) -> Self {
        Self {
            submission_id: self.submission_id,
            transaction: Arc::clone(&self.transaction),
            priority: self.priority.clone(),
        }
    }
}

impl<T: TransactionOrdering> Ord for PendingTransaction<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| self.submission_id.cmp(&other.submission_id))
    }
}

impl<T: TransactionOrdering> PartialOrd for PendingTransaction<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: TransactionOrdering> Eq for PendingTransaction<T> {}

impl<T: TransactionOrdering> PartialEq for PendingTransaction<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
