use std::array::TryFromSliceError;

use primitives::error::AddressError;

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
