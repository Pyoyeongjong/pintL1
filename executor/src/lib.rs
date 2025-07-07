pub mod error;
pub mod traits;
pub mod transaction;

use primitives::types::TxHash;
use storage::traits::StateProvider;

use crate::{
    error::BlockExecutionError, traits::BlockExecutor, transaction::ExecutableTranasction,
};

/// Transaction executor
pub struct PintBlockExecutor<SP> {
    state: SP,
    receipts: Vec<Receipt>,
}

impl<SP: StateProvider> BlockExecutor for PintBlockExecutor<SP>
where
    SP: StateProvider,
{
    type Transaction = ExecutableTranasction;

    fn execute_transaction(
        &mut self,
        tx: Self::Transaction,
    ) -> Result<Option<u64>, BlockExecutionError> {
        todo!()
    }
}

pub struct Receipt {
    tx_hash: TxHash,
    success: bool,
}
