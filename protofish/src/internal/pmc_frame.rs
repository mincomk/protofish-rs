use std::sync::Arc;

use bytes::BytesMut;
use dashmap::DashMap;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    constant::CHANNEL_BUFFER,
    internal::serialize::{deserialize_message, serialize_message},
    schema::payload::schema::{ContextId, Message, Payload, StreamId},
    utp::{error::UTPError, protocol::UTP},
};

pub struct PMCFrame<U: UTP> {
    utp: Arc<U>,
    senders: Arc<DashMap<ContextId, Sender<Payload>>>,
    stream_id: StreamId,
    _task: JoinHandle<()>,
}

impl<U: UTP> PMCFrame<U> {
    pub fn new(utp: U, stream_id: StreamId) -> Self {
        let utp = Arc::new(utp);
        let senders: Arc<DashMap<ContextId, Sender<Payload>>> = Default::default();

        let _task = {
            let utp = utp.clone();
            let senders = senders.clone();

            tokio::spawn(async move {
                while let Ok(Some(message)) = recv_frame(utp.as_ref(), stream_id).await {
                    if let Some(sender) = senders.get(&message.context_id) {
                        tokio::spawn({
                            let sender = sender.clone();
                            async move {
                                sender.send(message.payload).await;
                            }
                        });
                    }
                }
            })
        };

        Self {
            utp,
            senders,
            stream_id,
            _task,
        }
    }

    pub fn subscribe_context(&self, context_id: ContextId) -> Receiver<Payload> {
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER);

        self.senders.insert(context_id, tx);

        rx
    }

    pub async fn send(&self, message: Message) -> Result<(), UTPError> {
        send_frame(self.utp.as_ref(), self.stream_id, message).await
    }
}

pub async fn send_frame<U: UTP>(
    utp: &U,
    stream_id: StreamId,
    message: Message,
) -> Result<(), UTPError> {
    let buf = serialize_message(message);

    let len: u64 = buf.len() as u64;
    let len_bytes = len.to_le_bytes();

    utp.send(stream_id, &len_bytes).await?;
    utp.send(stream_id, &buf).await?;

    Ok(())
}

async fn recv_frame<U: UTP>(utp: &U, stream_id: StreamId) -> Result<Option<Message>, UTPError> {
    let mut len_bytes = [0u8; 8];
    utp.receive(stream_id, &mut len_bytes).await?;

    let len = u64::from_le_bytes(len_bytes);

    let mut buf = BytesMut::with_capacity(len as usize);
    utp.receive(stream_id, &mut buf).await?;

    let message = deserialize_message(&buf);

    Ok(message)
}
