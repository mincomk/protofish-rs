use thiserror::Error;

use crate::utp::error::UTPError;

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("UTP error: {0}")]
    UTP(#[from] UTPError),

    #[error("stream closed")]
    ClosedStream,

    #[error("handshake rejected: {0}")]
    HandshakeReject(String),

    #[error("malformed message")]
    MalformedMessage,
}
