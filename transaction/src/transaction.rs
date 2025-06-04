use core::{fmt, mem};
use primitives::{Address, ChainId, TxHash, U256};
use std::{io::Chain, sync::OnceLock};

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

pub struct Signature {
    y_parity: bool,
    r: U256,
    s: U256,
}

pub struct TransactionSigned {
    hash: OnceLock<TxHash>,
    signature: Signature,
    transaction: Transaction,
}
