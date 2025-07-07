use primitives::{
    account::Account,
    types::{Address, BlockHash},
};

use crate::error::ProviderError;

/// State which is created by StateProviderFactory
pub trait StateProvider {
    fn basic_account(&self, address: &Address) -> Result<Option<Account>, ProviderError>;
}

pub type StateProviderBox = Box<dyn StateProvider>;
pub type ProviderResult<Ok> = Result<Ok, ProviderError>;

/// Factory that makes StateProvider
pub trait StateProviderFactory {
    fn latest(&self) -> ProviderResult<StateProviderBox>;
    fn state_by_block_hash(&self, block: BlockHash) -> ProviderResult<StateProviderBox>;
}
