use thiserror::Error;

#[derive(Error, Debug)]
pub enum UTPError {
    #[error("UTP unknown warn {0}")]
    Warn(String),

    #[error("UTP unknown fatal {0}")]
    Fatal(String),
}
