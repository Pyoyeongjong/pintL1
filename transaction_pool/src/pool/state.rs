//! Implements [TxState] and [SubPool]
// TODO: It can be canged to bitflags using bitflag! macro library!
// refer reth/transaction_pool/pool/state.rs TxState
#[derive(Default, Clone, Copy)]
pub struct TxState {
    has_fee: bool,
    has_ancestor: bool,
}

impl TxState {
    pub fn has_fee(&mut self) {
        self.has_fee = true;
    }

    pub fn has_no_fee(&mut self) {
        self.has_fee = false;
    }

    pub fn has_ancestor(&mut self) {
        self.has_ancestor = true;
    }

    pub fn has_no_ancestor(&mut self) {
        self.has_ancestor = false
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SubPool {
    Pending,
    Parked,
}

impl SubPool {
    pub const fn is_pending(&self) -> bool {
        match self {
            SubPool::Pending => true,
            _ => false,
        }
    }
}

impl From<TxState> for SubPool {
    fn from(value: TxState) -> Self {
        match value.has_fee && !value.has_ancestor {
            true => SubPool::Pending,
            false => SubPool::Parked,
        }
    }
}
