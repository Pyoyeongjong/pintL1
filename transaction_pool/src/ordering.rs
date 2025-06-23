use std::fmt::Debug;
use std::marker::PhantomData;

use primitives::types::U256;

use crate::traits::PoolTransaction;

#[derive(PartialEq, Clone, Debug)]
pub enum Priority<T> {
    Value(T),
    None,
}

impl<T> From<Option<T>> for Priority<T> {
    fn from(value: Option<T>) -> Self {
        // if some -> f(x)
        // if none -> default
        value.map_or(Self::None, Priority::Value)
    }
}

pub trait TransactionOrdering: Debug + Send {
    type Transaction: PoolTransaction;
    type PriorityValue: Clone + Debug;

    fn priority(&self, transaction: &Self::Transaction) -> Priority<Self::PriorityValue>;
}

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
