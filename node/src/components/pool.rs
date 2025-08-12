use transaction_pool::traits::TransactionPool;

use crate::{components::FullNodeTypes, error::BuildError};

pub trait PoolBuilder<Node: FullNodeTypes> {
    type Pool: TransactionPool;

    fn build_pool(self, provider: Node::Provider) -> impl Future<Output = Result<Self::Pool, BuildError>>;
}
