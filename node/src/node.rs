use std::{alloc::handle_alloc_error, marker::PhantomData, sync::Arc};

use consensus::PintConsensus;
use executor::PintBlockExecutor;
use net::PintNetworkHandle;
use payload::{builder::PayloadBuilderHandle, traits::PayloadTypes, PintPayloadTypes};
use storage::{db::{Database, InMemoryDB}, PintStateProviderFactory};
use transaction_pool::{
    config::PoolConfig, ordering::PintOrdering, traits::{PintPooledTransaction, TransactionPool}, validate::{pint::{PintTransactionValidator, PintTransactionValidatorBuilder}, task::TransactionValidationTaskExecutor}, Pool
};

use crate::{
    builder::ComponentsBuilder,
    components::{
        consensus::ConsensusBuilder, execute::ExecutorBuilder, network::NetworkBuilder, payload::PayloadServiceBuilder, pool::PoolBuilder, FullNodeTypes
    }, error::BuildError,
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
        ComponentsBuilder::new::<Node>(
            PintPoolBuilder::default(),
            PintPayloadServiceBuilder::default(),
            PintNetworkBuilder::default(),
            PintExecutorBuilder::default(),
            PintConsensusBuilder::default(),
        )
    }
}


impl FullNodeTypes for PintNode {
    type Provider = PintStateProviderFactory<Arc<InMemoryDB>>;
    type Payload = PintPayloadTypes;
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

    async fn build_pool(self, provider: Node::Provider) -> Result<Self::Pool, BuildError> {

        let validator = PintTransactionValidatorBuilder::new(provider).build();

        let validator_task_executor = TransactionValidationTaskExecutor::new(validator);
        let transaction_pool = Pool::new(validator_task_executor, PintOrdering::default(), PoolConfig::default());
        
        Ok(transaction_pool)
    }
}

#[derive(Default)]
pub struct PintPayloadServiceBuilder;

impl<Node, Pool> PayloadServiceBuilder<Node, Pool> for PintPayloadServiceBuilder
where
    Node: FullNodeTypes,
    Pool: TransactionPool,
{
    async fn spawn_payload_builder_service(self, pool: Pool, provider: Node::Provider) -> Result<PayloadBuilderHandle<<Node as FullNodeTypes>::Payload>, BuildError> {
        todo!()
    }
}

#[derive(Default)]
pub struct PintNetworkBuilder;

impl<Node, Pool> NetworkBuilder<Node, Pool> for PintNetworkBuilder
where
    Node: FullNodeTypes,
    Pool: TransactionPool,
{
    type Network = PintNetworkHandle;
    
    async fn build_network(self, pool: Pool) -> Result<Self::Network, BuildError> {
        todo!()
    }
}

#[derive(Default)]
pub struct PintExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for PintExecutorBuilder where Node: FullNodeTypes {
    type Exec = PintBlockExecutor<InMemoryDB>;
    
    async fn build_exec(self) -> Result<Self::Exec, BuildError> {
        todo!()
    }
}

#[derive(Default)]
pub struct PintConsensusBuilder;

impl<Node> ConsensusBuilder<Node> for PintConsensusBuilder where Node: FullNodeTypes {
    type Consensus = PintConsensus;
    
    async fn build_consensus(self) -> Result<Self::Consensus, BuildError> {
        todo!()
    }
}


pub struct Components<Node: FullNodeTypes, Network, Pool, Exec, Consensus> {
    pub transaction_pool: Pool,
    pub executor: Exec,
    pub consensus: Consensus,
    pub network: Network,
    pub payload_builder: PayloadBuilderHandle<Node::Payload>
}