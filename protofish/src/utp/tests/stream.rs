use std::sync::Arc;

use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use tokio::sync::{Mutex, Notify};

use crate::{
    IntegrityType,
    schema::StreamId,
    utp::{error::UTPError, protocol::UTPStream},
};

#[derive(Clone)]
pub struct MockUTPStream {
    pub id: StreamId,
    peer: Option<Arc<MockUTPStream>>,
    buffer: Arc<Mutex<BytesMut>>,
    notify: Arc<Notify>,
}

impl MockUTPStream {
    pub fn new(id: StreamId, peer: Option<MockUTPStream>) -> Self {
        Self {
            id,
            peer: peer.map(Arc::new),
            buffer: Default::default(),
            notify: Notify::new().into(),
        }
    }

    pub fn set_peer(&mut self, peer: Option<MockUTPStream>) {
        self.peer = peer.map(Arc::new);
    }

    pub(self) async fn write_buffer(&self, data: &Bytes) {
        self.buffer.lock().await.extend_from_slice(data);
        self.notify.notify_waiters();
    }

    pub(self) async fn read_buffer(&self, n: usize) -> Bytes {
        loop {
            let mut buf = self.buffer.lock().await;
            if buf.len() >= n {
                let result = buf.split_to(n).freeze();
                return result;
            }
            drop(buf);
            self.notify.notified().await;
        }
    }
}

#[async_trait]
impl UTPStream for MockUTPStream {
    fn id(&self) -> StreamId {
        self.id
    }

    fn integrity_type(&self) -> IntegrityType {
        IntegrityType::Reliable
    }

    async fn send(&self, data: &Bytes) -> Result<(), UTPError> {
        if let Some(peer) = self.peer.as_ref() {
            peer.write_buffer(data).await;
        }
        Ok(())
    }

    async fn receive(&self, len: usize) -> Result<Bytes, UTPError> {
        let d = self.read_buffer(len).await;

        Ok(d)
    }

    async fn close(&self) -> Result<(), UTPError> {
        self.buffer.lock().await.clear();
        Ok(())
    }
}

// legacy compat
#[cfg(test)]
pub fn mock_pairs() -> (MockUTPStream, MockUTPStream) {
    let mut x = MockUTPStream::new(0, None);
    let y = MockUTPStream::new(1, Some(x.clone()));
    x.set_peer(Some(y.clone()));

    (x, y)
}

pub fn mock_utp_stream_pairs(id: StreamId) -> (MockUTPStream, MockUTPStream) {
    let mut x = MockUTPStream::new(id, None);
    let y = MockUTPStream::new(id, Some(x.clone()));
    x.set_peer(Some(y.clone()));

    (x, y)
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::utp::{protocol::UTPStream, tests::stream::mock_pairs};

    #[tokio::test]
    async fn test_mock_pairs_atob() {
        let (a, b) = mock_pairs();

        a.send(
            &BytesMut::zeroed(14)
                .iter()
                .map(|e| e + 1)
                .collect::<BytesMut>()
                .freeze(),
        )
        .await
        .unwrap();

        let b_data = b.receive(14).await.unwrap();

        assert_eq!(b_data[13], 1);
    }

    #[tokio::test]
    async fn test_mock_pairs_btoa() {
        let (a, b) = mock_pairs();

        b.send(
            &BytesMut::zeroed(14)
                .iter()
                .map(|e| e + 1)
                .collect::<BytesMut>()
                .freeze(),
        )
        .await
        .unwrap();

        let a_data = a.receive(14).await.unwrap();

        assert_eq!(a_data[13], 1);
    }
}
