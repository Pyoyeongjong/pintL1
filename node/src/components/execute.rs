use crate::{components::FullNodeTypes, error::BuildError};

pub trait ExecutorBuilder<Node: FullNodeTypes> {
    type Exec;

    fn build_exec(self) -> impl Future<Output = Result<Self::Exec, BuildError>>;
}
