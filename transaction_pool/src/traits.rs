//! Traits for Transaction Pool
use core::default::Default;
use std::{fmt::Debug, sync::Arc};

use primitives::types::{Address, TxHash, U256};
use transaction::{
    TransactionSigned,
    signed::Recovered,
    traits::{SignedTransaction, Transaction},
    transaction::TxEnvelope,
};

use crate::{error::PoolResult, validate::ValidPoolTransaction};

/// Origin of the Transaction
#[derive(Debug, Default, Clone, Copy)]
pub enum TransactionOrigin {
    #[default]
    Local,
    External,
    Private,
}

/// A traits for TransactionPool
///
pub trait TransactionPool {
    type Transaction: PoolTransaction;

    fn add_external_transaction(
        &self,
        transaction: Self::Transaction,
    ) -> impl Future<Output = PoolResult<TxHash>> + Send {
        self.add_transaction(TransactionOrigin::External, transaction)
    }

    fn add_transaction(
        &self,
        origin: TransactionOrigin,
        transaction: Self::Transaction,
    ) -> impl Future<Output = PoolResult<TxHash>> + Send;

    fn get(&self, tx_hash: &TxHash) -> Option<Arc<ValidPoolTransaction<Self::Transaction>>>;

    fn best_transactions(
        &self,
    ) -> Box<dyn BestTransactions<Item = Arc<ValidPoolTransaction<Self::Transaction>>>>;
}

/// Transaction Traits that should implement to get into pool
///
/// A traits for transaction whether it can be validated to [Pool](crate::Pool)
pub trait PoolTransaction: Debug + Transaction {
    type Pooled: SignedTransaction;

    fn tx_type(&self) -> u8;
    fn hash(&self) -> TxHash;
    fn sender(&self) -> Address;
    fn cost(&self) -> U256;
    fn from_pooled(tx: Recovered<Self::Pooled>) -> Self;
}

/// The default [`PoolTransaction`] for the [Pool](crate::Pool)
#[derive(Debug, Clone)]
pub struct PintPooledTransaction<T = TransactionSigned> {
    pub transaction: Recovered<T>,
}

impl Transaction for PintPooledTransaction {
    fn chain_id(&self) -> transaction::ChainId {
        self.transaction.chain_id()
    }

    fn nonce(&self) -> u64 {
        self.transaction.nonce()
    }

    fn value(&self) -> U256 {
        self.transaction.value()
    }

    fn get_priority(&self) -> Option<u128> {
        self.transaction.get_priority()
    }
}

impl PoolTransaction for PintPooledTransaction {
    fn tx_type(&self) -> u8 {
        self.transaction.inner().tx_type()
    }

    fn hash(&self) -> TxHash {
        self.transaction.inner().hash().clone()
    }

    fn sender(&self) -> Address {
        self.transaction.signer().clone()
    }

    fn cost(&self) -> U256 {
        if let Some(cost) = self.transaction.get_priority() {
            return U256::from(cost);
        }
        U256::from(0)
    }

    type Pooled = TxEnvelope;

    fn from_pooled(tx: Recovered<Self::Pooled>) -> Self {
        Self { transaction: tx }
    }
}

/// BestTransactions of the Pool's transaction
pub trait BestTransactions: Iterator + Send {}
