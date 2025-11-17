use std::sync::Arc;

use bytes::Bytes;
use dashmap::DashMap;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{
        Mutex, Notify,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
    task::JoinHandle,
};

use crate::{
    internal::serialize::{deserialize_message, serialize_message},
    schema::{ContextId, Message, Payload},
    utp::{UTPStream, error::UTPError},
};

type SenderMap = Arc<DashMap<ContextId, UnboundedSender<Payload>>>;

pub struct PMCFrame {
    senders: SenderMap,
    context_rx: Mutex<UnboundedReceiver<Message>>,
    _task: JoinHandle<()>,
}

impl PMCFrame {
    pub fn new(stream: Arc<impl UTPStream>) -> Self {
        let senders: SenderMap = Default::default();
        let (context_tx, context_rx) = mpsc::unbounded_channel();
        let shutdown_notify = Arc::new(Notify::new());

        let _task = {
            let stream = stream.clone();
            let senders = senders.clone();
            let notify = shutdown_notify.clone();

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = notify.notified() => {
                            break;
                        }
                        success = match_frame(stream.clone(), senders.clone(), context_tx.clone()) => {
                            if !success {break;}
                        }
                    }
                }
            })
        };

        Self {
            senders,
            context_rx: Mutex::new(context_rx),
            _task,
        }
    }

    pub fn subscribe_context(
        &self,
        context_id: ContextId,
        initial_item: Option<Payload>,
    ) -> UnboundedReceiver<Payload> {
        let (tx, rx) = mpsc::unbounded_channel();

        if let Some(item) = initial_item {
            send_curried(tx.clone())(item);
        }

        self.senders.insert(context_id, tx);

        rx
    }

    pub async fn next_context_message(&self) -> Option<Message> {
        self.context_rx.lock().await.recv().await
    }
}

async fn match_frame(
    stream: Arc<impl UTPStream>,
    senders: SenderMap,
    context_tx: UnboundedSender<Message>,
) -> bool {
    match recv_frame(stream).await {
        Ok(message_option) => {
            if let Some(message) = message_option {
                if let Some(sender) = senders.get(&message.context_id) {
                    send_curried(sender.clone())(message.payload);
                } else {
                    send_curried(context_tx)(message);
                }

                true
            } else {
                false
            }
        }
        Err(UTPError::Fatal(e)) => {
            tracing::error!("UTP receive failure: {}", e);
            false
        }
        Err(UTPError::Warn(e)) => {
            tracing::warn!("UTP receive warn: {}", e);
            true
        }
        Err(UTPError::Io(e)) => {
            tracing::error!("UTP receive IO error: {}", e);
            true
        }
    }
}

pub async fn send_frame<S: UTPStream>(stream: &S, message: Message) -> Result<(), UTPError> {
    let buf = serialize_message(message);

    let len: u64 = buf.len() as u64;
    let len_bytes = len.to_le_bytes();
    let len_bytes = Bytes::copy_from_slice(&len_bytes);

    stream.writer().write_all(&len_bytes).await?;
    stream.writer().write_all(&buf).await?;

    Ok(())
}

async fn recv_frame<S: UTPStream>(stream: Arc<S>) -> Result<Option<Message>, UTPError> {
    let len = stream.reader().read_u64_le().await?;

    let mut buf = vec![0; len as usize];
    stream.reader().read_exact(&mut buf).await?;

    let message = deserialize_message(&buf);

    Ok(message)
}

fn send_curried<T>(sender: impl Into<UnboundedSender<T>>) -> impl Fn(T) {
    let sender = sender.into().clone();
    move |data: T| {
        if let Err(e) = sender.send(data) {
            tracing::warn!("Send error while sending to the channel: {:?}", e);
        }
    }
}
