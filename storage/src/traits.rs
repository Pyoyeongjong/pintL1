use std::collections::HashMap;

use primitives::{
    account::Account,
    types::{Address, BlockHash, U256},
};

use crate::{db::Database, error::ProviderError};

/// State which is created by StateProviderFactory
pub trait StateProvider: AccountReader {
    fn account_balance(&self, addr: &Address) -> ProviderResult<Option<U256>> {
        self.basic_account(addr)?
            .map_or_else(|| Ok(None), |acc| Ok(Some(acc.balance)))
    }

    fn account_nonce(&self, addr: &Address) -> ProviderResult<Option<u64>> {
        self.basic_account(addr)?
            .map_or_else(|| Ok(None), |acc| Ok(Some(acc.nonce)))
    }

    fn prepare_execute(&self) -> ProviderResult<HashMap<Address, Account>>;
}

impl Database for StateProviderBox {
    fn basic(&self, address: &Address) -> Result<Option<Account>, crate::error::DatabaseError> {
        todo!()
    }

    fn block_hash(&self, number: u64) -> Result<Option<BlockHash>, crate::error::DatabaseError> {
        todo!()
    }

    fn block_number(&self) -> u64 {
        todo!()
    }

    fn copy_state_from_block_no(
        &self,
        number: u64,
    ) -> Result<std::collections::HashMap<Address, Account>, crate::error::DatabaseError> {
        todo!()
    }
}

pub trait AccountReader {
    fn basic_account(&self, address: &Address) -> Result<Option<Account>, ProviderError>;
}

pub type StateProviderBox = Box<dyn StateProvider>;

impl<T: StateProvider + ?Sized> StateProvider for Box<T> {
    fn prepare_execute(&self) -> ProviderResult<HashMap<Address, Account>> {
        (**self).prepare_execute()
    }
}

impl<T: StateProvider + ?Sized> AccountReader for Box<T> {
    fn basic_account(&self, address: &Address) -> Result<Option<Account>, ProviderError> {
        (**self).basic_account(address)
    }
}

pub type ProviderResult<Ok> = Result<Ok, ProviderError>;

/// Factory that makes StateProvider
pub trait StateProviderFactory {
    fn latest(&self) -> ProviderResult<StateProviderBox>;
    fn state_by_block_number(&self, block: u64) -> ProviderResult<StateProviderBox>;
}
