use std::sync::Arc;

use bytes::Bytes;

use crate::{
    constant::VERSION,
    core::common::{
        context::{ContextReader, ContextWriter},
        error::ConnectionError,
        pmc::PMC,
    },
    error::ProtofishError,
    schema::{
        common::schema::IntegrityType,
        payload::schema::{ClientHello, Payload},
    },
    utp::protocol::{UTP, UTPStream},
};

pub async fn connect<S: UTPStream>(utp: Arc<impl UTP<S>>) -> Result<(), ProtofishError> {
    utp.connect().await?;

    let stream = utp.open_stream(IntegrityType::Reliable).await?;
    let pmc = PMC::new(false, stream);

    let connection_token = client_handshake(pmc.create_context(), None).await?;

    Ok(())
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
                .ok_or(ProtofishError::Connection(
                    ConnectionError::MalformedMessage,
                ))?)
        } else {
            let msg = server_hello.message.unwrap_or("unknown error".to_string());

            Err(ProtofishError::Connection(
                ConnectionError::HandshakeReject(msg),
            ))
        }
    } else {
        Err(ProtofishError::Connection(
            ConnectionError::MalformedMessage,
        ))
    }
}

#[cfg(test)]
mod tests {

    use bytes::BytesMut;

    use crate::{
        constant::VERSION,
        core::common::{client::client_handshake, pmc::PMC},
        schema::payload::schema::{Payload, ServerHello},
        utp::tests::stream::mock_pairs,
    };

    #[tokio::test]
    async fn test_client_handshake_ok() {
        let (client_stream, server_stream) = mock_pairs();

        let server_pmc = PMC::new(true, server_stream);
        let client_pmc = PMC::new(false, client_stream);

        tokio::spawn(async move {
            let (payload, (tx, _)) = server_pmc.next_context().await.unwrap();

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
