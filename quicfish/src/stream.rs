use bytes::Bytes;
use std::io::ErrorKind;
use std::sync::Arc;
use std::task::Poll;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadHalf, SimplexStream};
use tokio::sync::Mutex;

use protofish::utp::UTPStream;
use protofish::{IntegrityType, StreamId};

use crate::datagram::DatagramRouter;

#[derive(Clone)]
pub struct QuicUTPStream {
    id: StreamId,
    inner: StreamInner,
}

#[derive(Clone)]
enum StreamInner {
    Reliable(ReliableStream),
    Unreliable(UnreliableStream),
}

#[derive(Clone)]
struct ReliableStream {
    send: Arc<Mutex<quinn::SendStream>>,
    recv: Arc<Mutex<quinn::RecvStream>>,
}

#[derive(Clone)]
struct UnreliableStream {
    router: DatagramRouter,
    stream_id: StreamId,
    read_half: Arc<Mutex<ReadHalf<SimplexStream>>>,
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

    pub fn new_unreliable(id: StreamId, router: DatagramRouter) -> Self {
        let read_half = router.register(id);

        Self {
            id,
            inner: StreamInner::Unreliable(UnreliableStream {
                router,
                stream_id: id,
                read_half: Arc::new(read_half.into()),
            }),
        }
    }
}

impl UTPStream for QuicUTPStream {
    type StreamRead = Self;
    type StreamWrite = Self;

    fn id(&self) -> StreamId {
        self.id
    }

    fn integrity_type(&self) -> IntegrityType {
        match &self.inner {
            StreamInner::Reliable(_) => IntegrityType::Reliable,
            StreamInner::Unreliable(_) => IntegrityType::Unreliable,
        }
    }

    fn reader(&self) -> Self::StreamRead {
        self.clone()
    }

    fn writer(&self) -> Self::StreamWrite {
        self.clone()
    }
}

impl AsyncRead for QuicUTPStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Box::pin(async move {
            match self.inner {
                StreamInner::Reliable(ref reliable) => {
                    reliable.recv.lock().await.read_buf(buf).await?;

                    Ok(())
                }
                StreamInner::Unreliable(ref unreliable) => {
                    unreliable.read_half.lock().await.read_buf(buf).await?;

                    Ok(())
                }
            }
        })
        .as_mut()
        .poll(cx)
    }
}

impl AsyncWrite for QuicUTPStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Box::pin(async move {
            match self.inner {
                StreamInner::Reliable(ref reliable) => reliable
                    .send
                    .lock()
                    .await
                    .write(buf)
                    .await
                    .map_err(|err| std::io::Error::new(ErrorKind::Other, err.to_string())),
                StreamInner::Unreliable(ref unreliable) => {
                    unreliable
                        .router
                        .write(unreliable.stream_id, Bytes::copy_from_slice(buf))
                        .map_err(|err| std::io::Error::new(ErrorKind::Other, err.to_string()))?;

                    Ok(buf.len())
                }
            }
        })
        .as_mut()
        .poll(cx)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}
