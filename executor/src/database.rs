use core::num;
use std::collections::HashMap;

use primitives::{
    account::{self, Account},
    block::Block,
    types::Address,
};
use storage::{
    db::Database,
    error::{DatabaseError, ProviderError},
    traits::{AccountReader, ProviderResult, StateProvider},
};

use crate::{
    Receipt,
    error::{BlockExecutionError, StateError},
    transaction::ExecutableTranasction,
};

pub struct State<DB> {
    pub database: DB,
    pub transition_state: Option<HashMap<Address, Account>>,
}

impl<DB: StateProvider> State<DB> {
    pub fn new(db: DB) -> Self {
        Self {
            database: db,
            transition_state: None,
        }
    }

    pub fn prepare_execute(&mut self) -> Result<(), StateError> {
        let res = self.database.prepare_execute();
        self.transition_state = match res {
            Ok(state) => Some(state),
            Err(_) => return Err(StateError::PreareExecutionError),
        };

        Ok(())
    }

    pub fn execute_transaction(
        &mut self,
        tx: &ExecutableTranasction,
    ) -> Result<Receipt, BlockExecutionError> {
        if self.transition_state.is_none() {
            return Err(BlockExecutionError::StateNotPrepared);
        }

        let ExecutableTranasction {
            tx_type,
            chain_id: _,
            sender,
            receiver,
            nonce: _,
            value,
        } = tx;

        let state = self.transition_state.as_mut().unwrap();

        // update receiver's info
        match state.get_mut(&receiver) {
            Some(account) => {
                account.balance += value;
            }
            None => {
                let mut account = Account::default();
                account.balance += value;
                state.insert(receiver.clone(), account);
            }
        }

        // update sender's info
        match state.get_mut(&sender) {
            Some(account) => {
                account.nonce += 1;
                account.balance -= value;
            }
            None => {
                // It should never happen!
                return Err(BlockExecutionError::SenderNotFound);
            }
        };

        Ok(Receipt {
            tx_type: tx_type.clone(),
            success: true,
        })
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

    pub fn inner(&self) -> &DB {
        &self.0
    }
}

impl<DB: Database> AccountReader for StateProviderDatabase<DB> {
    fn basic_account(
        &self,
        address: &Address,
    ) -> Result<Option<Account>, storage::error::ProviderError> {
        let res = self
            .inner()
            .basic(address)
            .map_err(|e| ProviderError::DatabaseError(e))?;

        Ok(res)
    }
}

impl<DB: Database> StateProvider for StateProviderDatabase<DB> {
    fn account_balance(
        &self,
        addr: &Address,
    ) -> storage::traits::ProviderResult<Option<transaction::U256>> {
        self.basic_account(addr)?
            .map_or_else(|| Ok(None), |acc| Ok(Some(acc.balance)))
    }

    fn account_nonce(&self, addr: &Address) -> storage::traits::ProviderResult<Option<u64>> {
        self.basic_account(addr)?
            .map_or_else(|| Ok(None), |acc| Ok(Some(acc.nonce)))
    }

    fn prepare_execute(&mut self) -> storage::traits::ProviderResult<HashMap<Address, Account>> {
        todo!()
    }
}

impl<DB: Database> Database for StateProviderDatabase<DB> {
    fn basic(
        &self,
        address: &primitives::types::Address,
    ) -> Result<Option<primitives::account::Account>, DatabaseError> {
        todo!()
    }

    fn block_hash(
        &self,
        number: u64,
    ) -> Result<Option<primitives::types::BlockHash>, DatabaseError> {
        todo!()
    }

    fn block_number(&self) -> u64 {
        self.inner().block_number()
    }

    fn copy_state_from_block_no(
        &self,
        number: u64,
    ) -> Result<HashMap<Address, Account>, DatabaseError> {
        self.inner().copy_state_from_block_no(number)
    }
}
