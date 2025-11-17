use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::{
    Mutex, Notify,
    mpsc::{self, Receiver, Sender},
};

use crate::{
    schema::{IntegrityType, StreamId},
    utp::{
        error::UTPError,
        protocol::{UTP, UTPEvent},
        tests::stream::{MockUTPStream, mock_utp_stream_pairs},
    },
};

#[derive(Default)]
struct PeerStreamStore {
    pub notify: Notify,
    pub streams: DashMap<StreamId, MockUTPStream>,
}

#[derive(Clone)]
pub struct MockUTP {
    peer: Option<Arc<MockUTP>>,
    id_counter: Arc<AtomicU64>,

    event_receiver: Arc<Mutex<Receiver<UTPEvent>>>,
    event_sender: Sender<UTPEvent>,

    pub(self) peer_streams: Arc<PeerStreamStore>,
}

impl MockUTP {
    pub fn new(id_counter: Arc<AtomicU64>) -> Self {
        let (tx, rx) = mpsc::channel(1024);

        Self {
            peer: None,
            id_counter,
            event_receiver: Mutex::new(rx).into(),
            event_sender: tx,
            peer_streams: Default::default(),
        }
    }

    pub fn set_peer(&mut self, peer: Arc<MockUTP>) {
        self.peer.replace(peer);
    }

    pub(self) async fn add_event(&self, event: UTPEvent) {
        if let Err(e) = self.event_sender.send(event).await {
            tracing::warn!("test send error: {:?}", e);
        }
    }
}

#[async_trait]
impl UTP for MockUTP {
    type Stream = MockUTPStream;

    async fn connect(&self) -> Result<(), UTPError> {
        Ok(())
    }

    async fn next_event(&self) -> UTPEvent {
        if let Some(event) = self.event_receiver.lock().await.recv().await {
            event
        } else {
            UTPEvent::UnexpectedClose
        }
    }

    async fn new_stream(&self, _: IntegrityType) -> Result<MockUTPStream, UTPError> {
        let id = self.id_counter.fetch_add(1, Ordering::Relaxed);

        let (a, b) = mock_utp_stream_pairs(id);

        if let Some(ref peer) = self.peer {
            peer.peer_streams.streams.insert(id, b);
            peer.peer_streams.notify.notify_waiters();
            peer.add_event(UTPEvent::NewStream(id)).await;
        }

        Ok(a)
    }

    async fn wait_stream(
        &self,
        stream_id: StreamId,
        _i: IntegrityType,
    ) -> Result<MockUTPStream, UTPError> {
        loop {
            if let Some((_, stream)) = self.peer_streams.streams.remove(&stream_id) {
                break Ok(stream);
            } else {
                self.peer_streams.notify.notified().await;
            }
        }
    }
}

pub fn mock_utp_pairs() -> (MockUTP, MockUTP) {
    let counter = Arc::new(AtomicU64::new(0));

    let mut a = MockUTP::new(counter.clone());
    let mut b = MockUTP::new(counter.clone());

    a.set_peer(Arc::new(b.clone()));
    b.set_peer(Arc::new(a.clone()));

    (a, b)
}

#[cfg(test)]
mod tests {

    use bytes::{Bytes, BytesMut};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::{
        schema::IntegrityType,
        utp::{
            UTPStream,
            protocol::{UTP, UTPEvent},
            tests::{
                stream::MockUTPStream,
                utp::{MockUTP, mock_utp_pairs},
            },
        },
    };

    async fn open_stream_ab((a, b): (MockUTP, MockUTP)) -> (MockUTPStream, MockUTPStream) {
        let s_a = a.new_stream(IntegrityType::Reliable).await.unwrap();

        let evt = b.next_event().await;
        let id = if let UTPEvent::NewStream(id) = evt {
            id
        } else {
            panic!("unknown event: {:?}", evt);
        };

        let s_b = b.wait_stream(id, IntegrityType::Reliable).await.unwrap();

        (s_a, s_b)
    }

    async fn check_stream_uni((a, b): (MockUTPStream, MockUTPStream)) {
        let bytes = BytesMut::zeroed(9).iter().map(|e| e + 2).collect::<Bytes>();

        a.writer().write_all(&bytes).await.unwrap();

        let mut recv_bytes = vec![0; 9];
        b.reader().read_exact(&mut recv_bytes).await.unwrap();

        assert_eq!(recv_bytes[0], 2);
    }

    async fn check_stream_bi((a, b): (MockUTPStream, MockUTPStream)) {
        check_stream_uni((a.clone(), b.clone())).await;
        check_stream_uni((b, a)).await;
    }

    #[tokio::test]
    async fn test_mock_utp_pairs() {
        let pairs = mock_utp_pairs();

        let ab = open_stream_ab(pairs.clone()).await;
        check_stream_bi(ab).await;

        let ba = open_stream_ab((pairs.1, pairs.0)).await;
        check_stream_bi(ba).await;
    }
}
