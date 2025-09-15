use prost::EncodeError;
use thiserror::Error;

use crate::{core::common::error::ConnectionError, utp::error::UTPError};

#[derive(Error, Debug)]
pub enum ProtofishError {
    #[error("UTP error: {0}")]
    UTP(#[from] UTPError),

    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    #[error("Encode error: {0}")]
    Encode(#[from] EncodeError),
}
