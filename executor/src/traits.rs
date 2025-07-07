use crate::error::BlockExecutionError;

pub trait ExecutableTx {}

pub trait BlockExecutor {
    type Transaction: ExecutableTx;
    fn execute_transaction(
        &mut self,
        tx: Self::Transaction,
    ) -> Result<Option<u64>, BlockExecutionError>;
}
