use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("QUIC connection error: {0}")]
    Connection(#[from] quinn::ConnectionError),

    #[error("QUIC stream write error: {0}")]
    StreamWrite(#[from] quinn::WriteError),

    #[error("QUIC stream read error: {0}")]
    StreamRead(#[from] quinn::ReadError),

    #[error("Send datagram: {0}")]
    SendDatagram(#[from] quinn::SendDatagramError),

    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Datagram error: {0}")]
    Datagram(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("TLS error: {0}")]
    Tls(String),

    #[error("Connection not established")]
    NotConnected,

    #[error("Stream closed")]
    StreamClosed,
}

impl From<Error> for protofish::utp::error::UTPError {
    fn from(err: Error) -> Self {
        protofish::utp::error::UTPError::Fatal(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
