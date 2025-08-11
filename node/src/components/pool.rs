use transaction_pool::traits::TransactionPool;

use crate::components::FullNodeTypes;

pub trait PoolBuilder<Node: FullNodeTypes> {
    type Pool: TransactionPool;

    fn build_pool(self) -> impl Future<Output = Self::Pool>;
}
