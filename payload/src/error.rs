use executor::error::BlockExecutionError;
use primitives::types::BlockHash;
use storage::error::ProviderError;

pub enum PayloadBuilderError {
    MissingParentHeader(BlockHash),
    ExecutionError,
    ProviderError,
    BlockExecutionError,
}

impl From<ProviderError> for PayloadBuilderError {
    fn from(value: ProviderError) -> Self {
        Self::ProviderError
    }
}

impl From<BlockExecutionError> for PayloadBuilderError {
    fn from(value: BlockExecutionError) -> Self {
        Self::BlockExecutionError
    }
}
