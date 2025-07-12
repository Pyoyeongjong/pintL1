use primitives::{
    account::Account,
    block::{self, traits::Block},
    types::{Address, BlockHash},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::error::DatabaseError;

/// Database Trait
pub trait Database {
    // latest account info
    fn basic(&self, address: &Address) -> Result<Option<Account>, DatabaseError>;
    // block hash at given block_number
    fn block_hash(&self, number: u64) -> Result<Option<BlockHash>, DatabaseError>;
    fn block_number(&self) -> u64;
}

/// In Memory Database for small project.
#[derive(Clone, Default)]
pub struct InMemoryDB {
    states_by_block: Arc<Mutex<HashMap<u64, HashMap<Address, Account>>>>,
    block_hash: Arc<Mutex<HashMap<u64, BlockHash>>>,
    latest: u64,
}

impl InMemoryDB {
    pub fn new() -> Self {
        Self {
            states_by_block: Default::default(),
            block_hash: Default::default(),
            latest: Default::default(),
        }
    }
}

impl Database for InMemoryDB {
    fn basic(&self, address: &Address) -> Result<Option<Account>, DatabaseError> {
        let binding = match self.states_by_block.lock() {
            Ok(res) => res,
            Err(_) => return Err(DatabaseError::LockError),
        };
        let state = match binding.get(&self.latest) {
            Some(state) => state,
            None => return Err(DatabaseError::StateNotFoundError),
        };
        let res = state.get(address);
        Ok(res.copied())
    }

    fn block_hash(&self, number: u64) -> Result<Option<BlockHash>, DatabaseError> {
        let binding = match self.block_hash.lock() {
            Ok(res) => res,
            Err(_) => return Err(DatabaseError::LockError),
        };
        let block_hash = binding.get(&number);
        Ok(block_hash.copied())
    }

    fn block_number(&self) -> u64 {
        self.latest
    }
}
