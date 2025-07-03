//! Signature-related Traits or Structs
use std::{ops::Deref, str::FromStr, sync::OnceLock};

use alloy_primitives::{B256, U256};
use k256::{
    EncodedPoint,
    ecdsa::{RecoveryId, VerifyingKey},
    sha2::{Digest, Sha256},
};

use crate::{
    SignatureError, Transaction,
    error::{DecodeError, RecoveryError},
    normalize_v,
    transaction::{Decodable, Encodable, SignableTransaction},
    types::{Address, ChainId, TxHash},
};

/// A trait for recovering public key from a signature.
pub trait SignerRecovable {
    fn recover_signer(&self) -> Result<Address, RecoveryError>;
}

/// ESDCA Signature
#[derive(Debug, Clone)]
pub struct Signature {
    // hint for recovery
    y_parity: bool,
    // randomly created k and is the x-coordinate of the curve point R = k*G
    r: U256,
    // r + signer's private key + message hash + k, r
    // s = k^-1 (z + r*d) mod n
    s: U256,
}

impl Signature {
    pub fn get_recovery_id(&self) -> Option<RecoveryId> {
        let recid = RecoveryId::from_byte(self.y_parity as u8)?;
        Some(recid)
    }
    pub fn from_raw_array(bytes: &[u8; 65]) -> Result<Self, SignatureError> {
        // Binding front array except the last one in byets
        let [bytes @ .., v] = bytes;
        let v = *v as u64;
        let Some(parity) = normalize_v(v) else {
            return Err(SignatureError::InvalidParity(v));
        };
        Ok(Self::from_bytes_and_parity(bytes, parity))
    }

    pub fn from_bytes_and_parity(bytes: &[u8], parity: bool) -> Self {
        let mut r_arr = [0u8; 32];
        let mut s_arr = [0u8; 32];

        let (r_bytes, s_bytes) = bytes[..64].split_at(32);
        r_arr.copy_from_slice(r_bytes);
        s_arr.copy_from_slice(s_bytes);

        let r = U256::from_be_bytes(r_arr);
        let s = U256::from_be_bytes(s_arr);
        Self {
            y_parity: parity,
            r,
            s,
        }
    }

    pub fn as_bytes(&self) -> [u8; 65] {
        let mut sig = [0u8; 65];
        sig[..32].copy_from_slice(&self.r.to_be_bytes::<32>());
        sig[32..64].copy_from_slice(&self.s.to_be_bytes::<32>());
        sig[64] = self.y_parity as u8;
        sig
    }

    pub fn from_sig(signature: k256::ecdsa::Signature, recid: RecoveryId) -> Self {
        let r: [u8; 32] = signature.r().to_bytes().into();
        let s: [u8; 32] = signature.s().to_bytes().into();

        let r = U256::from_be_bytes(r);
        let s = U256::from_be_bytes(s);
        let y_parity = if recid.to_byte() != 0 { true } else { false };

        Self { y_parity, r, s }
    }
}

impl FromStr for Signature {
    type Err = SignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = [0u8; 65];
        hex::decode_to_slice(s, &mut out)?;
        Self::from_raw_array(&mut out)
    }
}

impl Into<k256::ecdsa::Signature> for Signature {
    fn into(self) -> k256::ecdsa::Signature {
        let r_bytes: [u8; 32] = self.r.to_be_bytes();
        let s_bytes: [u8; 32] = self.s.to_be_bytes();

        let mut sig_bytes: [u8; 64] = [0u8; 64];
        sig_bytes[0..32].copy_from_slice(&r_bytes);
        sig_bytes[32..64].copy_from_slice(&s_bytes);

        let sig = k256::ecdsa::Signature::from_slice(&sig_bytes).unwrap();
        sig
    }
}

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
        let y_parity: u8 = if self.signature.y_parity { 1 } else { 0 };
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
