use core::fmt;
use crate::types::{TransactionSigned, ChainId, U256, TxHash};

pub trait Transaction: fmt::Debug + Send + Sync + 'static {
    fn chain_id(&self) -> ChainId;
    fn nonce(&self) -> u64;
    fn value(&self) -> U256;
}

// A signable transaction/
pub trait SignableTransaction<Signature>: Transaction {
    // Convert to a ['Signed'] Object
    fn into_signed(self, signature: Signature) -> TransactionSigned<Self, Signature> where Self: Sized;
}

pub trait Encodable<Signature>: Transaction {
    fn tx_hash(self, signature: &Signature) -> TxHash;
}
