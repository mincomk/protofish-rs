use std::sync::Arc;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    core::common::error::ConnectionError,
    internal::pmc_frame::send_frame,
    schema::{ContextId, Message, Payload},
    utp::UTPStream,
};

/// Writer half of a context, used to send payloads within a specific context.
///
/// Each context has a unique context ID that groups related messages together.
/// The context system provides strict ordering and grouping guarantees within
/// each context.
pub struct ContextWriter<S: UTPStream> {
    pub(crate) context_id: ContextId,
    pub(crate) utp_stream: Arc<S>,
}

impl<S: UTPStream> ContextWriter<S> {
    /// Writes a payload to this context.
    ///
    /// The payload will be wrapped in a `Message` with this context's ID
    /// and sent over the UTP stream.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying stream write fails.
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

/// Reader half of a context, used to receive payloads within a specific context.
///
/// Messages received on this context are delivered in order via an unbounded
/// channel.
pub struct ContextReader {
    pub(crate) receiver: tokio::sync::Mutex<UnboundedReceiver<Payload>>,
}

impl ContextReader {
    /// Reads the next payload from this context.
    ///
    /// This method blocks until a message arrives on this context.
    ///
    /// # Returns
    ///
    /// Returns the next `Payload`, or an error if the context is closed.
    ///
    /// # Errors
    ///
    /// Returns `ConnectionError::ClosedStream` if the context channel is closed.
    pub async fn read(&self) -> Result<Payload, ConnectionError> {
        self.receiver
            .lock()
            .await
            .recv()
            .await
            .ok_or(ConnectionError::ClosedStream)
    }
}

/// A context consisting of a writer and reader pair.
///
/// Contexts provide strict grouping and ordering of messages, enabling
/// conversational patterns in communication.
pub type Context<S> = (ContextWriter<S>, ContextReader);
