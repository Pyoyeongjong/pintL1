pub mod error;
pub mod traits;

use crate::traits::{StateProvider, StateProviderFactory};

/// State which is created by StateProviderFactory
pub struct PintStateProvider {}

impl StateProvider for PintStateProvider {
    fn basic_account(
        &self,
        address: &primitives::types::Address,
    ) -> Result<Option<primitives::account::Account>, crate::error::ProviderError> {
        todo!()
    }
}

/// Factory that makes StateProvider
pub struct PintStateProviderFactory {}

impl StateProviderFactory for PintStateProviderFactory {
    fn latest(&self) -> crate::traits::ProviderResult<crate::traits::StateProviderBox> {
        todo!()
    }

    fn state_by_block_hash(
        &self,
        block: primitives::types::BlockHash,
    ) -> crate::traits::ProviderResult<crate::traits::StateProviderBox> {
        todo!()
    }
}
