pub mod db;
pub mod error;
pub mod traits;

use std::collections::HashMap;

use primitives::account::Account;

use crate::{
    db::Database,
    error::ProviderError,
    traits::{AccountReader, ProviderResult, StateProvider, StateProviderFactory},
};

/// State which is created by StateProviderFactory
pub struct PintStateProvider<DB> {
    db: DB,
    block_no: u64,
}

impl<DB: Database> StateProvider for PintStateProvider<DB> {
    fn prepare_execute(&self) -> ProviderResult<HashMap<primitives::types::Address, Account>> {
        let res = match self.db.copy_state_from_block_no(self.block_no) {
            Ok(res) => res,
            Err(e) => return Err(ProviderError::DatabaseError(e)),
        };
        Ok(res)
    }
}

impl<DB: Database> AccountReader for PintStateProvider<DB> {
    fn basic_account(
        &self,
        address: &primitives::types::Address,
    ) -> Result<Option<Account>, ProviderError> {
        Ok(self.db.basic(address)?)
    }
}

/// Factory that makes StateProvider
#[derive(Clone)]
pub struct PintStateProviderFactory<DB> {
    pub db: DB,
}

impl<DB> PintStateProviderFactory<DB>
where
    DB: Database,
{
    pub fn new(db: DB) -> Self {
        Self { db }
    }
}

impl<DB> StateProviderFactory for PintStateProviderFactory<DB>
where
    DB: Database + Clone + 'static,
{
    // State for latest block
    fn latest(&self) -> ProviderResult<traits::StateProviderBox> {
        let block_no = self.db.block_number();
        self.state_by_block_number(block_no)
    }
    fn state_by_block_number(&self, block_no: u64) -> ProviderResult<traits::StateProviderBox> {
        Ok(Box::new(PintStateProvider {
            db: self.db.clone(),
            block_no,
        }))
    }
}
