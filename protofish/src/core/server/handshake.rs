use bytes::Bytes;

use crate::{
    constant::VERSION,
    core::{
        common::{context::Context, error::ConnectionError, pmc::PMC},
        server::token::generate_connection_token,
    },
    error::ProtofishError,
    schema::{Payload, ServerHello},
    utp::UTPStream,
};

pub async fn server_handshake<S: UTPStream>(pmc: &PMC<S>) -> Result<(), ProtofishError> {
    let ctx = get_client_hello(pmc).await?;
    let payload = ctx.1.read().await?;

    if let Payload::ClientHello(client_hello) = payload {
        if client_hello.resume_connection_token.is_some() {
            reject_client(ctx, "Resume connection is not supported.").await
        } else {
            accept_client(ctx, generate_connection_token()).await
        }
    } else {
        Err(ConnectionError::MalformedPayload("expected ClientHello".into(), payload).into())
    }
}

async fn accept_client<S: UTPStream>(
    ctx: Context<S>,
    connection_token: Bytes,
) -> Result<(), ProtofishError> {
    let (tx, _) = ctx;

    let server_hello = ServerHello {
        version: VERSION,
        ok: true,
        connection_token: Some(connection_token),

        message: None,
    };

    tx.write(Payload::ServerHello(server_hello)).await?;

    Ok(())
}

async fn reject_client<S: UTPStream>(ctx: Context<S>, message: &str) -> Result<(), ProtofishError> {
    let (tx, _) = ctx;

    let server_hello = ServerHello {
        version: VERSION,
        ok: false,
        connection_token: None,
        message: Some(message.into()),
    };

    tx.write(Payload::ServerHello(server_hello)).await?;

    Ok(())
}

async fn get_client_hello<S: UTPStream>(pmc: &PMC<S>) -> Result<Context<S>, ProtofishError> {
    pmc.next_context()
        .await
        .ok_or(ProtofishError::from(ConnectionError::ClosedStream))
}
