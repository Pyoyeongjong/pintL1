use std::sync::Arc;

use crate::{traits::PoolTransaction, validate::ValidPoolTransaction};

pub mod best;
pub mod parked;
pub mod pending;
pub mod state;
pub mod txpool;

// Represents a transaction that was added into the pool and its state
#[derive(Debug)]
pub enum AddedTransaction<T: PoolTransaction> {
    // Transaction was successfully added and moved to the pending pool
    Pending(AddedPendingTransaction<T>),
    // Successfully added but not yet ready for processing -> so it moved to the parked pool
    Parked {
        transaction: Arc<ValidPoolTransaction<T>>,
    },
}

// Tracks an added transaction
#[derive(Debug)]
pub struct AddedPendingTransaction<T: PoolTransaction> {
    transaction: Arc<ValidPoolTransaction<T>>,
}
