use thiserror::Error;

use crate::utp::error::UTPError;

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("UTP error {0}")]
    UTP(#[from] UTPError),

    #[error("stream closed")]
    ClosedStream,
}
