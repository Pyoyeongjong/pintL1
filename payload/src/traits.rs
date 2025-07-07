//! Traits to implements!
//! 
use crate::error::PayloadBuilderError;

pub trait PayloadJob {
    type PayloadAttributes;
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
    type PayloadBuilderAttributes;
}

pub trait PayloadBuilder {
    type BuiltPayload;
    fn try_build(&self) -> Result<Self::BuiltPayload, PayloadBuilderError>;
}
