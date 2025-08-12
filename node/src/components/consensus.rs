use payload::builder::BuildArguments;

use crate::{components::FullNodeTypes, error::BuildError};

pub trait ConsensusBuilder<Node: FullNodeTypes> {
    type Consensus;

    fn build_consensus(self) -> impl Future<Output = Result<Self::Consensus, BuildError>>;
}
