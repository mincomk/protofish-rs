use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::Mutex;

use protofish::utp::UTPStream;
use protofish::utp::error::UTPError;
use protofish::{IntegrityType, StreamId};

use crate::datagram::DatagramRouter;

pub struct QuicUTPStream {
    id: StreamId,
    inner: StreamInner,
}

enum StreamInner {
    Reliable(ReliableStream),
    Unreliable(UnreliableStream),
}

struct ReliableStream {
    send: Arc<Mutex<quinn::SendStream>>,
    recv: Arc<Mutex<quinn::RecvStream>>,
}

struct UnreliableStream {
    router: Arc<DatagramRouter>,
    recv_queue: Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<Bytes>>>,
}

impl QuicUTPStream {
    pub fn new_reliable(id: StreamId, send: quinn::SendStream, recv: quinn::RecvStream) -> Self {
        Self {
            id,
            inner: StreamInner::Reliable(ReliableStream {
                send: Arc::new(Mutex::new(send)),
                recv: Arc::new(Mutex::new(recv)),
            }),
        }
    }

    pub fn new_unreliable(
        id: StreamId,
        router: Arc<DatagramRouter>,
        recv_queue: tokio::sync::mpsc::UnboundedReceiver<Bytes>,
    ) -> Self {
        Self {
            id,
            inner: StreamInner::Unreliable(UnreliableStream {
                router,
                recv_queue: Arc::new(Mutex::new(recv_queue)),
            }),
        }
    }
}

#[async_trait]
impl UTPStream for QuicUTPStream {
    fn id(&self) -> StreamId {
        self.id
    }

    fn integrity_type(&self) -> IntegrityType {
        match &self.inner {
            StreamInner::Reliable(_) => IntegrityType::Reliable,
            StreamInner::Unreliable(_) => IntegrityType::Unreliable,
        }
    }

    async fn send(&self, data: &[u8]) -> Result<(), UTPError> {
        match &self.inner {
            StreamInner::Reliable(stream) => {
                let mut send = stream.send.lock().await;
                send.write_all(data)
                    .await
                    .map_err(|e| UTPError::Fatal(format!("send error: {}", e)))?;
                Ok(())
            }
            StreamInner::Unreliable(stream) => {
                stream
                    .router
                    .send_datagram(self.id, &Bytes::copy_from_slice(data))
                    .await
                    .map_err(|e| UTPError::Fatal(format!("send datagram error: {}", e)))?;
                Ok(())
            }
        }
    }

    async fn receive(&self, buf: &mut [u8]) -> Result<(), UTPError> {
        match &self.inner {
            StreamInner::Reliable(stream) => {
                let mut recv = stream.recv.lock().await;

                recv.read_exact(buf)
                    .await
                    .map_err(|err| UTPError::Fatal(format!("receive error: {}", err)))
            }
            StreamInner::Unreliable(stream) => {
                let mut queue = stream.recv_queue.lock().await;
                let bytes = queue
                    .recv()
                    .await
                    .ok_or_else(|| UTPError::Fatal("connection closed".to_string()))?;
                buf[..bytes.len()].copy_from_slice(&bytes);

                Ok(())
            }
        }
    }

    async fn close(&self) -> Result<(), UTPError> {
        match &self.inner {
            StreamInner::Reliable(stream) => {
                let mut send = stream.send.lock().await;
                send.finish()
                    .map_err(|e| UTPError::Fatal(format!("close error: {}", e)))?;
                Ok(())
            }
            StreamInner::Unreliable(_) => Ok(()),
        }
    }
}

impl Clone for QuicUTPStream {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: match &self.inner {
                StreamInner::Reliable(stream) => StreamInner::Reliable(ReliableStream {
                    send: Arc::clone(&stream.send),
                    recv: Arc::clone(&stream.recv),
                }),
                StreamInner::Unreliable(stream) => StreamInner::Unreliable(UnreliableStream {
                    router: Arc::clone(&stream.router),
                    recv_queue: Arc::clone(&stream.recv_queue),
                }),
            },
        }
    }
}
