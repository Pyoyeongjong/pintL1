use std::{ops::Add, time::Instant};

use primitives::{types::{Address, ChainId, TxHash, B256, U256}};
use paste::paste;
use crate::{identifier::{SenderIdentifiers, TransactionId}, ordering::PintOrdering, traits::{PoolTransaction, TransactionOrigin}, validate::ValidPoolTransaction};
use rand::{distr::Uniform, prelude::Distribution};

pub type MockValidTx = ValidPoolTransaction<MockTransaction>;

/*
    Rust 2024 edition에선 &mut $field나 ref mut $field가 매크로 변수로는 사용 불가해.
 */
macro_rules! set_value {
    ($this:ident => $field:ident) => {
        let value = $field;
        match $this {
            MockTransaction::Pint {$field, ..} => {
                *$field = value;
            },
        }
    };
}

// All Field is not implemented Copy trait, so used clone()
macro_rules! get_value {
    ($this:tt => $field:ident) => {
        match $this {
            MockTransaction::Pint {$field, ..} => $field.clone(),
        }
    };
}

// Using paste can add prefix like get_...
macro_rules! make_setters_getters {
    ($($name:ident => $t:ty);*) => {
        paste! {$(
            pub fn [<set_ $name>](&mut self, $name: $t) -> &mut Self {
                set_value!(self => $name);
                self
            }

            pub fn [<get_ $name>](&self) -> $t {
                get_value!(self => $name)
            }
        )*}
    }
}


// Transaction Factory that Mocking validate Tx
#[derive(Debug, Default)]
pub struct MockTransactionFactory {
    pub(crate) ids: SenderIdentifiers
}

impl MockTransactionFactory {

    pub fn tx_id(&mut self, tx: &MockTransaction) -> TransactionId {
        let sender = self.ids.sender_id_or_create(tx.sender());
        TransactionId::new(sender, tx.get_nonce())

    }

    pub fn validate(&mut self, transaction: MockTransaction) -> MockValidTx {
        self.validated_with_origin(TransactionOrigin::External, transaction)
    }

    pub fn validated_with_origin(&mut self, origin: TransactionOrigin, transaction: MockTransaction) -> MockValidTx {
        MockValidTx {
            transaction: transaction.clone(),
            transaction_id: self.tx_id(&transaction),
            origin,
            timestamp: Instant::now(),
        }
    }
}

// Struct for mocking transactions
#[derive(Debug, Clone)]
pub enum MockTransaction {
    Pint {
        chain_id: ChainId,
        hash: TxHash,
        sender: Address,
        fee: u128,
        nonce: u64,
        to: Address,
        value: U256
    },
}

impl MockTransaction {
    make_setters_getters! {
        chain_id => ChainId;
        hash => B256;
        sender => Address;
        fee => u128;
        nonce => u64;
        to => Address;
        value => U256
    }

    //#[cfg(feature = "rand") should set to use B256::random!]
    pub fn pint_tx() -> Self {
        Self::Pint { chain_id: 1, hash: B256::random(), fee: 0, nonce: 0, sender: Address::random(), to: Address::random(), value: Default::default() }
    }
}

impl primitives::Transaction for MockTransaction {

    fn chain_id(&self) -> ChainId {
        self.get_chain_id()
    }
    fn nonce(&self) -> u64 {
        self.get_nonce()
    }
    fn get_priority(&self) -> Option<u128> {
        Some(self.get_fee())
    }
    fn value(&self) -> U256 {
        self.get_value()
    }
}

impl PoolTransaction for MockTransaction {
    fn hash(&self) -> TxHash {
        self.get_hash()
    }

    fn sender(&self) -> Address {
        self.get_sender()
    }

    fn cost(&self) -> U256 {
        U256::from(self.get_fee())
    }
}

pub type MockOrdering = PintOrdering<MockTransaction>;