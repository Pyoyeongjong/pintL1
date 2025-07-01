use core::fmt;

pub trait StateProvider {}

pub type ProviderResult<Ok> = Result<Ok, ProviderError>;
pub type StateProviderBox = Box<dyn StateProvider>;

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
