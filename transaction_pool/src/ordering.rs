//! Ordering for Transaction Pool
//! It defines all of the ordering or transactions like [Pint](transaction::pint_tx)
use std::cmp::Ordering;
use std::fmt::Debug;
use std::marker::PhantomData;

use primitives::types::U256;

use crate::traits::PoolTransaction;

/// Transaction Ordering
/// It contains Transaction Type and Priority Value
pub trait TransactionOrdering: Debug + Send + 'static {
    type Transaction: PoolTransaction;
    type PriorityValue: Clone + Debug + Send + Sync + Ord;

    fn priority(&self, transaction: &Self::Transaction) -> Priority<Self::PriorityValue>;
}

// Priority enum
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Priority<T: Ord> {
    Value(T),
    None,
}

impl<T: Ord> From<Option<T>> for Priority<T> {
    fn from(value: Option<T>) -> Self {
        // if some -> f(x)
        // if none -> default
        value.map_or(Self::None, Priority::Value)
    }
}

impl<T: Ord> PartialOrd for Priority<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord> Ord for Priority<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Value(a), Self::Value(b)) => a.cmp(b),
            (Self::Value(_), Self::None) => Ordering::Greater,
            (Self::None, Self::Value(_)) => Ordering::Less,
            (Self::None, Self::None) => Ordering::Equal,
        }
    }
}

/// Implements [Pint](transaction::pint_tx) Ordering
#[derive(Debug)]
pub struct PintOrdering<T>(PhantomData<T>);

impl<T> Default for PintOrdering<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> TransactionOrdering for PintOrdering<T>
where
    T: PoolTransaction + Send,
{
    type PriorityValue = U256;
    type Transaction = T;

    fn priority(&self, transaction: &Self::Transaction) -> Priority<Self::PriorityValue> {
        transaction.get_priority().map(U256::from).into()
    }
}
