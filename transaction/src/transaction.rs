//! Transactions for PintL1
use k256::{EncodedPoint, ecdsa::VerifyingKey};
use primitives::{
    error::{DecodeError, EncodeError, RecoveryError},
    signed::{Signature, Signed, SignerRecovable},
    transaction::{Decodable, Encodable, SignableTransaction, SignedTransaction},
    types::{Address, B256, ChainId, TxHash, U256},
};
use sha2::{Digest, Sha256};

use crate::PintTx;

// Macro definitions must appear before any macro invocations.
/*
    - `expr`: Represents an expression.
    - `ident`: Represents an identifier (e.g., a variable or method name).
    - `tx`: A temporary variable name used in the enum payload.
    - `method`: Refers to a method name (e.g., `chain_id()`).
    - `$($arg:expr),*`: Captures one or more expressions as `args`, separated by commas.
    - `*`: Means "zero or more" repetitions.
    - `+`: Means "one or more" repetitions.
*/
macro_rules! delegate {
    ($self:expr => $tx:ident.$method:ident($($arg:expr),*)) => {
        match $self {
            Transaction::Pint($tx) => $tx.$method($($arg),*),
        }
    };
}

/// Transactions for PintL1 in enum types
#[derive(Debug, Clone)]
pub enum Transaction {
    Pint(PintTx),
}

impl primitives::Transaction for Transaction {
    fn chain_id(&self) -> ChainId {
        delegate!(self => tx.chain_id())
    }
    fn nonce(&self) -> u64 {
        delegate!(self => tx.nonce())
    }
    fn value(&self) -> U256 {
        delegate!(self => tx.value())
    }

    fn get_priority(&self) -> Option<u128> {
        delegate!(self => tx.get_priority())
    }
}

impl Encodable for Transaction {
    fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let (tid, tx_data): (u8, Vec<u8>) = match self {
            Transaction::Pint(pint_tx) => (0, pint_tx.encode()?),
        };

        let arr = vec![tid];
        let res = [arr, tx_data].concat();
        Ok(res)
    }
}

impl Decodable for Transaction {
    fn decode(vec: &Vec<u8>) -> Result<(Self, usize), DecodeError> {
        let tx_type = vec[0];
        match tx_type {
            0 => {
                let (pint_tx, size) = PintTx::decode(vec)?;
                Ok((Transaction::Pint(pint_tx), size))
            }
            _ => Err(DecodeError::InvalidTxType),
        }
    }
}

// This is a concrete type implementation.
// `impl<T> Trait<T>` is a generic implementation.
impl SignableTransaction<Signature> for Transaction {
    fn into_signed(self, signature: Signature) -> Signed<Self> {
        let tx_hash = delegate!(self.clone() => tx.encode_for_signing());
        Signed::new(self, signature, tx_hash)
    }

    fn encode_for_signing(&self) -> B256 {
        delegate!(self => tx.encode_for_signing())
    }
}

// From Transaction to enum Transaction
pub trait IntoTransaction {
    fn into_transaction(self) -> Transaction;
}

impl Transaction {
    pub fn from<T: IntoTransaction>(tx: T) -> Self {
        tx.into_transaction()
    }
}

/// Transactions for Signed PintL1 in enum types
#[derive(Debug, Clone)]
pub enum TxEnvelope {
    Pint(Signed<PintTx>),
}

impl TxEnvelope {
    pub fn tx_type(&self) -> u8 {
        match self {
            TxEnvelope::Pint(signed_tx) => signed_tx.transaction().tx_type(),
        }
    }
    // Identification Role of SignedTx
    pub fn hash(&self) -> TxHash {
        match self {
            TxEnvelope::Pint(signed_tx) => signed_tx.hash(),
        }
    }

    pub fn cost(&self) -> U256 {
        match self {
            TxEnvelope::Pint(signed_tx) => signed_tx.cost(),
        }
    }

    pub fn signature_hash(&self) -> TxHash {
        match self {
            TxEnvelope::Pint(signed_tx) => signed_tx.signature_hash(),
        }
    }

    pub fn signature(&self) -> &Signature {
        match self {
            TxEnvelope::Pint(signed_tx) => signed_tx.signature(),
        }
    }
}

impl Encodable for TxEnvelope {
    fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        match self {
            TxEnvelope::Pint(signed_tx) => signed_tx.encode(),
        }
    }
}

impl Decodable for TxEnvelope {
    fn decode(data: &Vec<u8>) -> Result<(Self, usize), DecodeError> {
        let (tx, _) = Signed::<Transaction>::decode(data)?;
        match tx.transaction() {
            Transaction::Pint(pint_tx) => Ok((
                TxEnvelope::Pint(Signed::new(
                    pint_tx.clone(),
                    tx.signature().clone(),
                    tx.hash(),
                )),
                0,
            )),
        }
    }
}

impl primitives::Transaction for TxEnvelope {
    fn chain_id(&self) -> ChainId {
        match self {
            TxEnvelope::Pint(signed_tx) => signed_tx.chain_id(),
        }
    }

    fn nonce(&self) -> u64 {
        match self {
            TxEnvelope::Pint(signed_tx) => signed_tx.nonce(),
        }
    }

    fn value(&self) -> U256 {
        match self {
            TxEnvelope::Pint(signed_tx) => signed_tx.value(),
        }
    }

    fn get_priority(&self) -> Option<u128> {
        match self {
            TxEnvelope::Pint(signed_tx) => signed_tx.get_priority(),
        }
    }
}

impl SignedTransaction for TxEnvelope {
    fn tx_hash(&self) -> TxHash {
        self.hash()
    }
}

impl SignerRecovable for TxEnvelope {
    fn recover_signer(&self) -> Result<Address, primitives::error::RecoveryError> {
        let signature_hash: TxHash = self.signature_hash();
        let signature = self.signature().clone();

        let recid = match signature.get_recovery_id() {
            Some(recid) => recid,
            None => return Err(RecoveryError::RecIdError),
        };

        let recover_signature = signature.into();

        let recovered_key = match VerifyingKey::recover_from_digest(
            Sha256::new_with_prefix(signature_hash),
            &recover_signature,
            recid,
        ) {
            Ok(rec_key) => rec_key,
            Err(_) => return Err(RecoveryError::RecKeyError),
        };

        let recovered_pubkey_uncompressed: EncodedPoint = recovered_key.to_encoded_point(false);
        let recovered_pubkey_bytes = recovered_pubkey_uncompressed.as_bytes();
        let recovered_address = &recovered_pubkey_bytes[recovered_pubkey_bytes.len() - 20..];

        Ok(Address::from_hex(hex::encode(recovered_address))?)
    }
}

#[cfg(test)]
mod tests {
    use k256::ecdsa::{RecoveryId, Signature as ECDSASig, SigningKey};
    use rand::Rng;

    use super::*;

    fn get_priv_pub_key(seed: &[u8]) -> (SigningKey, Vec<u8>) {
        let private_key_random = Sha256::digest(&seed);
        let signing_key = SigningKey::from_bytes(&private_key_random).unwrap();
        let verifying_key = signing_key.clone().verifying_key().clone();
        let pubkey_uncompressed: EncodedPoint = verifying_key.to_encoded_point(false);
        let pubkey_bytes = pubkey_uncompressed.as_bytes();
        let address = pubkey_bytes[pubkey_bytes.len() - 20..].to_vec();
        (signing_key, address)
    }

    #[test]
    fn test_transaction_encode_and_decode_transaction() {
        let (signing_key, sender) = get_priv_pub_key("hello".as_bytes());
        let sender = Address::from_byte(sender.try_into().unwrap());
        dbg!(&sender.get_addr_hex());
        let (_, receiver) = get_priv_pub_key("wow".as_bytes());
        let receiver = Address::from_byte(receiver.try_into().unwrap());
        dbg!(receiver.get_addr_hex());
        let pint_tx = PintTx {
            chain_id: 0,
            nonce: 0,
            to: receiver,
            fee: 0,
            value: U256::from(1),
        };

        let tx = Transaction::Pint(pint_tx);
        let tx_hash = tx.encode_for_signing();
        let digest = Sha256::new_with_prefix(tx_hash);
        let (signature, recid): (ECDSASig, RecoveryId) =
            signing_key.sign_digest_recoverable(digest).unwrap();

        let sig = Signature::from_sig(signature, recid);

        let signed = Signed::<Transaction>::new(tx, sig, tx_hash);

        let encoded = signed.encode().unwrap();
        dbg!(hex::encode(&encoded));

        let (recovered_signed, _) = Signed::<Transaction>::decode(&encoded).unwrap();
        let recovered_sender = recovered_signed.recover_signer().unwrap();

        assert_eq!(sender, recovered_sender);
    }
}
