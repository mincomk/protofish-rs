use bytes::Bytes;
use std::io::ErrorKind;
use std::task::Poll;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, SimplexStream};

use protofish::utp::UTPStream;
use protofish::{IntegrityType, StreamId};

use crate::datagram::DatagramRouter;

pub struct QuicUTPStream {
    id: StreamId,
    integrity_type: IntegrityType,
    writer: StreamWriteInner,
    reader: StreamReadInner,
}

pub enum StreamWriteInner {
    Reliable(quinn::SendStream),
    Unreliable(DatagramRouter, StreamId),
}

pub enum StreamReadInner {
    Reliable(quinn::RecvStream),
    Unreliable(ReadHalf<SimplexStream>),
}

impl QuicUTPStream {
    pub fn new_reliable(id: StreamId, send: quinn::SendStream, recv: quinn::RecvStream) -> Self {
        Self {
            id,
            integrity_type: IntegrityType::Reliable,
            writer: StreamWriteInner::Reliable(send),
            reader: StreamReadInner::Reliable(recv),
        }
    }

    pub fn new_unreliable(id: StreamId, router: DatagramRouter) -> Self {
        let read_half = router.register(id);

        Self {
            id,
            integrity_type: IntegrityType::Unreliable,
            writer: StreamWriteInner::Unreliable(router, id),
            reader: StreamReadInner::Unreliable(read_half),
        }
    }
}

impl UTPStream for QuicUTPStream {
    type StreamRead = StreamReadInner;
    type StreamWrite = StreamWriteInner;

    fn id(&self) -> StreamId {
        self.id
    }

    fn integrity_type(&self) -> IntegrityType {
        self.integrity_type.clone()
    }

    fn split(self) -> (Self::StreamWrite, Self::StreamRead) {
        (self.writer, self.reader)
    }
}

impl AsyncRead for StreamReadInner {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Box::pin(async move {
            match *self {
                StreamReadInner::Reliable(ref mut reliable) => {
                    reliable.read_buf(buf).await?;

                    Ok(())
                }
                StreamReadInner::Unreliable(ref mut unreliable) => {
                    unreliable.read_buf(buf).await?;

                    Ok(())
                }
            }
        })
        .as_mut()
        .poll(cx)
    }
}

impl AsyncWrite for StreamWriteInner {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Box::pin(async move {
            match *self {
                StreamWriteInner::Reliable(ref mut reliable) => {
                    let written = reliable.write(buf).await?;
                    Ok(written)
                }
                StreamWriteInner::Unreliable(ref mut unreliable, stream_id) => {
                    unreliable
                        .write(stream_id, Bytes::copy_from_slice(buf))
                        .map_err(|err| std::io::Error::new(ErrorKind::Other, err.to_string()))?;

                    Ok(buf.len())
                }
            }
        })
        .as_mut()
        .poll(cx)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Box::pin(async move {
            match *self {
                Self::Reliable(ref mut reliable) => {
                    reliable.flush().await?;

                    Ok(())
                }
                _ => Ok(()),
            }
        })
        .as_mut()
        .poll(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Box::pin(async move {
            match *self {
                Self::Reliable(ref mut reliable) => {
                    reliable.shutdown().await?;

                    Ok(())
                }
                _ => Ok(()),
            }
        })
        .as_mut()
        .poll(cx)
    }
}
