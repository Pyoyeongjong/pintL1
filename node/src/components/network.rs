use transaction_pool::traits::TransactionPool;

use crate::components::FullNodeTypes;

pub trait NetworkBuilder<Node: FullNodeTypes, Pool: TransactionPool> {}
