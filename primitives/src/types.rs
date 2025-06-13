use std::{str::FromStr, sync::OnceLock};
use hex;
pub use alloy_primitives::{B256, U256};

use crate::{utils::normalize_v, SignatureError};
pub type TxHash = B256;
pub type BlockHash = B256;
pub type ChainId = u64;

// TODO: Address String Should have 20 length bytes!
#[derive(Debug, Clone)]
pub struct Address(String);

impl Address {

    pub fn new(address: String) -> Address {
        Address(address)
    }

    pub fn get_addr(&self) -> String {
        self.0.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Signature {
    y_parity: bool,
    r: U256,
    s: U256,
}

impl FromStr for Signature {
    type Err = SignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = [0u8; 65];
        hex::decode_to_slice(s, &mut out)?;
        Self::from_raw_array(&mut out)
    }
}

impl Signature {

    pub fn from_raw_array(bytes: &[u8; 65]) -> Result<Self, SignatureError> {
        // Binding front array except the last one in byets
        let [bytes @.., v] = bytes;
        let v = *v as u64;
        let Some(parity) = normalize_v(v) else { return Err(SignatureError::InvalidParity(v))};
        Ok(Self::from_bytes_and_parity(bytes, parity))
    }

    pub fn from_bytes_and_parity(bytes: &[u8], parity: bool) -> Self{
        let mut r_arr = [0u8; 32];
        let mut s_arr = [0u8; 32];

        let (r_bytes, s_bytes) = bytes[..64].split_at(32);
        r_arr.copy_from_slice(r_bytes);
        s_arr.copy_from_slice(s_bytes);

        let r = U256::from_be_bytes(r_arr);
        let s = U256::from_be_bytes(r_arr);
        Self { y_parity: parity, r, s}
    }

    pub fn as_bytes(&self) -> [u8; 65] {
        let mut sig = [0u8; 65];
        sig[..32].copy_from_slice(&self.r.to_be_bytes::<32>());
        sig[32..64].copy_from_slice(&self.s.to_be_bytes::<32>());
        sig[64] = self.y_parity as u8;
        sig
    }
}

#[derive(Debug)]
// Sig을 생략하면 기본으로 Signature를 사용한다는 뜻
pub struct TransactionSigned<T, Sig = Signature> {
    tx: T,
    signature: Sig,
    hash: OnceLock<String>,
}

impl<T, Sig> TransactionSigned<T, Sig> {
    pub fn new(tx: T, signature: Sig, hash: TxHash) -> Self {
        let value = OnceLock::new();
        value.get_or_init(|| hash.to_string());
        Self { tx, signature, hash: value }
    }
}
