use primitives::types::Address;
use transaction::{ChainId, U256, traits::Transaction};
use transaction_pool::traits::PoolTransaction;

use crate::traits::ExecutableTx;

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
