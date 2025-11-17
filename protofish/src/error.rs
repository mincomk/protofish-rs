use prost::EncodeError;
use thiserror::Error;

use crate::{core::common::error::ConnectionError, utp::error::UTPError};

/// Main error type for Protofish operations.
///
/// This error type encompasses all possible errors that can occur during
/// Protofish protocol operations, including UTP transport errors, connection
/// errors, and message encoding errors.
#[derive(Error, Debug)]
pub enum ProtofishError {
    /// Error from the underlying UTP transport layer
    #[error("UTP error: {0}")]
    UTP(#[from] UTPError),

    /// Error during connection operations like handshake or message handling
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    /// Error encoding protobuf messages
    #[error("Encode error: {0}")]
    Encode(#[from] EncodeError),
}
