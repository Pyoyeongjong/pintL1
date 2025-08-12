pub enum BuildError {
    PoolError,
}
#[derive(Debug)]
pub enum LaunchError {
    BuildError,
}

impl From<BuildError> for LaunchError {
    fn from(value: BuildError) -> Self {
        Self::BuildError
    }
}