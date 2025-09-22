use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use bytes::Bytes;
use tokio::io::{AsyncRead, ReadBuf};

use crate::{
    schema::{common::schema::IntegrityType, payload::schema::StreamId},
    utp::{error::UTPError, protocol::UTPStream},
};

type IoResult<T> = std::io::Result<T>;

pub struct RecvStream<S>
where
    S: UTPStream,
{
    pub stream_id: StreamId,
    pub integrity: IntegrityType,

    upstream: Arc<S>,

    pending_fut: Option<Pin<Box<dyn Future<Output = Result<Bytes, UTPError>> + Send>>>,
}

impl<S> RecvStream<S>
where
    S: UTPStream,
{
    pub(crate) fn new(stream_id: StreamId, integrity: IntegrityType, upstream: S) -> Self {
        Self {
            stream_id,
            integrity,
            upstream: Arc::new(upstream),
            pending_fut: Default::default(),
        }
    }
}

impl<S> AsyncRead for RecvStream<S>
where
    S: UTPStream,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        let capacity = buf.capacity();

        let mut pending_fut = if let Some(fut) = self.pending_fut.take() {
            fut
        } else {
            let upstream = self.upstream.clone();

            let fut = { async move { upstream.receive(capacity).await } };
            

            Box::pin(fut)
        };

        match pending_fut.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(value) => match value {
                Ok(data) => {
                    buf.put_slice(data.as_ref());
                    Poll::Ready(Ok(()))
                }
                Err(e) => Poll::Ready(Err(std::io::Error::other(e))),
            },
        }
    }
}
