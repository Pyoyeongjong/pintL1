use transaction_pool::traits::TransactionPool;

use crate::components::FullNodeTypes;

pub trait PayloadServiceBuilder<Node, Pool>
where
    Node: FullNodeTypes,
    Pool: TransactionPool,
{
}
