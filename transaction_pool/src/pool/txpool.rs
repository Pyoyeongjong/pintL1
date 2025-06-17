use std::collections::btree_map::Entry;
use std::hash::Hash;
use std::ops::Sub;
use std::sync::{Arc, RwLock};
use std::collections::{BTreeMap, HashMap};
use core::cmp::Ord;
use primitives::types::{TxHash, U256};
use primitives::{transaction, Transaction};

use crate::error::{InsertErr, PoolError, PoolErrorKind, PoolResult};
use crate::identifier::{SenderId, TransactionId};
use crate::pool::parked::ParkedPool;
use crate::pool::pending::PendingPool;
use crate::pool::state::{SubPool, TxState};
use crate::pool::{AddedPendingTransaction, AddedTransaction};
use crate::validate::ValidPoolTransaction;
use crate::{config::PoolConfig, traits::PoolTransaction, ordering::{Priority, TransactionOrdering}};
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
    sub_pool: SubPool
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

    fn contains(&self, hash: &TxHash) -> bool {
        self.by_hash.contains_key(hash)
    }

    fn insert_tx(
        &mut self, 
        transaction: ValidPoolTransaction<T>, 
        on_chain_balance: U256,
        on_chain_nonce: u64
    ) -> Result<InsertOk<T>, InsertErr<T>>{

        assert!(on_chain_nonce <= transaction.nonce(), "Invalid transaction.");

        let mut state: TxState = Default::default();
        let tx = Arc::new(transaction);
        let mut replaced_tx = None;

        
        if tx.transaction.cost() > U256::from(0) {
            state.has_fee();
        }
        

        let pool_tx = PoolInternalTransaction {
            transaction: Arc::clone(&tx),
            subpool: state.into(),
            state
        };


        match self.txs.entry(*pool_tx.transaction.id()) {
            // Newly inserted transactionId
            Entry::Vacant(entry) => {
                self.by_hash.insert(pool_tx.transaction.hash(), Arc::clone(&tx));
                entry.insert(pool_tx);
            },
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

        Ok(InsertOk { transaction: tx, replaced_tx, sub_pool: state.into() })
    }
}

impl<T: PoolTransaction> Default for AllTransactions<T> {
    fn default() -> Self {
        Self {
            by_hash: Default::default(),
            txs: Default::default()
        }
    }
}

pub struct PoolInternalTransaction<T: PoolTransaction> {
    transaction: Arc<ValidPoolTransaction<T>>,
    state: TxState,
    subpool: SubPool
}

#[derive(Debug, Clone, Default)]
pub struct SenderInfo {
    pub(crate) state_nonce: u64,
    pub(crate) balance: U256
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
    parked_pool: ParkedPool<T>,
    config: PoolConfig
}


impl<T: TransactionOrdering> TxPool<T> {
    pub fn new(ordering: T, config: PoolConfig) -> Self {
        Self {
            sender_info: Default::default(),
            pending_pool: PendingPool::with_buffer(
                ordering,
                config.max_new_pending_txs_notifications
            ),
            parked_pool: Default::default(),
            all_transactions: AllTransactions::new(&config),         
            config
        }
    }

    // This function is called after validation of super struct
    pub(crate) fn add_transaction(
        &mut self,
        transaction: ValidPoolTransaction<T::Transaction>,
        on_chain_balance: U256,
        on_chain_nonce: u64
    ) -> PoolResult<AddedTransaction<T::Transaction>> {
        if self.contains(&transaction.hash()) {
            return Err(PoolError::new(transaction.hash(), PoolErrorKind::AlreadyImported))
        }

        self.validate_auth(&transaction, on_chain_nonce)?;

        // Update sender info with balance and nonce
        // It is for cached StateProvider. StateProvider is expensive to validate 
        self.sender_info.entry(transaction.sender_id()).or_default().update(on_chain_nonce, on_chain_balance);

        // If all_transaction inserted Ok,
        // Choose whether tx is inserted pending or parked pool
        // Current Parked Pool condition: Fee is zero
        match self.all_transactions.insert_tx(transaction, on_chain_balance, on_chain_nonce) {

            Ok(InsertOk {transaction, replaced_tx, sub_pool}) => {
                self.add_new_transaction(transaction.clone(), replaced_tx.clone(), sub_pool);

                let res = if sub_pool.is_pending() {
                    AddedTransaction::Pending(AddedPendingTransaction{
                        transaction
                    })
                } else {
                    AddedTransaction::Parked { transaction }
                };

                Ok(res)
            },
            Err(err) => {
                match err {
                    InsertErr::Underpriced { transaction } => {
                        Err(PoolError::new(
                            transaction.hash(),
                            PoolErrorKind::RelpacementUnderpriced
                        ))
                    }
                }
            }
        }
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

    fn remove_from_subpool(&mut self, tx_id: &TransactionId, subpool: SubPool) {
        match subpool {
            SubPool::Pending => self.pending_pool.remove_transaction(tx_id),
            SubPool::Parked => self.parked_pool.remove_transaction(tx_id)
        };
    }

    fn add_transaction_to_subpool(&mut self, transaction: Arc<ValidPoolTransaction<T::Transaction>>, subpool: SubPool) {
        match subpool {
            SubPool::Parked => {
                self.parked_pool.add_transaction(transaction);
            },
            SubPool::Pending => {
                self.pending_pool.add_transaction(transaction, 0);
            }
            
        }
    }

    pub(crate) fn contains(&self, tx_hash: &TxHash) -> bool {
        self.all_transactions.contains(tx_hash)
    }

    fn validate_auth(&self, transaction: &ValidPoolTransaction<T::Transaction>, on_chain_nonce: u64) -> Result<(), PoolError>{
        Ok(())
    }
}



// A transaction that is ready to be incloded in a block.
// pub(crate): is public inside this crate ( can't use this outside! )
#[derive(Debug)]
pub(crate) struct PendingTransaction<T: TransactionOrdering> {
    pub(crate) submission_id: u64,
    pub(crate) transaction: Arc<ValidPoolTransaction<T::Transaction>>,
    pub(crate) priority: Priority<T::PriorityValue> 
}

impl<T: TransactionOrdering> Clone for PendingTransaction<T> {
    fn clone(&self) -> Self {
        Self {
            submission_id: self.submission_id,
            transaction: Arc::clone(&self.transaction),
            priority: self.priority.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::mock::{MockOrdering, MockTransaction, MockTransactionFactory};
    use core::default::Default;

    #[test]
    fn test_txpool_add_transaction() {
        let mut factory = MockTransactionFactory::default();
        let mut pool = TxPool::new(MockOrdering::default(), Default::default());

        let tx = MockTransaction::pint_tx();

        let vtx = factory.validate(tx);
        let on_chain_balance = U256::from(100);
        let on_chain_nonce = 0;

        // on_chain_balance/on_chain_nonce: account balance/nonce who sends tx
        let _res = pool.add_transaction(vtx.clone(), on_chain_balance, on_chain_nonce);

        assert_eq!(1, pool.pending_pool.len());
    }

    #[test]
    fn insert_already_imported() {

    }
}