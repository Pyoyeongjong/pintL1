#[derive(Debug)]
pub enum BlockExecutionError {
    ExecutionError,
    StateNotPrepared,
    SenderNotFound,
}

pub enum StateError {
    PreareExecutionError,
}
