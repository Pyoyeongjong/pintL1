use ::payload::traits::PayloadTypes;
use storage::traits::StateProviderFactory;

use crate::error::BuildError;

pub mod consensus;
pub mod execute;
pub mod network;
pub mod payload;
pub mod pool;

pub trait FullNodeTypes {
    type Provider: Clone + StateProviderFactory;
    type Payload: PayloadTypes;
}

pub trait NodeComponentsBuilder {

    type Components;
    type Provider: StateProviderFactory;
    fn build_components(self, provider: Self::Provider) -> impl Future<Output = Result<Self::Components, BuildError>>;
}
