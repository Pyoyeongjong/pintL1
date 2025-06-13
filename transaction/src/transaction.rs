use std::sync::OnceLock;
use primitives::types::{TransactionSigned, TxHash};

use primitives::{
    types::{ChainId, U256, Signature},
    transaction::{SignableTransaction, Encodable},
};

use crate::PintTx;

// 매크로 정의는 반드시 매크로 호출보다 먼저 위치해야 한다.
/*
    expr: 표현식 역할을 해야 한다. ident: 식별자 역할을 해야 한다
    tx는 enum payload에 임시로 붙인 변수 이름
    method는 메소드 이름 ex) chain_id()
    $($arg:expr),*: 표현식 하나를 args 라는 이름으로 캡쳐, ,* 쉼표로 반복하겠다 라는 뜻.
    *: 0번 이상, +: 1번 이상 반복한다는 뜻
*/
macro_rules! delegate {
    ($self:expr => $tx:ident.$method:ident($($arg:expr),*)) => {
        match $self {
            Transaction::Pint($tx) => $tx.$method($($arg),*),
        }
    };
}

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
}


// 이건 구체 타입 impl
// impl<T> trait<T> 이게 제너릭 impl
impl SignableTransaction<Signature> for Transaction {
    fn into_signed(self, signature: Signature) -> TransactionSigned<Self>{
        let tx_hash = delegate!(self.clone() => tx.tx_hash(&signature));
        TransactionSigned::new(self, signature, tx_hash)
    }
}

pub trait IntoTransaction {
    fn into_transaction(self) -> Transaction;
}

impl Transaction {
    pub fn from<T: IntoTransaction>(tx: T) -> Self {
        tx.into_transaction()
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use primitives::types::Address;

    use super::*;

    #[test]
    fn test_into_signed() {
        let ptx = PintTx {
            chain_id: 0,
            nonce: 0,
            to: Address::new("deadbeef".to_string()),
            value: U256::from(1)
        };

        let tx: Transaction = Transaction::from::<PintTx>(ptx);
        let signature = Signature::from_str("48b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c8041b").unwrap();
        let signed_tx = tx.into_signed(signature);

        println!("{:?}", signed_tx);
    }
}

