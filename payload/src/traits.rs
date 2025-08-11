//! Traits to implements!
//!
use primitives::types::{BlockHash, PayloadId};

use crate::{BuildOutcome, PintBuiltPayload, builder::BuildArguments, error::PayloadBuilderError};

pub trait PayloadJob: Send + Sync {
    type PayloadAttributes: PayloadBuilderAttributes;
}

pub trait PayloadJobGenerator {
    // The type that manages the lifecycle of a payload.
    type Job: PayloadJob;
    fn new_payload_job(
        &self,
        attr: <Self::Job as PayloadJob>::PayloadAttributes,
    ) -> Result<Self::Job, PayloadBuilderError>;
}

pub trait PayloadTypes {
    type BuiltPayload;
    type PayloadBuilderAttributes: PayloadBuilderAttributes + Clone;
}

pub trait PayloadBuilderAttributes {
    fn payload_id(&self) -> PayloadId;
    // Returns the hash or the parent block this payload builds on
    fn parent(&self) -> BlockHash;
    fn timestamp(&self) -> u64;
}

pub trait PayloadBuilder: Send + Sync + Clone {
    type BuiltPayload;
    type Attributes: PayloadBuilderAttributes;
    fn try_build(
        &self,
        args: BuildArguments<Self::Attributes>,
    ) -> Result<BuildOutcome<PintBuiltPayload>, PayloadBuilderError>;
}
