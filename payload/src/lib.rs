use std::sync::Arc;

use executor::database::State;
use primitives::{
    block::{Block, body::SealedBlock},
    types::{BlockHash, PayloadId, U256},
};

use storage::traits::StateProviderFactory;
use transaction::TransactionSigned;
use transaction_pool::traits::TransactionPool;

use crate::{
    builder::BuildArguments,
    error::PayloadBuilderError,
    traits::{PayloadBuilder, PayloadJob, PayloadJobGenerator},
};

pub mod builder;
pub mod error;
/// This trait should be tested by e2e test
pub mod traits;

pub struct PintPayloadJobGenerator<Client, Builder>
where
    Builder: PayloadBuilder,
{
    // The Client that can interact with the chain
    // client: ctx.provider.clone()!
    // executer: ctx.task_executor().clone()!
    client: Client,
    builder: Builder,
}

impl<Client, Builder> PayloadJobGenerator for PintPayloadJobGenerator<Client, Builder>
where
    Builder: PayloadBuilder,
{
    type Job = PintPayloadJob;

    fn new_payload_job(
        &self,
        attr: <Self::Job as traits::PayloadJob>::PayloadAttributes,
    ) -> Result<Self::Job, error::PayloadBuilderError> {
        todo!()
    }
}

pub struct PintPayloadJob {}

impl PayloadJob for PintPayloadJob {
    type PayloadAttributes = ();
}

pub struct PintPayloadBuilder<Pool, Client> {
    client: Client,
    pool: Pool,
}

impl<Pool, Client> PayloadBuilder for PintPayloadBuilder<Pool, Client>
where
    Pool: TransactionPool,
{
    type BuiltPayload = ();

    fn try_build(&self) -> Result<Self::BuiltPayload, error::PayloadBuilderError> {
        todo!()
    }
}

pub struct PintPayloadBuilderAttributes {
    // Id of the payload
    pub id: PayloadId,
    // Parent block to build to payload on top
    pub parent: BlockHash,
    pub timestamp: u64,
    pub parent_beacon_block_root: Option<BlockHash>,
}

pub struct PintBuiltPayload {
    pub id: PayloadId,
    pub block: Arc<SealedBlock<Block<TransactionSigned>>>,
    pub fees: U256,
}

// pub fn default_pint_payload<PvmConfig, Client, Pool, F>(
//     args: BuildArguments<PintPayloadBuilderAttributes>,
//     client: Client,
//     pool: Pool,
//     best_txs: F,
// ) -> Result<BuildOutcome<PintBuiltPayload>, PayloadBuilderError>
// where
//     Pool: TransactionPool,
//     Client: StateProviderFactory,
// {
//     let BuildArguments {
//         parent_header,
//         attributes,
//     } = args;
//     let state_provider = client.state_by_block_hash(parent_header.hash())?;
//     let mut db = State::builder().with_database(state_provider).build();

//     let mut builder = pvm_config.builder_for_next_block();

//     let mut best_txs = pool.best_transactions();

//     let mut total_fee = U256::ZERO;

//     while let Some(pool_tx) = best_txs.next() {
//         let result = match builder.execute_transaction(pool_tx.clone()) {
//             Ok(()) => {}
//             Err(BlockExecutionError::Validation(BlockValidationError::InvalidTx { error })) => {}
//             Err(err) => return Err(PayloadBuilderError::ExecutionError(err)),
//         };

//         let miner_fee = pool_tx.transaction.cost();
//         total_fee += miner_fee;
//     }

//     let BlockBuilderOutcome {} = builder.finish(&state_provider)?;
//     let payload = PintBuiltPayload::new();
//     Ok(BuildOutcome::Better { payload })
// }

pub enum BuildOutcome<Payload> {
    Better { payload: Payload },
}
