use std::{marker::PhantomData, net::IpAddr, sync::Arc};

use storage::{db::InMemoryDB, PintStateProviderFactory};

use crate::{components::{
        consensus::ConsensusBuilder, execute::ExecutorBuilder, network::NetworkBuilder, payload::PayloadServiceBuilder, pool::PoolBuilder, FullNodeTypes, NodeComponentsBuilder
    }, error::{BuildError, LaunchError}, node::Components};

#[derive(Debug)]
pub struct ComponentsBuilder<Node, PoolB, PayloadB, NetworkB, ExecB, ConsB> {
    pool_builder: PoolB,
    payload_builder: PayloadB,
    network_builder: NetworkB,
    executor_builder: ExecB,
    consensus_builder: ConsB,
    _marker: PhantomData<Node>,
}

impl<Node, PoolB, PayloadB, NetworkB, ExecB, ConsB>
    ComponentsBuilder<Node, PoolB, PayloadB, NetworkB, ExecB, ConsB>
where
    Node: FullNodeTypes,
    PoolB: PoolBuilder<Node>,
    PayloadB: PayloadServiceBuilder<Node, PoolB::Pool>,
    NetworkB: NetworkBuilder<Node, PoolB::Pool>,
    ExecB: ExecutorBuilder<Node>,
    ConsB: ConsensusBuilder<Node>,
{
    pub fn new<Types>(
        pool_builder: PoolB,
        payload_builder: PayloadB,
        network_builder: NetworkB,
        executor_builder: ExecB,
        consensus_builder: ConsB,
    ) -> Self where Types: FullNodeTypes{
        Self {
            pool_builder,
            payload_builder,
            network_builder,
            executor_builder,
            consensus_builder,
            _marker: Default::default()
        }
    }
}

impl<Node, PoolB, PayloadB, NetworkB, ExecB, ConsB> NodeComponentsBuilder for ComponentsBuilder<Node, PoolB, PayloadB, NetworkB, ExecB, ConsB> 
where 
    Node: FullNodeTypes, 
    PoolB: PoolBuilder<Node>,
    PayloadB: PayloadServiceBuilder<Node, PoolB::Pool>,
    NetworkB: NetworkBuilder<Node, PoolB::Pool>,
    ExecB: ExecutorBuilder<Node>,
    ConsB: ConsensusBuilder<Node>
{

    type Components = Components<Node, NetworkB::Network, PoolB::Pool, ExecB::Exec, ConsB::Consensus>;
    type Provider = Node::Provider;
    async fn build_components(self, provider: Self::Provider) -> Result<Self::Components, BuildError> {
        let Self {
            pool_builder,
            payload_builder,
            network_builder,
            executor_builder,
            consensus_builder,
            _marker
        } = self;

        let exec_config = executor_builder.build_exec().await?;
        let transaction_pool = pool_builder.build_pool(provider.clone()).await?;
        let network = network_builder.build_network(transaction_pool.clone()).await?;
        let payload_builder_handle = payload_builder.spawn_payload_builder_service(transaction_pool.clone(), provider.clone()).await?;
        let consensus = consensus_builder.build_consensus().await?;

        Ok(Components{
            transaction_pool,
            executor: exec_config,
            network,
            payload_builder: payload_builder_handle,
            consensus
        })
    }
}

pub struct LaunchContext<CB> {
    pub address: IpAddr,
    pub port: u16,
    pub components_builder: CB,
}

impl<CB> LaunchContext<CB> where CB: NodeComponentsBuilder<Provider = PintStateProviderFactory<Arc<InMemoryDB>>> {
    pub async fn launch(self) -> Result<(), LaunchError>{
        // Here you would implement the logic to launch the node using the provided components builder.
        // This is a placeholder for the actual launch logic.
        println!("Launching node at {}:{}", self.address, self.port);

        // making database
        let database = Arc::new(InMemoryDB::new());
        // making providerFactory
        let provider = PintStateProviderFactory::new(database);
        // build_components
        let components = self.components_builder.build_components(provider).await?;

        println!("Launching node OK");
        Ok(())
    }
}
