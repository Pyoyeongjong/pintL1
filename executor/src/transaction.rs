use std::sync::Arc;

use primitives::types::Address;
use transaction::{ChainId, TransactionSigned, U256, signed::Recovered, traits::Transaction};
use transaction_pool::{traits::PoolTransaction, validate::ValidPoolTransaction};

use crate::traits::ExecutableTx;

/// Struct for executable transaction. It
#[derive(Debug)]
pub struct ExecutableTranasction {
    pub tx_type: u8,
    pub chain_id: ChainId,
    pub sender: Address,
    pub receiver: Address,
    pub nonce: u64,
    pub value: U256,
}

impl Transaction for ExecutableTranasction {
    fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }

    fn value(&self) -> U256 {
        self.value
    }

    fn to(&self) -> Address {
        self.receiver.clone()
    }

    fn get_priority(&self) -> Option<u128> {
        todo!()
    }
}

impl ExecutableTx for ExecutableTranasction {
    fn from_pool_transaction<Tx: PoolTransaction>(tx: Tx) -> Self {
        Self {
            tx_type: 0,
            chain_id: tx.chain_id(),
            sender: tx.sender(),
            receiver: tx.to(),
            nonce: tx.nonce(),
            value: tx.value(),
        }
    }
}

impl From<Recovered<TransactionSigned>> for ExecutableTranasction {
    fn from(recovered: Recovered<TransactionSigned>) -> Self {
        let Recovered {
            signer: address,
            inner: tx,
        } = recovered;
        ExecutableTranasction {
            tx_type: tx.tx_type(),
            chain_id: tx.chain_id(),
            sender: address,
            receiver: tx.to(),
            nonce: tx.nonce(),
            value: tx.value(),
        }
    }
}

impl<T> From<Arc<ValidPoolTransaction<T>>> for ExecutableTranasction
where
    T: PoolTransaction,
{
    fn from(tx: Arc<ValidPoolTransaction<T>>) -> Self {
        ExecutableTranasction {
            tx_type: tx.transaction.tx_type(),
            chain_id: tx.transaction.chain_id(),
            sender: tx.transaction.sender(),
            receiver: tx.transaction.to(),
            nonce: tx.transaction.nonce(),
            value: tx.transaction.value(),
        }
    }
}
