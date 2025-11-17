use thiserror::Error;

use crate::{schema::Payload, utp::error::UTPError};

/// Errors that can occur during Protofish connection operations.
#[derive(Error, Debug)]
pub enum ConnectionError {
    /// Error from the underlying UTP layer
    #[error("UTP error: {0}")]
    UTP(#[from] UTPError),

    /// The stream or context was closed
    #[error("stream closed")]
    ClosedStream,

    /// The server rejected the handshake
    #[error("handshake rejected: {0}")]
    HandshakeReject(String),

    /// Received malformed or invalid data
    #[error("malformed data: {0}")]
    MalformedData(String),

    /// Received an unexpected payload type
    #[error("malformed payload: {0} {1:?}")]
    MalformedPayload(String, Payload),
}
