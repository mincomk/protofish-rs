pub mod config;
pub mod connection;
pub mod datagram;
pub mod endpoint;
pub mod error;
pub mod stream;

pub type Connection = protofish::Connection<QuicUTP>;
pub type ArbContext = protofish::ArbContext<QuicUTP>;

pub use config::QuicConfig;
pub use connection::QuicUTP;
pub use endpoint::{QuicEndpoint, QuicEndpointBuilder};
pub use error::{Error, Result};
pub use stream::QuicUTPStream;

// TODO: connect()
