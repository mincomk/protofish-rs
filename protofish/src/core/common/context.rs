use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::Mutex;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{
    constant::CHANNEL_BUFFER,
    core::common::{counter::ContextCounter, error::StreamError, stream::Stream},
    internal::pmc_frame::send_frame,
    prost_generated::{common, payload},
    schema::payload::schema::{Message, Payload, StreamId},
    utp::{error::UTPError, protocol::UTP},
};

pub type ContextId = u64;

pub struct ContextWriter<U: UTP> {
    context_id: ContextId,
    stream_id: StreamId,
    utp: Arc<U>,
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
    receiver: tokio::sync::Mutex<Receiver<Payload>>,
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
