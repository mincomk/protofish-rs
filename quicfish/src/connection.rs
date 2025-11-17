use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, mpsc};

use protofish::utp::error::UTPError;
use protofish::utp::{UTP, UTPEvent};
use protofish::{IntegrityType, StreamId};

use crate::datagram::DatagramRouter;
use crate::stream::QuicUTPStream;

pub struct QuicUTP {
    connection: Arc<quinn::Connection>,
    streams: Arc<DashMap<StreamId, QuicUTPStream>>,
    next_stream_id: AtomicU64,
    event_tx: mpsc::UnboundedSender<UTPEvent>,
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<UTPEvent>>>,
    datagram_router: DatagramRouter,
}

impl QuicUTP {
    pub fn new(connection: quinn::Connection, is_server: bool) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let connection = Arc::new(connection);
        let streams = Arc::new(DashMap::new());
        // unwrap-safe since we've set it in config
        let datagram_router =
            DatagramRouter::new(connection.clone(), connection.max_datagram_size().unwrap());

        let instance = Self {
            connection: Arc::clone(&connection),
            streams,
            next_stream_id: AtomicU64::new(if is_server { 1 } else { 0 }),
            event_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
            datagram_router: datagram_router.clone(),
        };

        instance.spawn_stream_listener();
        datagram_router.spawn_listener();

        instance
    }

    fn add_unreliable_stream(&self, stream_id: StreamId) -> QuicUTPStream {
        let stream = QuicUTPStream::new_unreliable(stream_id, self.datagram_router.clone());

        self.streams.insert(stream_id, stream.clone());

        stream
    }

    fn next_id(&self) -> StreamId {
        self.next_stream_id.fetch_add(2, Ordering::Relaxed)
    }

    fn spawn_stream_listener(&self) {
        let connection = Arc::clone(&self.connection);
        let event_tx = self.event_tx.clone();
        let streams: Arc<DashMap<StreamId, QuicUTPStream>> = Arc::clone(&self.streams);

        tokio::spawn(async move {
            loop {
                match connection.accept_bi().await {
                    Ok((send, mut recv)) => match recv.read_u64_le().await {
                        Ok(stream_id) => {
                            let stream = QuicUTPStream::new_reliable(stream_id, send, recv);

                            streams.insert(stream_id, stream);

                            if event_tx.send(UTPEvent::NewStream(stream_id)).is_err() {
                                break;
                            }
                        }
                        Err(_) => {
                            let _ = event_tx.send(UTPEvent::UnexpectedClose);
                        }
                    },
                    Err(_) => {
                        let _ = event_tx.send(UTPEvent::UnexpectedClose);
                        break;
                    }
                }
            }
        });
    }
}

#[async_trait]
impl UTP for QuicUTP {
    type Stream = QuicUTPStream;

    async fn connect(&self) -> Result<(), UTPError> {
        Ok(())
    }

    async fn next_event(&self) -> UTPEvent {
        let mut rx = self.event_rx.lock().await;
        rx.recv().await.unwrap_or(UTPEvent::UnexpectedClose)
    }

    async fn new_stream(&self, integrity: IntegrityType) -> Result<Self::Stream, UTPError> {
        match integrity {
            IntegrityType::Reliable => {
                let (mut send, recv) = self
                    .connection
                    .open_bi()
                    .await
                    .map_err(|e| UTPError::Fatal(format!("stream open error: {}", e)))?;

                let stream_id = self.next_id();

                send.write_u64_le(stream_id).await?;

                let stream = QuicUTPStream::new_reliable(stream_id, send, recv);

                self.streams.insert(stream_id, stream.clone());

                Ok(stream)
            }
            IntegrityType::Unreliable => {
                let stream_id = self.next_id();
                Ok(self.add_unreliable_stream(stream_id))
            }
        }
    }

    async fn wait_stream(
        &self,
        id: StreamId,
        integrity: IntegrityType,
    ) -> Result<Self::Stream, UTPError> {
        match integrity {
            IntegrityType::Reliable => loop {
                if let Some(stream) = self.streams.get(&id) {
                    return Ok(stream.clone());
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            },
            IntegrityType::Unreliable => Ok(self.add_unreliable_stream(id)),
        }
    }
}
