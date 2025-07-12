use std::fmt;

#[derive(Debug)]
pub enum ProviderError {
    InvalidSomething,
    DatabaseError(DatabaseError),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderError::InvalidSomething => write!(f, "Invalid something"),
            ProviderError::DatabaseError(_) => write!(f, "Database Error"),
        }
    }
}

impl From<DatabaseError> for ProviderError {
    fn from(value: DatabaseError) -> Self {
        Self::DatabaseError(value)
    }
}

impl std::error::Error for ProviderError {}

#[derive(Debug)]
pub enum DatabaseError {
    LockError,
    StateNotFoundError,
}
