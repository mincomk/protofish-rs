use std::{io::Read, sync::Arc};

use bytes::{Bytes, BytesMut};
use dashmap::DashMap;
use tokio::{
    sync::{
        Mutex,
        mpsc::{self, Receiver, Sender},
    },
    task::JoinHandle,
};

use crate::{
    constant::CHANNEL_BUFFER,
    internal::serialize::{deserialize_message, serialize_message},
    schema::payload::schema::{ContextId, Message, Payload},
    utp::{error::UTPError, protocol::UTPStream},
};

pub struct PMCFrame<S: UTPStream> {
    utp_stream: Arc<S>,
    senders: Arc<DashMap<ContextId, Sender<Payload>>>,
    context_rx: Mutex<Receiver<Message>>,
    _task: JoinHandle<()>,
}

impl<S: UTPStream> PMCFrame<S> {
    pub fn new(stream: Arc<S>) -> Self {
        let senders: Arc<DashMap<ContextId, Sender<Payload>>> = Default::default();
        let (context_tx, context_rx) = mpsc::channel(CHANNEL_BUFFER);

        let _task = {
            let stream = stream.clone();
            let senders = senders.clone();

            tokio::spawn(async move {
                loop {
                    match recv_frame(stream.as_ref()).await {
                        Ok(message_option) => {
                            if let Some(message) = message_option {
                                if let Some(sender) = senders.get(&message.context_id) {
                                    tokio::spawn({
                                        let sender = sender.clone();
                                        async move {
                                            if let Err(e) = sender.send(message.payload).await {
                                                tracing::warn!(
                                                    "UTP error while sending to the channel: {:?}",
                                                    e
                                                );
                                            }
                                        }
                                    });
                                } else {
                                    tokio::spawn({
                                        let context_tx = context_tx.clone();
                                        async move {
                                            if let Err(e) = context_tx.send(message).await {
                                                tracing::warn!(
                                                    "Send error while sending to the channel: {:?}",
                                                    e
                                                );
                                            }
                                        }
                                    });
                                }
                            } else {
                                break;
                            }
                        }
                        Err(UTPError::Fatal(e)) => {
                            tracing::error!("UTP receive failure: {}", e);
                            break;
                        }
                        Err(UTPError::Warn(e)) => {
                            tracing::warn!("UTP receive warn: {}", e);
                        }
                    }
                }
            })
        };

        Self {
            utp_stream: stream,
            senders,
            context_rx: Mutex::new(context_rx),
            _task,
        }
    }

    pub fn subscribe_context(&self, context_id: ContextId) -> Receiver<Payload> {
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER);

        self.senders.insert(context_id, tx);

        rx
    }

    pub async fn next_context_message(&self) -> Option<Message> {
        self.context_rx.lock().await.recv().await
    }

    pub async fn send(&self, message: Message) -> Result<(), UTPError> {
        send_frame(self.utp_stream.as_ref(), message).await
    }
}

pub async fn send_frame<S: UTPStream>(stream: &S, message: Message) -> Result<(), UTPError> {
    let buf = serialize_message(message);

    let len: u64 = buf.len() as u64;
    let len_bytes = len.to_le_bytes();
    let len_bytes = Bytes::copy_from_slice(&len_bytes);

    stream.send(&len_bytes).await?;
    stream.send(&buf).await?;

    Ok(())
}

async fn recv_frame<S: UTPStream>(stream: &S) -> Result<Option<Message>, UTPError> {
    let mut len_bytes = BytesMut::zeroed(8);
    stream.receive(&mut len_bytes).await?;

    let len_bytes_slice = &len_bytes[..];

    let len = u64::from_le_bytes(len_bytes_slice.try_into().unwrap());
    // .with_capacity(8) -> safe to unwrap

    let mut buf = BytesMut::zeroed(len as usize);
    stream.receive(&mut buf).await?;

    let message = deserialize_message(&buf);

    Ok(message)
}
