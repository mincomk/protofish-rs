use std::sync::Arc;

use tokio::sync::mpsc::Receiver;

use crate::{
    core::common::error::ConnectionError,
    internal::pmc_frame::send_frame,
    schema::payload::schema::{ContextId, Message, Payload},
    utp::protocol::UTPStream,
};

pub struct ContextWriter<S: UTPStream> {
    pub(crate) context_id: ContextId,
    pub(crate) utp_stream: Arc<S>,
}

impl<S: UTPStream> ContextWriter<S> {
    pub async fn write(&self, payload: Payload) -> Result<(), ConnectionError> {
        send_frame(
            self.utp_stream.as_ref(),
            Message {
                context_id: self.context_id,
                payload,
            },
        )
        .await
        .map_err(ConnectionError::UTP)
    }
}

pub struct ContextReader {
    pub(crate) receiver: tokio::sync::Mutex<Receiver<Payload>>,
}

impl ContextReader {
    pub async fn read(&self) -> Result<Payload, ConnectionError> {
        self.receiver
            .lock()
            .await
            .recv()
            .await
            .ok_or(ConnectionError::ClosedStream)
    }
}

pub type Context<S> = (ContextWriter<S>, ContextReader);
