use std::{fmt::Debug};
use core::default::Default;

use primitives::{types::{Address, TxHash, U256}, Transaction};

#[derive(Debug, Default, Clone)]
pub enum TransactionOrigin {
    #[default]
    Local,
    External,
    Private
}

pub trait PoolTransaction: 
    Debug + Transaction
{
    fn hash(&self) -> TxHash;
    fn sender(&self) -> Address;
    fn cost(&self) -> U256;
}