//! Traits for Transaction Pool
use core::default::Default;
use std::fmt::Debug;

use primitives::{
    Transaction,
    signed::Recovered,
    transaction::SignedTransaction,
    types::{Address, TxHash, U256},
};
use transaction::{TransactionSigned, transaction::TxEnvelope};

use crate::error::PoolResult;

/// Origin of the Transaction
#[derive(Debug, Default, Clone, Copy)]
pub enum TransactionOrigin {
    #[default]
    Local,
    External,
    Private,
}

/// A traits for transaction whether it can be validated to [Pool](crate::Pool)
pub trait PoolTransaction: Debug + Transaction {
    type Pooled: SignedTransaction;
    fn hash(&self) -> TxHash;
    fn sender(&self) -> Address;
    fn cost(&self) -> U256;
    fn from_pooled(tx: Recovered<Self::Pooled>) -> Self;
}

/// A traits for TransactionPool
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
    fn hash(&self) -> TxHash {
        self.transaction.inner().hash().clone()
    }

    fn sender(&self) -> Address {
        todo!()
    }

    fn cost(&self) -> U256 {
        if let Some(cost) = self.transaction.get_priority() {
            return U256::from(cost);
        }
        U256::from(0)
    }

    type Pooled = TxEnvelope;

    fn from_pooled(tx: Recovered<Self::Pooled>) -> Self {
        todo!()
    }
}
