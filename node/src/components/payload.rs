use payload::builder::PayloadBuilderHandle;
use transaction_pool::traits::TransactionPool;

use crate::{components::FullNodeTypes, error::BuildError};

pub trait PayloadServiceBuilder<Node, Pool>
where
    Node: FullNodeTypes,
    Pool: TransactionPool,
{
    fn spawn_payload_builder_service(self, pool: Pool, provider: Node::Provider) -> impl Future<Output = Result<PayloadBuilderHandle<Node::Payload>, BuildError>>;
}
