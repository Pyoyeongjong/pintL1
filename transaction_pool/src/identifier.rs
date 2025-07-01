//! Identifiers (Sender, Transaction Id, ...) for Transaction Pool
use std::collections::HashMap;

use primitives::types::Address;
use transaction::U256;

// Internal mapping of addresses
// This assigns a unique [`SenderId`] for a new [`Address`].
#[derive(Debug, Default)]
pub struct SenderIdentifiers {
    id: u64,
    address_to_id: HashMap<Address, SenderId>,
    sender_to_address: HashMap<SenderId, Address>,
}

impl SenderIdentifiers {
    pub fn address(&self, id: &SenderId) -> Option<&Address> {
        self.sender_to_address.get(id)
    }
    pub fn sender_id(&self, address: &Address) -> Option<SenderId> {
        self.address_to_id.get(address).copied()
    }
    pub fn sender_id_or_create(&mut self, addr: Address) -> SenderId {
        self.sender_id(&addr).unwrap_or_else(|| {
            let id = self.next_id();
            self.address_to_id.insert(addr.clone(), id);
            self.sender_to_address.insert(id, addr);
            id
        })
    }

    fn next_id(&mut self) -> SenderId {
        let id = self.id;
        self.id = self.id.wrapping_add(1);
        id.into()
    }
}

// Ord, Eq: Always can compare each other!
// PartialOrd, PartialEq: Usually can compare each other
#[derive(Debug, Clone, Ord, PartialEq, Eq, PartialOrd, Copy)]
pub struct TransactionId {
    pub sender: SenderId,
    pub nonce: u64,
}

impl TransactionId {
    pub fn new(sender: SenderId, nonce: u64) -> Self {
        Self { sender, nonce }
    }
}

#[derive(Debug, Clone, Ord, PartialEq, PartialOrd, Eq, Hash, Copy)]
pub struct SenderId(u64);

impl From<u64> for SenderId {
    fn from(value: u64) -> Self {
        Self(value)
    }
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
