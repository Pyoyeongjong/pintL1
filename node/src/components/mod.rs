use storage::traits::StateProviderFactory;

pub mod consensus;
pub mod execute;
pub mod network;
pub mod payload;
pub mod pool;

pub trait FullNodeTypes {
    type Provider: Clone + StateProviderFactory;
}
