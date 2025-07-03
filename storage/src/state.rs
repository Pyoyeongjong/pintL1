use core::fmt;
use primitives::{account::Account, types::Address};

pub trait StateProvider {
    fn basic_account(&self, address: &Address) -> Result<Option<Account>, ProviderError>;
}

pub type StateProviderBox = Box<dyn StateProvider>;
pub type ProviderResult<Ok> = Result<Ok, ProviderError>;

#[derive(Debug)]
pub enum ProviderError {
    InvalidSomething,
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderError::InvalidSomething => write!(f, "Invalid something"),
        }
    }
}

impl std::error::Error for ProviderError {}

pub trait StateProviderFactory {
    fn latest(&self) -> ProviderResult<StateProviderBox>;
}
