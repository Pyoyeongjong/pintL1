use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures_util::StreamExt;
use primitives::{block::header::SealedHeader, types::PayloadId};
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    oneshot,
};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    error::PayloadBuilderError,
    traits::{PayloadBuilderAttributes, PayloadJob, PayloadJobGenerator, PayloadTypes},
};

/// Command for PayloadBuilder
pub enum PayloadServiceCommand<T: PayloadTypes> {
    BuildNewPayload(
        T::PayloadBuilderAttributes,
        oneshot::Sender<Result<PayloadId, PayloadBuilderError>>,
    ),
}

/// Top Struct or PayloadBuilder (PayloadJobGenerator + Channel)
pub struct PayloadBuilderService<Gen, T>
where
    T: PayloadTypes,
    Gen: PayloadJobGenerator,
{
    generator: Gen,
    service_tx: UnboundedSender<PayloadServiceCommand<T>>,
    command_rx: UnboundedReceiverStream<PayloadServiceCommand<T>>,
    payload_jobs: Vec<(Gen::Job, PayloadId)>,
}

impl<Gen, T> PayloadBuilderService<Gen, T>
where
    T: PayloadTypes,
    Gen: PayloadJobGenerator,
{
    pub fn new(generator: Gen) -> (Self, PayloadBuilderHandle<T>) {
        let (service_tx, command_rx) = mpsc::unbounded_channel();
        let service = Self {
            generator,
            service_tx,
            command_rx: UnboundedReceiverStream::new(command_rx),
            payload_jobs: Vec::new(),
        };

        let handle = service.handle();
        (service, handle)
    }

    pub fn handle(&self) -> PayloadBuilderHandle<T> {
        PayloadBuilderHandle::new(self.service_tx.clone())
    }

    // Returns true if the given payload is currently being built
    fn contains_payload(&self, id: PayloadId) -> bool {
        self.payload_jobs.iter().any(|(_, job_id)| *job_id == id)
    }
}

/// It can be performed by await future..!
impl<Gen, T> Future for PayloadBuilderService<Gen, T>
where
    T: PayloadTypes,
    Gen: PayloadJobGenerator + Unpin,
    <Gen as PayloadJobGenerator>::Job: Unpin,
    Gen::Job: PayloadJob<PayloadAttributes = T::PayloadBuilderAttributes>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        loop {
            let mut new_job = false;
            while let Poll::Ready(Some(cmd)) = this.command_rx.poll_next_unpin(cx) {
                match cmd {
                    PayloadServiceCommand::BuildNewPayload(attr, tx) => {
                        let id = attr.payload_id();
                        let mut res = Ok(id);

                        if this.contains_payload(id) {
                            dbg!("Payload job already in progress. ignoring");
                        } else {
                            // If no job for this payload yet, create one!
                            match this.generator.new_payload_job(attr.clone()) {
                                Ok(job) => {
                                    new_job = true;
                                    this.payload_jobs.push((job, id));
                                }
                                Err(err) => {
                                    res = Err(err);
                                }
                            }
                        }

                        let _ = tx.send(res);
                    }
                }
            }

            if !new_job {
                return Poll::Pending;
            }
        }
    }
}

/// Channel Handler for PayloadBuilder. It serves Command for Payload Service.
pub struct PayloadBuilderHandle<T: PayloadTypes> {
    to_service: UnboundedSender<PayloadServiceCommand<T>>,
}

impl<T: PayloadTypes> PayloadBuilderHandle<T> {
    pub fn new(to_service: mpsc::UnboundedSender<PayloadServiceCommand<T>>) -> Self {
        Self { to_service }
    }
}

/// PayloadBuilder Arguments (parent header + attributes)
pub struct BuildArguments<Attributes, Header = primitives::block::header::Header> {
    pub parent_header: Arc<SealedHeader<Header>>,
    pub attributes: Attributes,
}
