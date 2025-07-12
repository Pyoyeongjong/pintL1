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
}

pub trait AccountReader {
    fn basic_account(&self, address: &Address) -> Result<Option<Account>, ProviderError>;
}

pub type StateProviderBox = Box<dyn StateProvider>;

impl StateProvider for StateProviderBox {}

impl AccountReader for StateProviderBox {
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
