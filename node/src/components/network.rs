use transaction_pool::traits::TransactionPool;

use crate::{components::FullNodeTypes, error::BuildError};

pub trait NetworkBuilder<Node: FullNodeTypes, Pool: TransactionPool> {
    type Network;

    fn build_network(self, pool: Pool) -> impl Future<Output = Result<Self::Network, BuildError>>;
}
