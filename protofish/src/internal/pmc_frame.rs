use std::sync::Arc;

use bytes::Bytes;
use dashmap::DashMap;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
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

pub struct PMCFrame<U>
where
    U: UTPStream,
{
    senders: SenderMap,
    context_rx: Mutex<UnboundedReceiver<Message>>,
    writer: Mutex<U::StreamWrite>,
    shutdown_notify: Arc<Notify>,
    _task: JoinHandle<()>,
}

impl<U> PMCFrame<U>
where
    U: UTPStream,
{
    pub fn new(stream: U) -> Self {
        let senders: SenderMap = Default::default();
        let (context_tx, context_rx) = mpsc::unbounded_channel();
        let shutdown_notify = Arc::new(Notify::new());

        let (writer, mut reader) = stream.split();

        let _task = {
            let senders = senders.clone();
            let notify = shutdown_notify.clone();

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = notify.notified() => {
                            break;
                        }
                        success = match_frame(&mut reader, senders.clone(), context_tx.clone()) => {
                            if !success {break;}
                        }
                    }
                }
            })
        };

        Self {
            senders,
            context_rx: Mutex::new(context_rx),
            shutdown_notify,
            writer: Mutex::new(writer),
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

    pub async fn send_frame(&self, message: Message) -> Result<(), UTPError> {
        let buf = serialize_message(message);

        let len: u64 = buf.len() as u64;
        let len_bytes = len.to_le_bytes();
        let len_bytes = Bytes::copy_from_slice(&len_bytes);

        let mut writer = self.writer.lock().await;
        writer.write_all(&len_bytes).await?;
        writer.write_all(&buf).await?;

        Ok(())
    }
}

impl<U: UTPStream> Drop for PMCFrame<U> {
    fn drop(&mut self) {
        self.shutdown_notify.notify_waiters();
    }
}

async fn match_frame<R: AsyncRead + Unpin>(
    stream: &mut R,
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
        Err(UTPError::Io(_)) => {
            // stream closed
            false
        }
    }
}

async fn recv_frame<R: AsyncRead + Unpin>(stream: &mut R) -> Result<Option<Message>, UTPError> {
    let len = stream.read_u64_le().await?;

    let mut buf = vec![0; len as usize];
    stream.read_exact(&mut buf).await?;

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
