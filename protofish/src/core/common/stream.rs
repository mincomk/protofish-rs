use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::utp::UTPStream;

pub struct ProtofishStream<U: UTPStream> {
    stream: U,
}

impl<U: UTPStream> ProtofishStream<U> {
    pub(crate) fn new(stream: U) -> Self {
        Self { stream }
    }
}

impl<U: UTPStream> AsyncRead for ProtofishStream<U> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Box::pin(async {
            let mut data = vec![0; buf.capacity()];
            self.stream
                .receive(&mut data)
                .await
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
            buf.put_slice(&data);

            Ok(())
        })
        .as_mut()
        .poll(cx)
    }
}

impl<U: UTPStream> AsyncWrite for ProtofishStream<U> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Box::pin(async {
            self.stream
                .send(buf)
                .await
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))
                .map(|_| buf.len())
        })
        .as_mut()
        .poll(cx)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
