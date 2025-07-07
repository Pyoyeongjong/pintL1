//! Errors for primitive trait or structs
use std::{error::Error, fmt};
/// Signature Error
#[derive(Debug)]
pub enum SignatureError {
    InvalidParity(u64),
    FromHex(hex::FromHexError),
    RecoveryError,
}

impl From<hex::FromHexError> for SignatureError {
    fn from(err: hex::FromHexError) -> Self {
        Self::FromHex(err)
    }
}

#[derive(Debug)]
pub enum AddressError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressError::InvalidHex(e) => write!(f, "Invalid hex: {}", e),
            AddressError::InvalidLength(e) => write!(f, "Invalid hex Length: {}", e),
        }
    }
}

impl Error for AddressError {}

impl From<hex::FromHexError> for AddressError {
    fn from(err: hex::FromHexError) -> Self {
        Self::InvalidHex(err)
    }
}
