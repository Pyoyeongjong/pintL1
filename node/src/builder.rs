use std::{marker::PhantomData, net::IpAddr};

use crate::{
    components::{
        FullNodeTypes, consensus::ConsensusBuilder, execute::ExecutorBuilder,
        network::NetworkBuilder, payload::PayloadServiceBuilder, pool::PoolBuilder,
    },
    node::PintNode,
};

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
    pub fn new(
        pool_builder: PoolB,
        payload_builder: PayloadB,
        network_builder: NetworkB,
        executor_builder: ExecB,
        consensus_builder: ConsB,
    ) -> Self {
        Self {
            pool_builder,
            payload_builder,
            network_builder,
            executor_builder,
            consensus_builder,
            _marker: Default::default(),
        }
    }
}

pub struct LaunchContext<Builder> {
    pub address: IpAddr,
    pub port: u16,
}

impl<Builder> LaunchContext<Builder> {
    pub fn launch(self) {
        // Here you would implement the logic to launch the node using the provided components builder.
        // This is a placeholder for the actual launch logic.
        println!("Launching node at {}:{}", self.address, self.port);
    }
}
