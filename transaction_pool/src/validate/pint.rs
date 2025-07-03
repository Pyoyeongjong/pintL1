//! Handle validation of Pint type Transaction
use std::{marker::PhantomData, sync::Arc};

use storage::state::{StateProvider, StateProviderFactory};
use transaction::U256;

use crate::{
    error::InvalidPoolTransactionError,
    traits::{PoolTransaction, TransactionOrigin},
    validate::{TransactionValidationOutcome, TransactionValidator},
};

/// Pint Transaction Validator
pub struct PintTransactionValidator<Client, Tx> {
    inner: Arc<PintTransactionValidatorInner<Client, Tx>>,
}

impl<Client, Tx> TransactionValidator for PintTransactionValidator<Client, Tx>
where
    Client: Sync + Send + StateProviderFactory,
    Tx: PoolTransaction,
{
    type Transaction = Tx;

    async fn validate_transaction(
        &self,
        origin: TransactionOrigin,
        transaction: Self::Transaction,
    ) -> TransactionValidationOutcome<Tx> {
        self.validate_one(origin, transaction)
    }
}

impl<Client, Tx> PintTransactionValidator<Client, Tx>
where
    Client: StateProviderFactory,
    Tx: PoolTransaction,
{
    pub fn validate_one(
        &self,
        origin: TransactionOrigin,
        transaction: Tx,
    ) -> TransactionValidationOutcome<Tx> {
        self.inner.validate_one(origin, transaction)
    }
}

/// Pint Transaction Validator Inner
pub(crate) struct PintTransactionValidatorInner<Client, Tx> {
    client: Client,
    tx_fee_cap: Option<u128>,
    _marker: PhantomData<Tx>,
}

impl<Client, Tx> PintTransactionValidatorInner<Client, Tx>
where
    Client: StateProviderFactory,
    Tx: PoolTransaction,
{
    pub fn validate_one(
        &self,
        origin: TransactionOrigin,
        transaction: Tx,
    ) -> TransactionValidationOutcome<Tx> {
        let mut provider = None;
        self.validate_one_with_provider(origin, transaction, &mut provider)
    }

    // Validates a single transaction using an optional cached state provider.
    // If no provider is passed, a new one will be created. THis allow reusing the same provider across
    // multiple txs.
    fn validate_one_with_provider(
        &self,
        origin: TransactionOrigin,
        transaction: Tx,
        maybe_state: &mut Option<Box<dyn StateProvider>>,
    ) -> TransactionValidationOutcome<Tx> {
        match self.validate_one_no_state(origin, transaction) {
            Ok(transaction) => {
                if maybe_state.is_none() {
                    match self.client.latest() {
                        Ok(new_state) => {
                            *maybe_state = Some(new_state);
                        }
                        Err(err) => {
                            return TransactionValidationOutcome::Error(
                                transaction.hash(),
                                Box::new(err),
                            );
                        }
                    }
                }

                let state: &dyn StateProvider = maybe_state.as_deref().expect("provider is set");
                self.validate_one_against_state(origin, transaction, state)
            }
            Err(invalid_outcome) => invalid_outcome,
        }
    }

    fn validate_one_against_state(
        &self,
        origin: TransactionOrigin,
        transaction: Tx,
        state: &dyn StateProvider,
    ) -> TransactionValidationOutcome<Tx> {
        let account = match state.basic_account(&transaction.sender()) {
            Ok(account) => account.unwrap_or_default(),
            Err(err) => {
                return TransactionValidationOutcome::Error(transaction.hash(), Box::new(err));
            }
        };

        // Checks nonce >= on_chain_node
        if transaction.nonce() < account.nonce {
            return TransactionValidationOutcome::Invalid(
                transaction,
                InvalidPoolTransactionError::NonceNotConsistent,
            );
        }

        TransactionValidationOutcome::Valid {
            transaction,
            balance: account.balance,
            nonce: account.nonce,
            propagate: match origin {
                TransactionOrigin::External => true,
                TransactionOrigin::Local => true,
                TransactionOrigin::Private => false,
            },
        }
    }

    fn validate_one_no_state(
        &self,
        origin: TransactionOrigin,
        transaction: Tx,
    ) -> Result<Tx, TransactionValidationOutcome<Tx>> {
        match transaction.tx_type() {
            0 => {}
            _ => {
                return Err(TransactionValidationOutcome::Invalid(
                    transaction,
                    InvalidPoolTransactionError::TxTypeNotSupported,
                ));
            }
        };

        // Check Fee is bigger than 0
        if transaction.cost() <= U256::from(0) {
            return Err(TransactionValidationOutcome::Invalid(
                transaction,
                InvalidPoolTransactionError::NotEnoughFee,
            ));
        }

        Ok(transaction)
    }
}

/// Pint Transaction Validator Builder
pub struct PintTransactionValidatorBuilder<Client> {
    client: Client,
    tx_fee_cap: Option<u128>,
}

impl<Client> PintTransactionValidatorBuilder<Client> {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            tx_fee_cap: Some(1e9 as u128),
        }
    }

    // fee_cap = 0 -> no cap
    pub fn set_tx_fee_cap(mut self, fee_cap: u128) -> Self {
        self.tx_fee_cap = Some(fee_cap);
        self
    }

    // If Tx can be inferred by other function like validate_transaction(tx)..
    pub fn build<Tx>(self) -> PintTransactionValidator<Client, Tx> {
        let Self { client, tx_fee_cap } = self;

        let inner = PintTransactionValidatorInner {
            client,
            tx_fee_cap,
            _marker: Default::default(),
        };

        PintTransactionValidator {
            inner: Arc::new(inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::TransactionPool;
    use primitives::transaction::SignedTransaction;
    use primitives::{Transaction, transaction::Decodable};
    use transaction::{U256, transaction::TxEnvelope};

    use crate::{
        Pool,
        ordering::PintOrdering,
        test_utils::mock::{ExtendedAccount, MockPintProvider},
        traits::{PintPooledTransaction, TransactionOrigin},
        validate::TransactionValidationOutcome,
    };

    use super::*;

    fn get_transaction() -> PintPooledTransaction {
        // This is external serialized encoded tx
        // fee = 1, value = 1, nonce = 0
        let raw = "000000000000000000000000000000000025f8130ba8d468bb79017d108ee62434ca224f3900000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000101c00ee231f05d26e97edb92b8426aec0767aded7d675e800b6ff07e28afeaf612352d25ecca9a18b15dd45bc79adb25c6d1acd2ddf3e76ce51bfba2bbbdb40500";
        let data = hex::decode(raw).unwrap();

        let (tx, _) = TxEnvelope::decode(&data).unwrap();
        PintPooledTransaction::from_pooled(tx.try_into_recovered().unwrap())
    }

    fn get_transaction_with_zero_fee() -> PintPooledTransaction {
        // This is external serialized encoded tx
        let raw = "000000000000000000000000000000000025f8130ba8d468bb79017d108ee62434ca224f3900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000101c00ee231f05d26e97edb92b8426aec0767aded7d675e800b6ff07e28afeaf612352d25ecca9a18b15dd45bc79adb25c6d1acd2ddf3e76ce51bfba2bbbdb40500";
        let data = hex::decode(raw).unwrap();

        let (tx, _) = TxEnvelope::decode(&data).unwrap();
        PintPooledTransaction::from_pooled(tx.try_into_recovered().unwrap())
    }

    #[tokio::test]
    async fn test_validate_pending_transaction() {
        let transaction = get_transaction();
        let mut provider = MockPintProvider::default();

        provider.add_account(
            transaction.sender(),
            ExtendedAccount::new(transaction.nonce(), U256::MAX),
        );

        let validator = PintTransactionValidatorBuilder::new(provider).build();

        let outcome: TransactionValidationOutcome<PintPooledTransaction> =
            validator.validate_one(TransactionOrigin::External, transaction.clone());
        assert!(outcome.is_valid());

        let pool = Pool::new(validator, PintOrdering::default(), Default::default());

        let res = pool.add_external_transaction(transaction.clone()).await;
        assert!(res.is_ok());
        let tx = pool.get(&transaction.hash());
        assert!(tx.is_some());
    }

    #[tokio::test]
    async fn test_validate_parked_transaction() {
        let transaction = get_transaction();
        let mut provider = MockPintProvider::default();

        provider.add_account(
            transaction.sender(),
            ExtendedAccount::new(transaction.nonce(), U256::ZERO),
        );

        let validator = PintTransactionValidatorBuilder::new(provider).build();

        let outcome: TransactionValidationOutcome<PintPooledTransaction> =
            validator.validate_one(TransactionOrigin::External, transaction.clone());
        assert!(outcome.is_valid());

        let pool = Pool::new(validator, PintOrdering::default(), Default::default());

        let res = pool.add_external_transaction(transaction.clone()).await;
        assert!(res.is_ok());
        let tx = pool.get(&transaction.hash());
        assert!(tx.is_some());
    }

    #[tokio::test]
    async fn test_validate_invalid_on_nonce() {
        let transaction = get_transaction();
        let mut provider = MockPintProvider::default();

        provider.add_account(
            transaction.sender(),
            ExtendedAccount::new(transaction.nonce() + 1, U256::MAX),
        );

        let validator = PintTransactionValidatorBuilder::new(provider).build();

        let outcome: TransactionValidationOutcome<PintPooledTransaction> =
            validator.validate_one(TransactionOrigin::External, transaction.clone());
        assert!(!outcome.is_valid());

        let pool = Pool::new(validator, PintOrdering::default(), Default::default());

        let res = pool.add_external_transaction(transaction.clone()).await;
        assert!(res.is_err());
        let tx = pool.get(&transaction.hash());
        assert!(tx.is_none());
    }

    #[tokio::test]
    async fn test_validate_invalid_on_fee() {
        let transaction = get_transaction_with_zero_fee();
        let mut provider = MockPintProvider::default();

        provider.add_account(
            transaction.sender(),
            ExtendedAccount::new(transaction.nonce(), U256::MAX),
        );

        let validator = PintTransactionValidatorBuilder::new(provider).build();

        let outcome: TransactionValidationOutcome<PintPooledTransaction> =
            validator.validate_one(TransactionOrigin::External, transaction.clone());
        assert!(!outcome.is_valid());

        let pool = Pool::new(validator, PintOrdering::default(), Default::default());

        let res = pool.add_external_transaction(transaction.clone()).await;
        assert!(res.is_err());
        let tx = pool.get(&transaction.hash());
        assert!(tx.is_none());
    }
}
