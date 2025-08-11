use primitives::block::traits::Block;

use transaction_pool::traits::PoolTransaction;

use crate::{BlockBuilderOutcome, error::BlockExecutionError};

/// Executable Tx Trait
pub trait ExecutableTx {
    fn from_pool_transaction<Tx: PoolTransaction>(tx: Tx) -> Self;
}

/// BlockExecutor Trait
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

    fn finish<B: Block>(&self) -> Result<BlockBuilderOutcome<B>, BlockExecutionError>;
}
