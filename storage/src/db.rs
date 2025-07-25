use core::num;
use primitives::{
    account::Account,
    block::{self, traits::Block},
    types::{Address, BlockHash, U256},
};
use std::{
    collections::HashMap,
    hash::Hash,
    ops::Add,
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
    fn copy_state_from_block_no(
        &self,
        number: u64,
    ) -> Result<HashMap<Address, Account>, DatabaseError>;
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
        let states_by_block: Arc<Mutex<HashMap<u64, HashMap<Address, Account>>>> =
            Default::default();

        states_by_block
            .lock()
            .unwrap()
            .insert(0, Default::default());

        Self {
            states_by_block: states_by_block,
            block_hash: Default::default(),
            latest: Default::default(),
        }
    }

    pub fn set_balance(&mut self, address: Address, balance: U256) -> Result<(), DatabaseError> {
        let mut block = self
            .states_by_block
            .lock()
            .map_err(|_| DatabaseError::LockError)?;

        let map = block
            .get_mut(&self.latest)
            .ok_or(DatabaseError::StateNotFoundError)?;

        if let Some(account) = map.get_mut(&address) {
            account.balance = balance;
            Ok(())
        } else {
            map.insert(
                address,
                Account {
                    nonce: 0,
                    balance: balance,
                },
            );
            Ok(())
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

    fn copy_state_from_block_no(
        &self,
        number: u64,
    ) -> Result<HashMap<Address, Account>, DatabaseError> {
        let states = self
            .states_by_block
            .lock()
            .map_err(|_| DatabaseError::LockError)?;
        let state = states
            .get(&number)
            .ok_or(DatabaseError::StateNotFoundError)?
            .clone();
        Ok(state)
    }
}
