use std::collections::BTreeMap;

use tokio::sync::broadcast;

use crate::{pool::txpool::PendingTransaction, ordering::TransactionOrdering};
use crate::identifier::TransactionId;

#[derive(Debug)]
pub struct BestTransactions<T: TransactionOrdering> {
    pub(crate) all: BTreeMap<TransactionId, PendingTransaction<T>>,
    pub(crate) new_transaction_receiver: Option<broadcast::Receiver<PendingTransaction<T>>>,
}
