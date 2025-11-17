use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use tokio::{
    io::{
        AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, DuplexStream, ReadBuf, ReadHalf,
        WriteHalf,
    },
    sync::Mutex,
};

#[derive(Clone)]
pub struct DuplexMockUTP {
    read_half: Arc<Mutex<ReadHalf<DuplexStream>>>,
    write_half: Arc<Mutex<WriteHalf<DuplexStream>>>,
}

impl DuplexMockUTP {
    pub fn new(duplex: DuplexStream) -> Self {
        let (read_half, write_half) = tokio::io::split(duplex);
        Self {
            read_half: Mutex::new(read_half).into(),
            write_half: Mutex::new(write_half).into(),
        }
    }
}

impl AsyncRead for DuplexMockUTP {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Box::pin(async move {
            self.read_half.lock().await.read_buf(buf).await?;
            Ok(())
        })
        .as_mut()
        .poll(cx)
    }
}

impl AsyncWrite for DuplexMockUTP {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Box::pin(async move { self.write_half.lock().await.write(buf).await })
            .as_mut()
            .poll(cx)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}
