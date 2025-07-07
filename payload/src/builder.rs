use std::{sync::Arc, task::Poll};

use futures_util::StreamExt;
use primitives::{block::header::SealedHeader, types::PayloadId};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::traits::{PayloadJobGenerator, PayloadTypes};

pub enum PayloadServiceCommand<T: PayloadTypes> {
    BuildNewPayload(T::PayloadBuilderAttributes),
}

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

    fn contains_payload(&self, id: PayloadId) -> bool {
        self.payload_jobs.iter().any(|(_, job_id)| *job_id == id)
    }
}

impl<Gen, T> Future for PayloadBuilderService<Gen, T>
where
    T: PayloadTypes,
    Gen: PayloadJobGenerator + Unpin,
{
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
        // let this = self.get_mut();
        // loop {
        //     //while let Poll::Ready(Some(cmd)) = this.command_
        //     while let Poll::Ready(Some(cmd)) = this.command_rx.poll_next_unpin(cx) {
        //         match cmd {
        //             PayloadServiceCommand::BuildNewPayload(attr) => {
        //                 let id = attr.payload_id();
        //                 let mut res = Ok(id);

        //                 if this.contains_payload(id) {
        //                     dbg!("Payload job already in progress. ignoring");
        //                 } else {
        //                     let parent = attr.parent();
        //                     match this.generator.new_payload_job(attr.clone()) {
        //                         Ok(job) => {
        //                             new_job = true;
        //                             this.payload_jobs.push((job, id));
        //                         }
        //                         Err(err) => {
        //                             res = Err(err);
        //                         }
        //                     }
        //                 }

        //                 let _ = tx.send(res);
        //             }
        //         }
        //     }
        // }
    }
}

pub struct PayloadBuilderHandle<T: PayloadTypes> {
    to_service: UnboundedSender<PayloadServiceCommand<T>>,
}

impl<T: PayloadTypes> PayloadBuilderHandle<T> {
    pub fn new(to_service: mpsc::UnboundedSender<PayloadServiceCommand<T>>) -> Self {
        Self { to_service }
    }
}

pub struct BuildArguments<Attributes, Header = primitives::block::header::Header> {
    pub parent_header: Arc<SealedHeader<Header>>,
    pub attributes: Attributes,
}
