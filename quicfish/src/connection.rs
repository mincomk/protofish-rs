use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{Mutex, RwLock, mpsc};

use protofish::utp::error::UTPError;
use protofish::utp::{UTP, UTPEvent};
use protofish::{IntegrityType, StreamId};

use crate::datagram::DatagramRouter;
use crate::stream::QuicUTPStream;

pub struct QuicUTP {
    connection: Arc<quinn::Connection>,
    streams: Arc<RwLock<HashMap<StreamId, QuicUTPStream>>>,
    next_stream_id: AtomicU64,
    event_tx: mpsc::UnboundedSender<UTPEvent>,
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<UTPEvent>>>,
    datagram_router: Arc<DatagramRouter>,
}

impl QuicUTP {
    pub fn new(connection: quinn::Connection) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let connection = Arc::new(connection);
        let datagram_router = DatagramRouter::new(Arc::downgrade(&connection));

        let instance = Self {
            connection: Arc::clone(&connection),
            streams: Arc::new(RwLock::new(HashMap::new())),
            next_stream_id: AtomicU64::new(0),
            event_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
            datagram_router: Arc::clone(&datagram_router),
        };

        instance.spawn_stream_listener();
        datagram_router.spawn_listener((*connection).clone());

        instance
    }

    async fn add_unreliable_stream(&self, stream_id: StreamId) -> QuicUTPStream {
        let recv_queue = self.datagram_router.register_stream(stream_id).await;
        let stream =
            QuicUTPStream::new_unreliable(stream_id, Arc::clone(&self.datagram_router), recv_queue);

        self.streams.write().await.insert(stream_id, stream.clone());

        stream
    }

    async fn next_id(&self) -> StreamId {
        self.next_stream_id.fetch_add(1, Ordering::Relaxed)
    }

    fn spawn_stream_listener(&self) {
        let connection = Arc::clone(&self.connection);
        let event_tx = self.event_tx.clone();
        let streams: Arc<RwLock<HashMap<StreamId, QuicUTPStream>>> = Arc::clone(&self.streams);
        let next_stream_id = AtomicU64::new(self.next_stream_id.load(Ordering::Relaxed));

        tokio::spawn(async move {
            loop {
                match connection.accept_bi().await {
                    Ok((send, recv)) => {
                        let stream_id = next_stream_id.fetch_add(1, Ordering::Relaxed);
                        let stream = QuicUTPStream::new_reliable(stream_id, send, recv);

                        streams.write().await.insert(stream_id, stream);

                        if event_tx.send(UTPEvent::NewStream(stream_id)).is_err() {
                            break;
                        }
                    }
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
                let (send, recv) = self
                    .connection
                    .open_bi()
                    .await
                    .map_err(|e| UTPError::Fatal(format!("stream open error: {}", e)))?;

                let stream_id = self.next_id().await;
                let stream = QuicUTPStream::new_reliable(stream_id, send, recv);

                self.streams.write().await.insert(stream_id, stream.clone());

                Ok(stream)
            }
            IntegrityType::Unreliable => {
                let stream_id = self.next_id().await;
                Ok(self.add_unreliable_stream(stream_id).await)
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
                let streams = self.streams.read().await;
                if let Some(stream) = streams.get(&id) {
                    return Ok(stream.clone());
                }
                drop(streams);

                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            },
            IntegrityType::Unreliable => Ok(self.add_unreliable_stream(id).await),
        }
    }
}
