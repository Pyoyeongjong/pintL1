use std::{
    pin::Pin,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use executor::{
    BlockBuilderOutcome, PintBlockExecutor,
    database::State,
    error::{BlockExecutionError, BlockValidationError},
    traits::BlockExecutor,
};
use primitives::{
    block::{Block, body::SealedBlock},
    types::{BlockHash, PayloadId, U256},
};

use storage::traits::{BlockReader, StateProviderFactory};
use tokio::time::Sleep;
use transaction::TransactionSigned;
use transaction_pool::traits::{PoolTransaction, TransactionPool};

use crate::{
    builder::BuildArguments,
    error::PayloadBuilderError,
    traits::{PayloadBuilder, PayloadBuilderAttributes, PayloadJob, PayloadJobGenerator},
};

pub mod builder;
pub mod error;
/// This trait should be tested by e2e test
pub mod traits;

/// PayloadJobGenerator
pub struct PintPayloadJobGenerator<Client, Tasks, Builder>
where
    Builder: PayloadBuilder,
{
    // The Client that can interact with the chain
    // client: ctx.provider.clone()!
    // executer: ctx.task_executor().clone()!
    client: Client,
    /// Tokio task executor
    executor: Tasks,
    /// This is executor in our project
    builder: Builder,
}

impl<Client, Task, Builder> PintPayloadJobGenerator<Client, Task, Builder>
where
    Builder: PayloadBuilder,
{
    fn job_deadline(&self, unix_timestamp: u64) -> tokio::time::Instant {
        let unix_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp = Duration::from_secs(unix_timestamp);
        tokio::time::Instant::now() + timestamp.saturating_sub(unix_now)
    }
}

/// spawn new payload job task
impl<Client, Tasks, Builder> PayloadJobGenerator for PintPayloadJobGenerator<Client, Tasks, Builder>
where
    Client: BlockReader,
    Builder: PayloadBuilder,
    Tasks: Send + Sync + Clone,
{
    type Job = PintPayloadJob<Tasks, Builder>;

    fn new_payload_job(
        &self,
        attr: <Self::Job as PayloadJob>::PayloadAttributes,
    ) -> Result<Self::Job, error::PayloadBuilderError> {
        let parent_header = if attr.parent().is_zero() {
            self.client
                .latest_header()
                .map_err(PayloadBuilderError::from)?
                .ok_or_else(|| PayloadBuilderError::MissingParentHeader(BlockHash::ZERO))?
        } else {
            self.client
                .sealed_header_by_hash(attr.parent())
                .map_err(PayloadBuilderError::from)?
                .ok_or_else(|| PayloadBuilderError::MissingParentHeader(BlockHash::ZERO))?
        };

        let until = self.job_deadline(attr.timestamp());
        let deadline = Box::pin(tokio::time::sleep_until(until));

        let mut job = PintPayloadJob {
            executor: self.executor.clone(),
            builder: self.builder.clone(),
            deadline,
        };

        job.spawn_build_job();

        Ok(job)
    }
}

/// Generated Payload Job
pub struct PintPayloadJob<Tasks, Builder>
where
    Builder: PayloadBuilder,
{
    executor: Tasks,
    builder: Builder,
    deadline: Pin<Box<Sleep>>,
}

impl<Task, Builder> PintPayloadJob<Task, Builder>
where
    Builder: PayloadBuilder,
{
    // spawns new payload build job
    fn spawn_build_job(&mut self) {}
}

impl<Tasks, Builder> PayloadJob for PintPayloadJob<Tasks, Builder>
where
    Tasks: Send + Sync,
    Builder: PayloadBuilder,
{
    type PayloadAttributes = Builder::Attributes;
}

#[derive(Clone)]
pub struct PintPayloadBuilder<Pool, Client> {
    client: Client,
    pool: Pool,
}

impl<Pool, Client> PintPayloadBuilder<Pool, Client> {
    pub const fn new(client: Client, pool: Pool) -> Self {
        Self { client, pool }
    }
}

impl<Pool, Client> PayloadBuilder for PintPayloadBuilder<Pool, Client>
where
    Client: StateProviderFactory + Clone,
    Pool: TransactionPool,
{
    type BuiltPayload = PintBuiltPayload;
    type Attributes = PintPayloadBuilderAttributes;

    fn try_build(
        &self,
        args: BuildArguments<Self::Attributes>,
    ) -> Result<BuildOutcome<PintBuiltPayload>, PayloadBuilderError> {
        default_pint_payload(args, self.client.clone(), self.pool.clone())
    }
}

#[derive(Clone)]
pub struct PintPayloadBuilderAttributes {
    // Id of the payload
    pub id: PayloadId,
    // Parent block to build to payload on top
    pub parent: BlockHash,
    pub timestamp: u64,
    pub parent_beacon_block_root: Option<BlockHash>,
}

impl PayloadBuilderAttributes for PintPayloadBuilderAttributes {
    fn payload_id(&self) -> PayloadId {
        todo!()
    }

    fn parent(&self) -> BlockHash {
        todo!()
    }

    fn timestamp(&self) -> u64 {
        todo!()
    }
}

#[derive(Clone)]
pub struct PintBuiltPayload {
    pub id: PayloadId,
    pub block: Arc<SealedBlock<Block<TransactionSigned>>>,
    pub fees: U256,
}

impl PintBuiltPayload {
    pub const fn new(
        id: PayloadId,
        block: Arc<SealedBlock<Block<TransactionSigned>>>,
        fees: U256,
    ) -> Self {
        Self { id, block, fees }
    }
}

pub enum BuildOutcome<Payload> {
    Better { payload: Payload },
}

/// Constructs an Ethereum transaction payload using the best transactions from
/// the pool
///
pub fn default_pint_payload<Client, Pool>(
    args: BuildArguments<PintPayloadBuilderAttributes>,
    client: Client,
    pool: Pool,
) -> Result<BuildOutcome<PintBuiltPayload>, PayloadBuilderError>
where
    Pool: TransactionPool,
    Client: StateProviderFactory + Clone,
{
    let BuildArguments {
        parent_header,
        attributes,
    } = args;
    let state_provider = client.state_by_block_hash(parent_header.hash())?;
    let state = State::new(state_provider);
    let mut executor = PintBlockExecutor {
        state,
        receipts: Vec::new(),
    };
    let mut best_txs = pool.best_transactions();
    let mut total_fee = U256::ZERO;

    while let Some(pool_tx) = best_txs.next() {
        match executor.execute_transaction(&pool_tx.clone().into()) {
            Ok(_) => {}
            Err(BlockExecutionError::Validation(BlockValidationError::InvalidTx)) => {}
            Err(err) => return Err(PayloadBuilderError::ExecutionError),
        };

        let miner_fee = pool_tx.transaction.cost();
        total_fee += miner_fee;
    }

    let BlockBuilderOutcome {
        receipts, block, ..
    } = executor.finish()?;

    let sealed_block = Arc::new(block);

    let payload = PintBuiltPayload::new(attributes.id, sealed_block, total_fee);
    Ok(BuildOutcome::Better { payload })
}
