// TODO: Address String Should have 20 length bytes!
use core::fmt;
pub use ethnum::U256;

#[derive(Debug, Clone)]
pub struct Address(String);
pub type TxHash = U256;
pub type ChainId = u64;

pub trait Transaction: fmt::Debug + Send + Sync + 'static {
    fn chain_id(&self) -> ChainId;
    fn nonce(&self) -> u64;
    fn value(&self) -> U256;
}

pub trait SignableTransaction<Signature>: Transaction {
    fn set_chain_id(&mut self, chain_id: ChainId);

    fn into_signed(self, signature: Signature) -> Signed
}

