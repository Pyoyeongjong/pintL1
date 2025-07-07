//! Signature-related Traits or Structs
use std::{ops::Deref, sync::OnceLock};

use k256::{
    EncodedPoint,
    ecdsa::{RecoveryId, VerifyingKey},
    sha2::{Digest, Sha256},
};
use primitives::{
    signature::Signature,
    types::{Address, B256, ChainId, TxHash, U256},
};

use crate::{
    error::{DecodeError, RecoveryError},
    traits::{Decodable, Encodable, SignableTransaction, SignerRecovable, Transaction},
};

// Signed object with recovered signer
#[derive(Debug, Clone)]
pub struct Recovered<T> {
    signer: Address,
    inner: T,
}

impl<T> Recovered<T> {
    pub const fn new_unchecked(inner: T, signer: Address) -> Self {
        Self { signer, inner }
    }

    pub const fn inner(&self) -> &T {
        &self.inner
    }

    pub const fn signer(&self) -> &Address {
        &self.signer
    }
}

impl<T: Transaction> Recovered<T> {
    pub fn chain_id(&self) -> ChainId {
        self.inner.chain_id()
    }

    pub fn nonce(&self) -> u64 {
        self.inner.nonce()
    }

    pub fn value(&self) -> U256 {
        self.inner.value()
    }

    pub fn get_priority(&self) -> Option<u128> {
        self.inner.get_priority()
    }
}

impl<T> Deref for Recovered<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Signed Transaction
/// This is for
#[derive(Debug, Clone)]
pub struct Signed<T, Sig = Signature> {
    tx: T,
    signature: Sig,
    // Only hash for tx, not for (signature + tx)
    hash: OnceLock<TxHash>,
}

impl<T, Sig> Signed<T, Sig> {
    pub fn new_unhashed(tx: T, signature: Sig) -> Self {
        Self {
            tx,
            signature,
            hash: OnceLock::new(),
        }
    }
}

impl<T: SignableTransaction<Signature>> Signed<T> {
    pub fn new(tx: T, signature: Signature, hash: TxHash) -> Self {
        let value = OnceLock::new();
        value.get_or_init(|| hash);
        Self {
            tx,
            signature,
            hash: value,
        }
    }

    pub fn transaction(&self) -> &T {
        &self.tx
    }

    pub const fn signature(&self) -> &Signature {
        &self.signature
    }

    // signature + hash
    pub fn hash(&self) -> TxHash {
        let mut hasher = Sha256::new();
        hasher.update(self.signature.as_bytes());
        hasher.update(self.hash.get().unwrap());
        B256::from_slice(&hasher.finalize())
    }

    pub fn cost(&self) -> U256 {
        if let Some(priority) = self.tx.get_priority() {
            U256::from(priority)
        } else {
            U256::from(0)
        }
    }

    pub fn signature_hash(&self) -> TxHash {
        self.tx.encode_for_signing()
    }
}

impl<T: Transaction> Transaction for Signed<T> {
    fn chain_id(&self) -> ChainId {
        self.tx.chain_id()
    }

    fn nonce(&self) -> u64 {
        self.tx.nonce()
    }

    fn value(&self) -> U256 {
        self.tx.value()
    }

    fn get_priority(&self) -> Option<u128> {
        self.tx.get_priority()
    }
}

impl<T: Encodable> Encodable for Signed<T> {
    fn encode(&self) -> Result<Vec<u8>, crate::error::EncodeError> {
        let tx_arr = self.tx.encode()?;
        let sig_arr = self.signature.as_bytes().to_vec();
        Ok([tx_arr, sig_arr].concat())
    }
}

impl<T: Decodable + SignableTransaction<Signature>> Decodable for Signed<T> {
    fn decode(raw: &Vec<u8>) -> Result<(Self, usize), crate::error::DecodeError> {
        let size = raw.len();
        let (tx, tx_size) = T::decode(&raw)?;

        if size < tx_size + 65 {
            return Err(DecodeError::InputTooShort);
        }

        let sig_raw: [u8; 65] = match raw[tx_size..tx_size + 65].try_into() {
            Ok(arr) => arr,
            Err(err) => return Err(DecodeError::SignatureLengthError(err)),
        };

        let signature = match Signature::from_raw_array(&sig_raw) {
            Ok(sig) => sig,
            Err(_) => return Err(DecodeError::SignatureDecodeError),
        };

        let signed = tx.into_signed(signature);
        Ok((signed, size))
    }
}

impl<T> SignerRecovable for Signed<T> {
    fn recover_signer(&self) -> Result<Address, RecoveryError> {
        let y_parity: u8 = if self.signature.y_parity() { 1 } else { 0 };
        let recid = RecoveryId::from_byte(y_parity).unwrap(); // safe!
        let signature: k256::ecdsa::Signature = self.signature.clone().into();
        let hash = match self.hash.get() {
            Some(hash) => hash,
            None => return Err(RecoveryError::HashGetError),
        };

        let recovered_key = match VerifyingKey::recover_from_digest(
            Sha256::new_with_prefix(hash),
            &signature,
            recid,
        ) {
            Ok(key) => key,
            Err(_) => return Err(RecoveryError::RecoveryFromDigestError),
        };

        let recovered_pubkey_uncompressed: EncodedPoint = recovered_key.to_encoded_point(false);
        let recovered_pubkey_bytes = recovered_pubkey_uncompressed.as_bytes();
        let recovered_address: [u8; 20] = recovered_pubkey_bytes
            [recovered_pubkey_bytes.len() - 20..]
            .try_into()
            .expect("slice is not 20 bytes");

        Ok(Address::from_byte(recovered_address))
    }
}
