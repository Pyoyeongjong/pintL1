//! PintTx
//! [PintTx] is a representative transaction for this PintL1 Project.
use primitives::{
    error::{DecodeError, EncodeError},
    signed::Signature,
    transaction::{Decodable, Encodable, SignableTransaction},
    types::{Address, B256, ChainId, TxHash, U256},
};
use sha2::{Digest, Sha256};

use crate::transaction::IntoTransaction;

/// PintTx
#[derive(Debug, Clone)]
pub struct PintTx {
    pub chain_id: ChainId, // 8
    pub nonce: u64,        // 8
    pub to: Address,       // 20
    pub fee: u128,         // 16
    pub value: U256,       // 32
}

impl PintTx {
    fn size() -> usize {
        let size = size_of::<ChainId>()
            + size_of::<u64>()
            + size_of::<Address>()
            + size_of::<u128>()
            + size_of::<U256>();
        size
    }
}

impl primitives::Transaction for PintTx {
    fn chain_id(&self) -> ChainId {
        self.chain_id
    }
    fn nonce(&self) -> u64 {
        self.nonce
    }
    fn value(&self) -> U256 {
        self.value
    }

    fn get_priority(&self) -> Option<u128> {
        Some(self.fee)
    }
}

impl Encodable for PintTx {
    fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let mut arr: [u8; 84] = [0u8; 84];

        arr[0..8].copy_from_slice(&self.chain_id.to_be_bytes());
        arr[8..16].copy_from_slice(&self.nonce.to_be_bytes());
        arr[16..36].copy_from_slice(&self.to.get_addr());
        arr[36..52].copy_from_slice(&self.fee.to_be_bytes());
        arr[52..].copy_from_slice(&self.value.to_be_bytes::<32>());

        Ok(arr.to_vec())
    }
}

impl Decodable for PintTx {
    fn decode(data: &Vec<u8>) -> Result<(Self, usize), primitives::error::DecodeError> {
        let raw: [u8; 84] = match data[1..85].try_into() {
            Ok(arr) => arr,
            Err(_) => return Err(DecodeError::InputTooShort),
        };

        let chain_id: ChainId = ChainId::from_be_bytes(raw[0..8].try_into()?);
        let nonce: u64 = u64::from_be_bytes(raw[8..16].try_into()?);
        let to = match Address::from_hex(hex::encode(&raw[16..36])) {
            Ok(addr) => addr,
            Err(_) => return Err(DecodeError::InvalidAddress),
        };
        let fee: u128 = u128::from_be_bytes(raw[36..52].try_into()?);
        let value: U256 = U256::from_be_bytes::<32>(raw[52..84].try_into()?);

        Ok((
            Self {
                chain_id,
                nonce,
                to,
                fee,
                value,
            },
            85 as usize,
        ))
    }
}

impl IntoTransaction for PintTx {
    fn into_transaction(self) -> crate::transaction::Transaction {
        crate::transaction::Transaction::Pint(self)
    }
}

impl SignableTransaction<Signature> for PintTx {
    fn into_signed(self, signature: Signature) -> primitives::signed::Signed<Self, Signature>
    where
        Self: Sized,
    {
        todo!()
    }

    fn encode_for_signing(&self) -> TxHash {
        let mut hasher = Sha256::new();
        hasher.update(self.chain_id.to_string().as_bytes());
        hasher.update(self.nonce.to_string().as_bytes());
        hasher.update(self.to.get_addr());
        hasher.update(self.value.to_string().as_bytes());
        B256::from_slice(&hasher.finalize())
    }
}
