use transaction_pool::{
    Pool,
    ordering::PintOrdering,
    traits::{PintPooledTransaction, TransactionPool},
    validate::{pint::PintTransactionValidator, task::TransactionValidationTaskExecutor},
};

use crate::{
    builder::ComponentsBuilder,
    components::{
        FullNodeTypes, consensus::ConsensusBuilder, execute::ExecutorBuilder,
        network::NetworkBuilder, payload::PayloadServiceBuilder, pool::PoolBuilder,
    },
};

#[derive(Default)]
pub struct PintNode;

impl PintNode {
    pub fn components<Node>() -> ComponentsBuilder<
        Node,
        PintPoolBuilder,
        PintPayloadServiceBuilder,
        PintNetworkBuilder,
        PintExecutorBuilder,
        PintConsensusBuilder,
    >
    where
        Node: FullNodeTypes,
    {
        ComponentsBuilder::new(
            PintPoolBuilder::default(),
            PintPayloadServiceBuilder::default(),
            PintNetworkBuilder::default(),
            PintExecutorBuilder::default(),
            PintConsensusBuilder::default(),
        )
    }
}

#[derive(Default)]
pub struct PintPoolBuilder;

impl<Node> PoolBuilder<Node> for PintPoolBuilder
where
    Node: FullNodeTypes,
{
    type Pool = Pool<
        TransactionValidationTaskExecutor<
            PintTransactionValidator<Node::Provider, PintPooledTransaction>,
        >,
        PintOrdering<PintPooledTransaction>,
    >;

    async fn build_pool(self) -> Self::Pool {
        let validator = TransactionValidationTaskExecutor::new(PintTransactionValidator::defalut());

        let transaction_pool = Pool::new(validator, ordering, config);
    }
}

#[derive(Default)]
pub struct PintPayloadServiceBuilder;

impl<Node, Pool> PayloadServiceBuilder<Node, Pool> for PintPayloadServiceBuilder
where
    Node: FullNodeTypes,
    Pool: TransactionPool,
{
}

#[derive(Default)]
pub struct PintNetworkBuilder;

impl<Node, Pool> NetworkBuilder<Node, Pool> for PintNetworkBuilder
where
    Node: FullNodeTypes,
    Pool: TransactionPool,
{
}

#[derive(Default)]
pub struct PintExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for PintExecutorBuilder where Node: FullNodeTypes {}

#[derive(Default)]
pub struct PintConsensusBuilder;

impl<Node> ConsensusBuilder<Node> for PintConsensusBuilder where Node: FullNodeTypes {}
