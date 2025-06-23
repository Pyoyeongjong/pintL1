use core::cmp::Ord;
use primitives::types::{TxHash, U256};
use primitives::{Transaction, transaction};
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::ops::Sub;
use std::sync::{Arc, RwLock};
use tracing::trace;

use crate::error::{InsertErr, PoolError, PoolErrorKind, PoolResult};
use crate::identifier::{SenderId, TransactionId};
use crate::pool::parked::ParkedPool;
use crate::pool::pending::PendingPool;
use crate::pool::state::{SubPool, TxState};
use crate::pool::{AddedPendingTransaction, AddedTransaction};
use crate::validate::ValidPoolTransaction;
use crate::{
    config::PoolConfig,
    ordering::{Priority, TransactionOrdering},
    traits::PoolTransaction,
};
use tokio::sync::broadcast;

pub struct TransactionPool<T>
where
    T: TransactionOrdering,
{
    pool: RwLock<TxPool<T>>,
}

pub struct InsertOk<T: PoolTransaction> {
    transaction: Arc<ValidPoolTransaction<T>>,
    replaced_tx: Option<(Arc<ValidPoolTransaction<T>>, SubPool)>,
    sub_pool: SubPool,
}

// 아니 트랜잭션이면 트랜잭션이지 왜 굳이 ValidPoolTransaction, PoolInternalTransaction 나누냐?
// ValidPoolTransaction<T>: 검증된 트랜잭션
// PoolInternalTransaction: 검증된 트랜잭션에 additional info를 붙임
pub(crate) struct AllTransactions<T: PoolTransaction> {
    by_hash: HashMap<TxHash, Arc<ValidPoolTransaction<T>>>,
    txs: BTreeMap<TransactionId, PoolInternalTransaction<T>>,
}

impl<T: PoolTransaction> AllTransactions<T> {
    fn new(config: &PoolConfig) -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn contains(&self, hash: &TxHash) -> bool {
        self.by_hash.contains_key(hash)
    }

    pub fn remove_transaction_by_hash(
        &mut self,
        hash: &TxHash,
    ) -> Option<(Arc<ValidPoolTransaction<T>>, SubPool)> {
        let tx = self.by_hash.remove(hash)?;
        let internal = self.txs.remove(&tx.transaction_id)?;

        Some((tx, internal.subpool))
    }

    pub fn remove_transaction(
        &mut self,
        id: &TransactionId,
    ) -> Option<(Arc<ValidPoolTransaction<T>>, SubPool)> {
        let internal = self.txs.remove(id)?;
        let hash = internal.transaction.hash();
        let tx = self.by_hash.remove(&hash)?;

        Some((tx, internal.subpool))
    }

    pub fn insert_tx(
        &mut self,
        transaction: ValidPoolTransaction<T>,
        on_chain_balance: U256,
        on_chain_nonce: u64,
    ) -> Result<InsertOk<T>, InsertErr<T>> {
        // invariant check: after several varifies we use this function.
        assert!(
            on_chain_nonce <= transaction.nonce(),
            "Invalid transaction."
        );

        let mut state: TxState = Default::default();
        let tx = Arc::new(transaction);
        let mut replaced_tx = None;

        if tx.transaction.cost() > U256::from(0) {
            state.has_fee();
        } else {
            state.has_no_fee();
            return Err(InsertErr::InvalidTransaction { transaction: tx });
        }

        if tx.transaction.nonce() > on_chain_nonce {
            state.has_ancestor();
        } else {
            state.has_no_ancestor();
        }

        let pool_tx = PoolInternalTransaction {
            transaction: Arc::clone(&tx),
            subpool: state.into(),
            state,
        };

        match self.txs.entry(*pool_tx.transaction.id()) {
            // Newly inserted transactionId
            Entry::Vacant(entry) => {
                self.by_hash
                    .insert(pool_tx.transaction.hash(), Arc::clone(&tx));
                entry.insert(pool_tx);
            }
            // Already inserted transactionId
            // 1. compare price of both transactions
            // 2. if new tx wins, replace it.
            Entry::Occupied(mut entry) => {
                let old_tx: &ValidPoolTransaction<T> = entry.get().transaction.as_ref();
                let new_tx = tx.as_ref();

                if old_tx.is_underpriced(new_tx) {
                    return Err(InsertErr::Underpriced { transaction: tx });
                }

                let new_hash = new_tx.transaction.hash();
                let new_tx = pool_tx.transaction.clone();
                let replaced = entry.insert(pool_tx);
                self.by_hash.remove(&replaced.transaction.hash());
                self.by_hash.insert(new_hash, new_tx);

                replaced_tx = Some((replaced.transaction, replaced.subpool))
            }
        }

        Ok(InsertOk {
            transaction: tx,
            replaced_tx,
            sub_pool: state.into(),
        })
    }
}

impl<T: PoolTransaction> Default for AllTransactions<T> {
    fn default() -> Self {
        Self {
            by_hash: Default::default(),
            txs: Default::default(),
        }
    }
}

pub struct PoolInternalTransaction<T: PoolTransaction> {
    transaction: Arc<ValidPoolTransaction<T>>,
    state: TxState,
    subpool: SubPool,
}

#[derive(Debug, Clone, Default)]
pub struct SenderInfo {
    pub(crate) state_nonce: u64,
    pub(crate) balance: U256,
}

impl SenderInfo {
    pub fn update(&mut self, state_nonce: u64, balance: U256) {
        self.state_nonce = state_nonce;
        self.balance = balance;
    }
}

pub struct TxPool<T: TransactionOrdering> {
    sender_info: HashMap<SenderId, SenderInfo>,
    all_transactions: AllTransactions<T::Transaction>,
    pending_pool: PendingPool<T>,
    // queue subpool = sender's lack balance or nonce gap.
    // basefee_pool = currently have not sufficient base_fee, in future it can move to pending pool
    // In this project, base_fee is the number that bigger than just 0, so basefee_pool is no here.
    parked_pool: ParkedPool<T>,
    config: PoolConfig,
}

impl<T: TransactionOrdering> TxPool<T> {
    pub fn new(ordering: T, config: PoolConfig) -> Self {
        Self {
            sender_info: Default::default(),
            pending_pool: PendingPool::with_buffer(
                ordering,
                config.max_new_pending_txs_notifications,
            ),
            parked_pool: Default::default(),
            all_transactions: AllTransactions::new(&config),
            config,
        }
    }

    // This function is called after validation of super struct
    pub(crate) fn add_transaction(
        &mut self,
        transaction: ValidPoolTransaction<T::Transaction>,
        on_chain_balance: U256,
        on_chain_nonce: u64,
    ) -> PoolResult<AddedTransaction<T::Transaction>> {
        if self.contains(&transaction.hash()) {
            return Err(PoolError::new(
                transaction.hash(),
                PoolErrorKind::AlreadyImported,
            ));
        }

        self.validate_auth(&transaction, on_chain_nonce)?;

        // Update sender info with balance and nonce
        // It is for cached StateProvider. StateProvider is expensive to validate
        self.sender_info
            .entry(transaction.sender_id())
            .or_default()
            .update(on_chain_nonce, on_chain_balance);

        // If all_transaction inserted Ok,
        // Choose whether tx is inserted pending or parked pool
        // Current Parked Pool condition: Fee is zero
        match self
            .all_transactions
            .insert_tx(transaction, on_chain_balance, on_chain_nonce)
        {
            Ok(InsertOk {
                transaction,
                replaced_tx,
                sub_pool,
            }) => {
                self.add_new_transaction(transaction.clone(), replaced_tx.clone(), sub_pool);

                let res = if sub_pool.is_pending() {
                    AddedTransaction::Pending(AddedPendingTransaction { transaction })
                } else {
                    AddedTransaction::Parked { transaction }
                };

                Ok(res)
            }
            Err(err) => match err {
                InsertErr::Underpriced { transaction } => Err(PoolError::new(
                    transaction.hash(),
                    PoolErrorKind::RelpacementUnderpriced,
                )),
                InsertErr::InvalidTransaction { transaction } => Err(PoolError::new(
                    transaction.hash(),
                    PoolErrorKind::InvalidTransaction,
                )),
            },
        }
    }

    pub(crate) fn remove_transaction(
        &mut self,
        id: &TransactionId,
    ) -> Option<Arc<ValidPoolTransaction<T::Transaction>>> {
        let (tx, subpool) = self.all_transactions.remove_transaction(id)?;
        self.remove_from_subpool(tx.id(), subpool)
    }

    pub(crate) fn remove_transactions(
        &mut self,
        hashes: Vec<TxHash>,
    ) -> Vec<Arc<ValidPoolTransaction<T::Transaction>>> {
        let txs = hashes
            .into_iter()
            .filter_map(|hash| self.remove_transaction_by_hash(&hash))
            .collect();
        txs
    }

    fn remove_transaction_by_hash(
        &mut self,
        tx_hash: &TxHash,
    ) -> Option<Arc<ValidPoolTransaction<T::Transaction>>> {
        let (tx, subpool) = self.all_transactions.remove_transaction_by_hash(tx_hash)?;

        // After remove, its decendant must become parked due to the nonce gap
        self.remove_from_subpool(tx.id(), subpool)
    }

    fn add_new_transaction(
        &mut self,
        transaction: Arc<ValidPoolTransaction<T::Transaction>>,
        replaced_tx: Option<(Arc<ValidPoolTransaction<T::Transaction>>, SubPool)>,
        subpool: SubPool,
    ) {
        if let Some((replaced, replaced_subpool)) = replaced_tx {
            self.remove_from_subpool(replaced.id(), replaced_subpool);
        }
        self.add_transaction_to_subpool(transaction, subpool);
    }

    fn remove_from_subpool(
        &mut self,
        tx_id: &TransactionId,
        subpool: SubPool,
    ) -> Option<Arc<ValidPoolTransaction<T::Transaction>>> {
        let tx = match subpool {
            SubPool::Pending => self.pending_pool.remove_transaction(tx_id),
            SubPool::Parked => self.parked_pool.remove_transaction(tx_id),
        };

        if let Some(ref tx) = tx {
            // ? = std::fmt::Debug
            // should use tracing lib to manage this huge program!
            trace!(target: "txpool", hash=%tx.transaction.hash(), ?subpool, "Removed transaction from a subpool");
        }
        tx
    }

    fn add_transaction_to_subpool(
        &mut self,
        transaction: Arc<ValidPoolTransaction<T::Transaction>>,
        subpool: SubPool,
    ) {
        match subpool {
            SubPool::Parked => {
                self.parked_pool.add_transaction(transaction);
            }
            SubPool::Pending => {
                self.pending_pool.add_transaction(transaction, 0);
            }
        }
    }

    pub(crate) fn contains(&self, tx_hash: &TxHash) -> bool {
        self.all_transactions.contains(tx_hash)
    }

    // This verifies that the transaction compiles with code authorization??
    // Pass
    fn validate_auth(
        &self,
        transaction: &ValidPoolTransaction<T::Transaction>,
        on_chain_nonce: u64,
    ) -> Result<(), PoolError> {
        Ok(())
    }
}

// A transaction that is ready to be incloded in a block.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::mock::{MockOrdering, MockTransaction, MockTransactionFactory};
    use core::default::Default;

    #[test]
    fn test_insert_pending_pool() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        let mut tx = MockTransaction::pint_tx();
        tx.set_fee(1);

        let vtx = factory.validate(tx);
        let on_chain_balance = U256::from(100);
        let on_chain_nonce = 0;

        // on_chain_balance/on_chain_nonce: account balance/nonce who sends tx
        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);

        assert_eq!(1, pool.pending_pool.len());
        assert_eq!(0, pool.parked_pool.len());
    }

    #[test]
    fn test_insert_already_imported() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        let mut tx = MockTransaction::pint_tx();
        tx.set_fee(1);

        let vtx = factory.validate(tx);
        let on_chain_balance = U256::from(100);
        let on_chain_nonce = 0;

        let _ = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);

        assert_eq!(_res.unwrap_err().kind, PoolErrorKind::AlreadyImported);
        assert_eq!(1, pool.pending_pool.len());
    }

    #[test]
    fn test_insert_parked_pool() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        let mut tx = MockTransaction::pint_tx();
        tx.set_fee(10);
        tx.set_value(U256::from(10));
        tx.set_nonce(1);

        let vtx = factory.validate(tx);
        let on_chain_balance = U256::from(10);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);

        assert_eq!(0, pool.pending_pool.len());
        assert_eq!(1, pool.parked_pool.len());
    }

    #[test]
    #[should_panic(expected = "Invalid transaction")]
    fn test_insert_invalid_nonce() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        // nonce = 0
        let mut tx = MockTransaction::pint_tx();
        tx.set_fee(1);

        let vtx = factory.validate(tx);
        let on_chain_balance = U256::from(100);
        // sender's on_chain_nonce
        let on_chain_nonce = 1;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
    }

    #[test]
    fn test_insert_invalid_fee() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        // nonce = 0
        let mut tx = MockTransaction::pint_tx();
        tx.set_fee(0);

        let vtx = factory.validate(tx);
        let on_chain_balance = U256::from(100);
        // sender's on_chain_nonce
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
        assert_eq!(_res.unwrap_err().kind, PoolErrorKind::InvalidTransaction);
    }

    #[test]
    fn test_insert_txs_same_sender() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        let mut tx = MockTransaction::pint_tx();
        tx.set_fee(1);

        let mut next_tx = tx.next();
        next_tx.set_fee(1);

        let vtx = factory.validate(tx);
        let next_vtx = factory.validate(next_tx);

        let on_chain_balance = U256::from(100);
        // sender's on_chain_nonce
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
        let _res = pool.add_transaction(next_vtx.clone(), on_chain_balance, on_chain_nonce);
        assert_eq!(1, pool.pending_pool.len());
        assert_eq!(1, pool.parked_pool.len());
    }

    #[test]
    fn test_replace_tx_with_higher_fee_pending() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        let mut tx = MockTransaction::pint_tx();
        tx.set_fee(1);

        let mut next_tx = tx.next();
        next_tx.set_nonce(0);
        next_tx.set_fee(2);

        let vtx = factory.validate(tx);
        let next_vtx = factory.validate(next_tx);

        let on_chain_balance = U256::from(100);
        // sender's on_chain_nonce
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
        let _res = pool.add_transaction(next_vtx.clone(), on_chain_balance, on_chain_nonce);
        assert_eq!(1, pool.pending_pool.len());
        assert_eq!(0, pool.parked_pool.len());
    }

    #[test]
    fn test_replace_tx_with_higher_fee_parked() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        let mut tx = MockTransaction::pint_tx();
        tx.set_nonce(1);
        tx.set_fee(1);

        let mut next_tx = tx.next();
        next_tx.set_nonce(1);
        next_tx.set_fee(2);

        let vtx = factory.validate(tx);
        let next_vtx = factory.validate(next_tx);

        let on_chain_balance = U256::from(100);
        // sender's on_chain_nonce
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
        let _res = pool.add_transaction(next_vtx.clone(), on_chain_balance, on_chain_nonce);
        assert_eq!(0, pool.pending_pool.len());
        assert_eq!(1, pool.parked_pool.len());
    }

    #[test]
    fn test_replace_tx_with_lower_fee_pending() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        let mut tx = MockTransaction::pint_tx();
        tx.set_fee(2);

        let mut next_tx = tx.next();
        next_tx.set_nonce(0);
        next_tx.set_fee(1);

        let vtx = factory.validate(tx);
        let next_vtx = factory.validate(next_tx);

        let on_chain_balance = U256::from(100);
        // sender's on_chain_nonce
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
        let _res = pool.add_transaction(next_vtx.clone(), on_chain_balance, on_chain_nonce);
        assert_eq!(1, pool.pending_pool.len());
        assert_eq!(0, pool.parked_pool.len());
        assert_eq!(
            _res.unwrap_err().kind,
            PoolErrorKind::RelpacementUnderpriced
        )
    }

    #[test]
    fn test_replace_tx_with_lower_fee_parked() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        let mut tx = MockTransaction::pint_tx();
        tx.set_nonce(1);
        tx.set_fee(2);

        let mut next_tx = tx.next();
        next_tx.set_nonce(1);
        next_tx.set_fee(1);

        let vtx = factory.validate(tx);
        let next_vtx = factory.validate(next_tx);

        let on_chain_balance = U256::from(100);
        // sender's on_chain_nonce
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
        let _res = pool.add_transaction(next_vtx.clone(), on_chain_balance, on_chain_nonce);
        assert_eq!(0, pool.pending_pool.len());
        assert_eq!(1, pool.parked_pool.len());
        assert_eq!(
            _res.unwrap_err().kind,
            PoolErrorKind::RelpacementUnderpriced
        )
    }

    #[test]
    fn test_multiple_senders() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        let mut tx1 = MockTransaction::pint_tx();
        tx1.set_fee(1);
        let mut tx2 = MockTransaction::pint_tx();
        tx2.set_fee(1);

        let vtx1 = factory.validate(tx1);
        let vtx2 = factory.validate(tx2);

        let on_chain_balance = U256::from(100);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx1.clone(), on_chain_balance, on_chain_nonce);
        let _res = pool.add_transaction(vtx2.clone(), on_chain_balance, on_chain_nonce);

        assert_eq!(2, pool.pending_pool.len());
        assert_eq!(0, pool.parked_pool.len());
    }

    #[test]
    fn test_remove_pending_tx() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        let mut tx = MockTransaction::pint_tx();
        tx.set_fee(1);

        let vtx = factory.validate(tx);
        let on_chain_balance = U256::from(100);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
        assert_eq!(1, pool.pending_pool.len());

        pool.remove_transaction(vtx.id());
        assert_eq!(0, pool.pending_pool.len());
    }

    #[test]
    fn test_remove_parked_tx() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        let mut tx = MockTransaction::pint_tx();
        tx.set_nonce(1);
        tx.set_fee(1);

        let vtx = factory.validate(tx);
        let on_chain_balance = U256::from(100);
        let on_chain_nonce = 0;

        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);
        assert_eq!(1, pool.parked_pool.len());

        pool.remove_transaction(vtx.id());
        assert_eq!(0, pool.pending_pool.len());
    }
}
