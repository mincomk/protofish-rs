use std::sync::Arc;

use bytes::Bytes;

use crate::{
    constant::VERSION,
    core::common::{
        connection::Connection,
        context::{ContextReader, ContextWriter},
        error::ConnectionError,
        pmc::PMC,
    },
    error::ProtofishError,
    schema::{ClientHello, IntegrityType, Payload},
    utp::{UTP, UTPStream},
};

/// Establishes a Protofish connection as a client.
///
/// This function performs the following steps:
/// 1. Connects to the server via the provided UTP implementation
/// 2. Opens a reliable stream for the Primary Messaging Channel (PMC)
/// 3. Performs the client-side handshake by sending `ClientHello`
/// 4. Returns a `Connection` if the handshake succeeds
///
/// # Arguments
///
/// * `utp` - An Arc-wrapped UTP implementation for the underlying transport
///
/// # Returns
///
/// Returns a `Connection` on successful handshake, or a `ProtofishError` if
/// the connection or handshake fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The UTP connection fails
/// - Opening the stream fails
/// - The server rejects the handshake
pub async fn connect<U>(utp: Arc<U>) -> Result<Connection<U>, ProtofishError>
where
    U: UTP,
{
    utp.connect().await?;

    let stream = utp.new_stream(IntegrityType::Reliable).await?;
    let pmc = PMC::new(false, stream);

    let _ = client_handshake(pmc.create_context(), None).await?;

    Ok(Connection::new(utp.clone(), pmc))
}

async fn client_handshake<S: UTPStream>(
    ctx: (ContextWriter<S>, ContextReader),
    resume_token: Option<Bytes>,
) -> Result<Bytes, ProtofishError> {
    let (tx, rx) = ctx;

    let client_hello = ClientHello {
        version: VERSION,
        resume_connection_token: resume_token.map(Into::into),
    };

    tx.write(Payload::ClientHello(client_hello)).await?;

    let server_hello = rx.read().await?;

    if let Payload::ServerHello(server_hello) = server_hello {
        if server_hello.ok {
            Ok(server_hello
                .connection_token
                .ok_or(ProtofishError::Connection(ConnectionError::MalformedData(
                    "connection token is not provided".into(),
                )))?)
        } else {
            let msg = server_hello.message.unwrap_or("unknown error".to_string());

            Err(ProtofishError::Connection(
                ConnectionError::HandshakeReject(msg),
            ))
        }
    } else {
        Err(ProtofishError::Connection(
            ConnectionError::MalformedPayload("expected ServerHello".into(), server_hello),
        ))
    }
}

#[cfg(test)]
mod tests {

    use bytes::BytesMut;

    use crate::{
        constant::VERSION,
        core::{client::client::client_handshake, common::pmc::PMC},
        schema::{Payload, ServerHello},
        utp::tests::stream::mock_utp_stream_pairs,
    };

    #[tokio::test]
    async fn test_client_handshake_ok() {
        let (client_stream, server_stream) = mock_utp_stream_pairs(0);

        let server_pmc = PMC::new(true, server_stream);
        let client_pmc = PMC::new(false, client_stream);

        tokio::spawn(async move {
            let (tx, rx) = server_pmc.next_context().await.unwrap();
            let payload = rx.read().await.unwrap();

            if let Payload::ClientHello(_) = payload {
                tx.write(Payload::ServerHello(ServerHello {
                    ok: true,
                    connection_token: Some(BytesMut::zeroed(20).freeze()),
                    message: None,
                    version: VERSION,
                }))
                .await
                .unwrap();
            } else {
                panic!("expected ClientHello, got: {:?}", payload);
            }
        });

        let ctx = client_pmc.create_context();
        client_handshake(ctx, Some(BytesMut::zeroed(30).freeze()))
            .await
            .unwrap();
    }
}
