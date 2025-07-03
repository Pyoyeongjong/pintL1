//! Implements [Account]
use alloy_primitives::U256;

#[derive(Debug, Default, Copy, Clone)]
/// On chain accout
pub struct Account {
    pub nonce: u64,
    pub balance: U256,
}
