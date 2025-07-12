use transaction_pool::traits::PoolTransaction;

use crate::error::BlockExecutionError;

pub trait ExecutableTx {
    fn from_pool_transaction<Tx: PoolTransaction>(tx: Tx) -> Self;
}

pub trait BlockExecutor {
    type Transaction: ExecutableTx;
    fn execute_transaction(
        &mut self,
        tx: &Self::Transaction,
    ) -> Result<Option<u64>, BlockExecutionError>;

    fn execute_and_commit(
        &mut self,
        tx: &Self::Transaction,
    ) -> Result<Option<u64>, BlockExecutionError>;
}
