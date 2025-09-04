use std::sync::Arc;

use tokio::sync::mpsc::Receiver;

use crate::{
    core::common::error::StreamError,
    internal::pmc_frame::send_frame,
    schema::payload::schema::{ContextId, Message, Payload, StreamId},
    utp::protocol::UTP,
};

pub struct ContextWriter<U: UTP> {
    pub(crate) context_id: ContextId,
    pub(crate) stream_id: StreamId,
    pub(crate) utp: Arc<U>,
}

impl<U: UTP> ContextWriter<U> {
    pub async fn write(&self, payload: Payload) -> Result<(), StreamError> {
        send_frame(
            self.utp.as_ref(),
            self.stream_id,
            Message {
                context_id: self.context_id,
                payload,
            },
        )
        .await
        .map_err(|e| StreamError::UTP(e))
    }
}

pub struct ContextReader {
    pub(crate) receiver: tokio::sync::Mutex<Receiver<Payload>>,
}

impl ContextReader {
    pub async fn read(&self) -> Result<Payload, StreamError> {
        self.receiver
            .lock()
            .await
            .recv()
            .await
            .ok_or(StreamError::ClosedStream)
    }
}
