//! Errors for primitive trait or structs
use std::{array::TryFromSliceError, error::Error, fmt};
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

/// Recovery Error
#[derive(Debug)]
pub enum RecoveryError {
    RecIdError,
    RecKeyError,
    AddressError(AddressError),
    HashGetError,
    RecoveryFromDigestError,
}

impl From<AddressError> for RecoveryError {
    fn from(err: AddressError) -> Self {
        Self::AddressError(err)
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

#[derive(Debug)]
pub enum DecodeError {
    InvalidTxType,
    SignatureLengthError(TryFromSliceError),
    SignatureDecodeError,
    InputTooShort,
    TryFromError(TryFromSliceError),
    InvalidAddress,
}
impl From<TryFromSliceError> for DecodeError {
    fn from(err: TryFromSliceError) -> Self {
        Self::TryFromError(err)
    }
}

#[derive(Debug)]
pub enum EncodeError {
    InvalidSomething,
}
