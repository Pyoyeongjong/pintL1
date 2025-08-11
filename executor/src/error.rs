#[derive(Debug)]
pub enum BlockExecutionError {
    Validation(BlockValidationError),
    ExecutionError,
    StateNotPrepared,
    SenderNotFound,
    InvalidTx,
}

#[derive(Debug)]
pub enum BlockValidationError {
    InvalidTx,
}

pub enum StateError {
    PreareExecutionError,
}
