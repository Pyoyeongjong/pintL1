use std::sync::Arc;

use crate::{traits::PoolTransaction, validate::ValidPoolTransaction};

pub mod txpool;
pub mod pending;
pub mod best;
pub mod state;
pub mod parked;

// Represents a transaction that was added into the pool and its state
pub enum AddedTransaction<T: PoolTransaction> {
    // Transaction was successfully added and moved to the pending pool
    Pending(AddedPendingTransaction<T>),
    // Successfully added but not yet ready for processing -> so it moved to the parked pool
    Parked {
        transaction: Arc<ValidPoolTransaction<T>>,
    }
}

// Tracks an added transaction
pub struct AddedPendingTransaction<T: PoolTransaction> {
    transaction: Arc<ValidPoolTransaction<T>>
}