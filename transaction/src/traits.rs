//! Transactions primitive traits
use core::fmt;
use primitives::types::{Address, B256, ChainId, TxHash, U256};

use crate::{
    error::{DecodeError, EncodeError, RecoveryError},
    signed::{Recovered, Signed},
};

// A raw transaction
pub trait Transaction: fmt::Debug + Send + Sync + 'static {
    fn chain_id(&self) -> ChainId;
    fn nonce(&self) -> u64;
    fn value(&self) -> U256;
    fn to(&self) -> Address;
    fn get_priority(&self) -> Option<u128>;
}

// A trait for encodable transaction
pub trait Encodable {
    fn encode(&self) -> Result<Vec<u8>, EncodeError>;
}

// A trait for raw string into struct
pub trait Decodable: Sized {
    fn decode(vec: &Vec<u8>) -> Result<(Self, usize), DecodeError>;
}

/// A trait for recovering public key from a signature.
pub trait SignerRecovable {
    fn recover_signer(&self) -> Result<Address, RecoveryError>;
}

/// A trait for signed transaction
pub trait SignedTransaction: SignerRecovable {
    fn tx_hash(&self) -> TxHash;
    // recover signer's signature
    fn try_recover(&self) -> Result<Address, RecoveryError> {
        self.recover_signer()
    }

    fn try_into_recovered(self) -> Result<Recovered<Self>, Self>
    where
        Self: Sized,
    {
        match self.recover_signer() {
            Ok(signer) => Ok(Recovered::new_unchecked(self, signer)),
            Err(_) => Err(self),
        }
    }
}

/// A trait for signable transaction
pub trait SignableTransaction<Signature>: Transaction {
    // Convert to a ['Signed'] Object
    fn into_signed(self, signature: Signature) -> Signed<Self, Signature>
    where
        Self: Sized,
    {
        Signed::new_unhashed(self, signature)
    }

    fn encode_for_signing(&self) -> B256;
}
