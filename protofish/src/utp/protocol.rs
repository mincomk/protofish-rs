use async_trait::async_trait;
use bytes::Bytes;

use crate::{
    schema::{IntegrityType, StreamId},
    utp::error::UTPError,
};

/// Trait defining the interface for a UTP stream.
///
/// A UTP stream is an independent logical channel for binary data transmission.
/// Streams can be either reliable (lossless) or unreliable (lossy) based on
/// the `IntegrityType` used when opening the stream.
#[async_trait]
pub trait UTPStream: Send + Sync + Unpin + 'static {
    /// Returns the unique identifier for this stream.
    fn id(&self) -> StreamId;

    /// Returns the integrity type for this stream.
    fn integrity_type(&self) -> IntegrityType;

    /// Sends binary data over this stream.
    ///
    /// # Errors
    ///
    /// Returns an error if the send operation fails.
    async fn send(&self, data: &Bytes) -> Result<(), UTPError>;

    /// Receives binary data from this stream.
    ///
    /// # Arguments
    ///
    /// * `len` - The number of bytes to receive
    ///
    /// # Errors
    ///
    /// Returns an error if the receive operation fails.
    async fn receive(&self, len: usize) -> Result<Bytes, UTPError>;

    /// Closes this stream.
    ///
    /// # Errors
    ///
    /// Returns an error if the close operation fails.
    async fn close(&self) -> Result<(), UTPError>;
}

/// Trait defining the Upstream Transport Protocol (UTP) interface.
///
/// UTP abstracts the underlying transport mechanism, allowing Protofish to
/// operate over various protocols. Implementations must provide connection
/// management, event notification, and stream operations.
///
/// The UTP specification includes:
/// - Stream open/close operations
/// - Send/receive operations
/// - Primary stream management
/// - Event notification for incoming streams
#[async_trait]
pub trait UTP: Send + Sync + 'static {
    /// The stream type used by this UTP implementation
    type Stream: UTPStream;

    /// Establishes the underlying transport connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails.
    async fn connect(&self) -> Result<(), UTPError>;

    /// Waits for the next UTP event.
    ///
    /// This method blocks until an event occurs, such as a new incoming
    /// stream or an unexpected connection close.
    async fn next_event(&self) -> UTPEvent;

    /// Opens a new stream with the specified integrity type.
    ///
    /// # Arguments
    ///
    /// * `integrity` - Either `Reliable` for lossless transmission or
    ///   `Unreliable` for lossy transmission
    ///
    /// # Errors
    ///
    /// Returns an error if stream creation fails.
    async fn new_stream(&self, integrity: IntegrityType) -> Result<Self::Stream, UTPError>;

    /// Waits for a stream with the given ID to be ready.
    ///
    /// This is typically used after receiving a `NewStream` event to
    /// obtain the actual stream object.
    ///
    /// # Arguments
    ///
    /// * `id` - The stream ID from a `NewStream` event
    ///
    /// # Errors
    ///
    /// Returns an error if the stream cannot be obtained.
    async fn wait_stream(
        &self,
        id: StreamId,
        integrity: IntegrityType,
    ) -> Result<Self::Stream, UTPError>;
}

/// Events that can occur on a UTP connection.
#[derive(Clone, Debug)]
pub enum UTPEvent {
    /// The connection was unexpectedly closed
    UnexpectedClose,

    /// A new stream with the given ID has been opened by the peer
    NewStream(StreamId),
}
