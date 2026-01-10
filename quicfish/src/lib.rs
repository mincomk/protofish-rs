pub mod config;
pub mod connection;
pub mod datagram;
pub mod endpoint;
pub mod error;
pub mod stream;

pub type Connection = protofish::Connection<QuicUTP>;
pub type ArbContext = protofish::ArbContext<QuicUTP>;

use std::io::BufRead;

pub use config::QuicConfig;
pub use connection::QuicUTP;
pub use endpoint::{QuicEndpoint, QuicEndpointBuilder};
pub use error::{Error, Result};
pub use stream::QuicUTPStream;

mod tls;

/// Connects to a QUIC server at the specified address using the provided server name and
/// certificate.
pub async fn connect(
    addr: std::net::SocketAddr,
    server_name: &str,
    cert: &mut impl BufRead,
) -> error::Result<protofish::Connection<QuicUTP>> {
    let client_crypto = tls::create_client_config(cert)?;
    let quic_config = QuicConfig::client_default().with_client_crypto(client_crypto);
    let endpoint = QuicEndpoint::client("0.0.0.0:0".parse().unwrap(), quic_config)?;

    let connection = endpoint.connect(addr, server_name).await?;
    let utp = QuicUTP::new(connection, false);
    let pf_conn = protofish::connect(utp.into(), server_name).await?;

    Ok(pf_conn)
}

/// Creates a QUIC server endpoint bound to the specified address using the provided certificate
/// and key.
pub async fn create_server_endpoint(
    bind_addr: std::net::SocketAddr,
    cert: &mut impl BufRead,
    key: &mut impl BufRead,
) -> error::Result<QuicEndpoint> {
    let server_crypto = tls::create_server_config(cert, key)?;
    let quic_config = QuicConfig::server_default().with_server_crypto(server_crypto);
    let endpoint = QuicEndpoint::server(bind_addr, quic_config)?;

    Ok(endpoint)
}
