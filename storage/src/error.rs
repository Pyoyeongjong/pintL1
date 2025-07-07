use std::fmt;

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
