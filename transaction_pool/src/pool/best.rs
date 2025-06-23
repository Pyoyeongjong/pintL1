use std::collections::BTreeMap;

use tokio::sync::broadcast;

use crate::identifier::TransactionId;
use crate::{ordering::TransactionOrdering, pool::txpool::PendingTransaction};

#[derive(Debug)]
pub struct BestTransactions<T: TransactionOrdering> {
    pub(crate) all: BTreeMap<TransactionId, PendingTransaction<T>>,
    pub(crate) new_transaction_receiver: Option<broadcast::Receiver<PendingTransaction<T>>>,
}
