use thiserror::Error;

#[derive(Error, Debug)]
pub enum UTPError {
    #[error("unknown error {0}")]
    Unknown(String),
}
