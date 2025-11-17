use thiserror::Error;

/// Errors that can occur in UTP operations.
#[derive(Error, Debug)]
pub enum UTPError {
    /// Non-fatal warning that may not prevent operation
    #[error("UTP unknown warn {0}")]
    Warn(String),

    /// Fatal error that prevents further operation
    #[error("UTP unknown fatal {0}")]
    Fatal(String),

    #[error("UTP IO error {0}")]
    Io(#[from] std::io::Error),
}
