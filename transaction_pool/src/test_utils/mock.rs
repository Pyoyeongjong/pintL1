//! Mocking structs for test!
use std::{collections::HashMap, sync::Arc, time::Instant};

use crate::{
    identifier::{SenderIdentifiers, TransactionId},
    ordering::PintOrdering,
    traits::{PoolTransaction, TransactionOrigin},
    validate::ValidPoolTransaction,
};
use parking_lot::Mutex;
use paste::paste;
use primitives::{
    account::Account,
    types::{Address, B256, ChainId, StorageKey, StorageValue, TxHash, U256},
};
use storage::traits::{ProviderResult, StateProvider, StateProviderBox, StateProviderFactory};
use transaction::transaction::TxEnvelope;

/// Mocking Types
pub type MockValidTx = ValidPoolTransaction<MockTransaction>;
pub type MockOrdering = PintOrdering<MockTransaction>;

// In Rust 2024 edition, we can't use &mut $field or ref mut $field as macro variable.
macro_rules! set_value {
    ($this:ident => $field:ident) => {
        let value = $field;
        match $this {
            MockTransaction::Pint { $field, .. } => {
                *$field = value;
            }
        }
    };
}

// All Field is not implemented Copy trait, so used clone()
macro_rules! get_value {
    ($this:tt => $field:ident) => {
        match $this {
            MockTransaction::Pint { $field, .. } => $field.clone(),
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

/// Transaction Factory that Mocking validate Tx
/// It means, we validate txs with no validation!!
#[derive(Debug, Default)]
pub struct MockTransactionFactory {
    pub(crate) ids: SenderIdentifiers,
}

impl MockTransactionFactory {
    pub fn tx_id(&mut self, tx: &MockTransaction) -> TransactionId {
        let sender = self.ids.sender_id_or_create(tx.sender().clone());
        TransactionId::new(sender, tx.get_nonce())
    }
    // This mocks validation of the transaction.
    // This validation functhion check transaction formats only.
    // Not validate on_chain_balance / on_chain_nonce
    pub fn validate(&mut self, transaction: MockTransaction) -> MockValidTx {
        self.validated_with_origin(TransactionOrigin::External, transaction)
    }

    pub fn validated_with_origin(
        &mut self,
        origin: TransactionOrigin,
        transaction: MockTransaction,
    ) -> MockValidTx {
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
        value: U256,
    },
}

impl MockTransaction {
    make_setters_getters! {
        chain_id => ChainId;
        hash => TxHash;
        sender => Address;
        fee => u128;
        nonce => u64;
        to => Address;
        value => U256
    }

    //#[cfg(feature = "rand") should set to use B256::random!]
    pub fn pint_tx() -> Self {
        Self::Pint {
            chain_id: 1,
            hash: B256::random(),
            fee: 0,
            nonce: 0,
            sender: Address::random(),
            to: Address::random(),
            value: Default::default(),
        }
    }

    pub fn next(&self) -> Self {
        let mut next = self.clone();
        next.set_hash(B256::random());
        next.set_nonce(self.get_nonce() + 1);
        next
    }
}

impl transaction::traits::Transaction for MockTransaction {
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
    type Pooled = TxEnvelope;

    fn tx_type(&self) -> u8 {
        match self {
            Self::Pint { .. } => 0,
        }
    }
    fn hash(&self) -> TxHash {
        self.get_hash()
    }

    fn sender(&self) -> Address {
        self.get_sender()
    }

    fn cost(&self) -> U256 {
        U256::from(self.get_fee())
    }

    fn from_pooled(_: transaction::signed::Recovered<Self::Pooled>) -> Self {
        MockTransaction::pint_tx()
    }
}

/// A provider for mocking!
#[derive(Default, Clone)]
pub struct MockPintProvider {
    pub accounts: Arc<Mutex<HashMap<Address, ExtendedAccount>>>,
}

impl MockPintProvider {
    pub fn add_account(&mut self, address: Address, account: ExtendedAccount) {
        self.accounts.lock().insert(address, account);
    }
}

impl StateProvider for MockPintProvider {
    fn basic_account(
        &self,
        address: &Address,
    ) -> Result<Option<Account>, storage::error::ProviderError> {
        match self.accounts.lock().get(address) {
            Some(extend_account) => Ok(Some(extend_account.account)),
            None => Ok(None),
        }
    }
}

impl StateProviderFactory for MockPintProvider {
    fn latest(&self) -> ProviderResult<StateProviderBox> {
        Ok(Box::new(self.clone()))
    }

    fn state_by_block_hash(
        &self,
        block: primitives::types::BlockHash,
    ) -> ProviderResult<StateProviderBox> {
        todo!()
    }
}

/// Extended Account
pub struct ExtendedAccount {
    account: Account,
    storage: HashMap<StorageKey, StorageValue>,
}

impl ExtendedAccount {
    pub fn new(nonce: u64, balance: U256) -> Self {
        let account = Account { nonce, balance };
        Self {
            account,
            storage: Default::default(),
        }
    }
}
