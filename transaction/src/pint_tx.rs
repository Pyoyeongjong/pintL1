use primitives::{
    transaction::Encodable, types::{Address, ChainId, Signature, TxHash, B256, U256}
};
use sha2::{Digest, Sha256};

use crate::transaction::IntoTransaction;

#[derive(Debug, Clone)]
pub struct PintTx {
    pub chain_id: ChainId,
    pub nonce: u64,
    pub to: Address,
    pub value: U256,
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
}

impl Encodable<Signature> for PintTx {
    fn tx_hash(self, signature: &Signature) -> TxHash {
        let mut hasher = Sha256::new();
        hasher.update(self.chain_id.to_string().as_bytes());
        hasher.update(self.nonce.to_string().as_bytes());
        hasher.update(self.to.get_addr().as_bytes());
        hasher.update(self.value.to_string().as_bytes());
        hasher.update(signature.as_bytes());
        B256::from_slice(&hasher.finalize())
    }
}

impl IntoTransaction for PintTx {
    fn into_transaction(self) -> crate::transaction::Transaction {
        crate::transaction::Transaction::Pint(self)
    }
}

#[cfg(test)] 
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_pint_tx_hash_and_convert_signed() {
        let ptx = PintTx {
            chain_id: 0,
            nonce: 0,
            to: Address::new("deadbeef".to_string()),
            value: U256::from(1)
        };

        let signature = Signature::from_str("48b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c8041b").unwrap();
        let hash = ptx.tx_hash(&signature);

        println!("{:?}", hash);
    }
}