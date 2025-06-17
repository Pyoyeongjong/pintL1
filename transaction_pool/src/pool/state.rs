// TODO: It can be canged to bitflags using bitflag! macro library! 
// refer reth/transaction_pool/pool/state.rs TxState
#[derive(Default, Clone, Copy)]
pub struct TxState {
    zero_fee: bool
}

impl TxState {
    pub fn has_fee(&mut self) {
        self.zero_fee = false;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SubPool{
    Pending,
    Parked
}

impl SubPool {
    pub const fn is_pending(&self) -> bool {
        match self {
            SubPool::Pending => true,
            _ => false
        }
    }
}

impl From<TxState> for SubPool {
    fn from(value: TxState) -> Self {
        match value.zero_fee {
            true => SubPool::Parked,
            false => SubPool::Pending
        }
    }
}