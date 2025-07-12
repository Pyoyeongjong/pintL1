use storage::{
    db::Database,
    traits::{StateProvider, StateProviderBox},
};
pub struct State<DB> {
    pub database: DB,
}

impl<DB> State<DB> {
    pub fn new(db: DB) -> Self {
        Self { database: db }
    }
}

#[derive(Default)]
pub struct StateProviderDatabase<DB>(pub DB);

impl<DB> StateProviderDatabase<DB> {
    pub fn new(db: DB) -> Self {
        Self(db)
    }

    pub fn into_inner(self) -> DB {
        self.0
    }
}

impl<DB: StateProvider> Database for StateProviderDatabase<DB> {
    fn basic(
        &self,
        address: &primitives::types::Address,
    ) -> Result<Option<primitives::account::Account>, storage::error::DatabaseError> {
        todo!()
    }

    fn block_hash(
        &self,
        number: u64,
    ) -> Result<Option<primitives::types::BlockHash>, storage::error::DatabaseError> {
        todo!()
    }

    fn block_number(&self) -> u64 {
        todo!()
    }
}
