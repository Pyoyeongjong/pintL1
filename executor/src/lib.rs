pub mod database;
pub mod error;
pub mod traits;
pub mod transaction;

use primitives::types::TxHash;
use storage::{db::Database, traits::StateProvider};

use crate::{
    database::State, error::BlockExecutionError, traits::BlockExecutor,
    transaction::ExecutableTranasction,
};

/// Transaction executor
pub struct PintBlockExecutor<DB> {
    pub state: State<DB>,
    pub receipts: Vec<Receipt>,
}

impl<DB: StateProvider> PintBlockExecutor<DB> {
    pub fn prepare_execute(&mut self) -> Result<(), BlockExecutionError> {
        let _ = self
            .state
            .prepare_execute()
            .map_err(|_| BlockExecutionError::ExecutionError)?;
        Ok(())
    }
}

impl<DB> BlockExecutor for PintBlockExecutor<DB> {
    type Transaction = ExecutableTranasction;

    fn execute_transaction(
        &mut self,
        tx: &Self::Transaction,
    ) -> Result<Option<u64>, BlockExecutionError> {
        todo!()
    }

    fn execute_and_commit(
        &mut self,
        tx: &Self::Transaction,
    ) -> Result<Option<u64>, BlockExecutionError> {
        todo!()
    }
}

#[derive(Default)]
pub struct Receipt {
    tx_hash: TxHash,
    success: bool,
}

#[cfg(test)]
mod tests {

    use ::transaction::{
        U256,
        traits::{Decodable, SignedTransaction},
        transaction::TxEnvelope,
    };
    use primitives::types::Address;
    use storage::{
        PintStateProviderFactory,
        db::InMemoryDB,
        traits::{StateProviderBox, StateProviderFactory},
    };
    use transaction_pool::{
        Pool,
        config::PoolConfig,
        ordering::PintOrdering,
        traits::{PintPooledTransaction, PoolTransaction, TransactionPool},
        validate::pint::{PintTransactionValidator, PintTransactionValidatorBuilder},
    };

    use crate::{
        database::{State, StateProviderDatabase},
        traits::ExecutableTx,
    };

    use super::*;

    fn make_pool() -> (
        Pool<
            PintTransactionValidator<PintStateProviderFactory<InMemoryDB>, PintPooledTransaction>,
            PintOrdering<PintPooledTransaction>,
        >,
        InMemoryDB,
        PintStateProviderFactory<InMemoryDB>,
    ) {
        let db = InMemoryDB::new();
        let provider = PintStateProviderFactory::new(db.clone());
        let validator = PintTransactionValidatorBuilder::new(provider.clone()).build();
        let config = PoolConfig::default();
        let pool: Pool<
            PintTransactionValidator<PintStateProviderFactory<InMemoryDB>, PintPooledTransaction>,
            PintOrdering<PintPooledTransaction>,
        > = Pool::new(validator, PintOrdering::default(), config);
        (pool, db, provider)
    }

    fn make_pool_transaction_1() -> PintPooledTransaction {
        // sender: a24a188cdcb3bf5fc6ec498d2657c6066b242028, receiver: e0aa4e80c739ee08b5a6680586d1bf3991840c21, fee: 1, value: 1, nonce: 0
        let raw = "0000000000000000000000000000000000e0aa4e80c739ee08b5a6680586d1bf3991840c21000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001be2855167f254060b5812e4a2849c7ba3d34ea4aeb175e87f83c2a7c1424379a6e722511c17cb5191e090b2a75dfe2b924d2b1bcbf0a2f26e207cb728dcaa34501";
        let data = hex::decode(raw).unwrap();
        let (tx, _) = TxEnvelope::decode(&data).unwrap();
        PintPooledTransaction::from_pooled(tx.try_into_recovered().unwrap())
    }

    fn make_pool_transaction_2() -> PintPooledTransaction {
        // sender: 314f3ea92a6fc23d6b66057d3acfba04d6b08b58, receiver: 802d9a22dddb7b03ff11eea121bdd4a75135e408, fee: 1, value: 1, nonce: 0
        let raw = "0000000000000000000000000000000000802d9a22dddb7b03ff11eea121bdd4a75135e4080000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000016969fda9b07fdf03f3092c06e9bd4def87edd1138b214be3ab724d980c0c12764b7150e282c63b1f42107b07a82a946d15ff56d921c2acd6fab423e22b94485f01";
        let data = hex::decode(raw).unwrap();
        let (tx, _) = TxEnvelope::decode(&data).unwrap();
        PintPooledTransaction::from_pooled(tx.try_into_recovered().unwrap())
    }

    /// for making new payload
    #[tokio::test]
    async fn test_execute_transaction() {
        // make default pool
        let (pool, _db, mut provider) = make_pool();
        // modify db (add balance .etc)
        provider
            .db
            .set_balance(
                Address::from_hex("a24a188cdcb3bf5fc6ec498d2657c6066b242028".to_string()).unwrap(),
                U256::MAX,
            )
            .unwrap();

        provider
            .db
            .set_balance(
                Address::from_hex("314f3ea92a6fc23d6b66057d3acfba04d6b08b58".to_string()).unwrap(),
                U256::MAX,
            )
            .unwrap();

        // get the latest state provider
        // 이건 StateProvider (DB + State Block Number)
        let state_provider: StateProviderBox = provider.latest().unwrap();
        // State 실행을 위해서 wrapping (Provider + Transition_State(모의 실행용))
        let state = State::new(state_provider);
        // make the executor (State + 영수증 용)
        let mut executor = PintBlockExecutor {
            state,
            receipts: Vec::new(),
        };

        // add_transaction
        let tx1 = make_pool_transaction_1();
        let tx2 = make_pool_transaction_2();
        let _res1 = pool.add_external_transaction(tx1).await.unwrap();
        let _res2 = pool.add_external_transaction(tx2).await.unwrap();
        // pool.inner().pool().read().print_pool_len();
        // get best transactions from the pool
        let txs: Vec<_> = pool
            .best_transactions()
            .map(|tx| {
                let exec_tx = ExecutableTranasction::from_pool_transaction(tx.transaction.clone());
                exec_tx
            })
            .collect();

        assert_eq!(txs.len(), 2);

        executor.prepare_execute().unwrap();

        for tx in txs.iter() {
            let res = executor.execute_transaction(tx);
        }
    }
}
