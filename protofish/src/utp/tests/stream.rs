use tokio::io::DuplexStream;

use crate::{
    IntegrityType,
    schema::StreamId,
    utp::{protocol::UTPStream, tests::duplex::DuplexMockUTP},
};

#[derive(Clone)]
pub struct MockUTPStream {
    pub id: StreamId,
    stream: DuplexMockUTP,
}

impl MockUTPStream {
    pub fn new(id: StreamId, stream: DuplexStream) -> Self {
        Self {
            id,
            stream: DuplexMockUTP::new(stream),
        }
    }
}

impl UTPStream for MockUTPStream {
    type StreamRead = DuplexMockUTP;
    type StreamWrite = DuplexMockUTP;

    fn id(&self) -> StreamId {
        self.id
    }

    fn integrity_type(&self) -> IntegrityType {
        IntegrityType::Reliable
    }

    fn reader(&self) -> Self::StreamRead {
        self.stream.clone()
    }

    fn writer(&self) -> Self::StreamWrite {
        self.stream.clone()
    }
}

pub fn mock_utp_stream_pairs(id: StreamId) -> (MockUTPStream, MockUTPStream) {
    let (a, b) = tokio::io::duplex(1024);

    let x = MockUTPStream::new(id, a);
    let y = MockUTPStream::new(id, b);

    (x, y)
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::utp::{UTPStream, tests::stream::mock_utp_stream_pairs};

    #[tokio::test]
    async fn test_mock_pairs_atob() {
        let (a, b) = mock_utp_stream_pairs(0);

        a.writer()
            .write_all(
                &BytesMut::zeroed(14)
                    .iter()
                    .map(|e| e + 1)
                    .collect::<BytesMut>()
                    .freeze(),
            )
            .await
            .unwrap();

        let mut b_data = vec![0; 14];
        b.reader().read_exact(&mut b_data).await.unwrap();

        assert_eq!(b_data[13], 1);
    }

    #[tokio::test]
    async fn test_mock_pairs_btoa() {
        let (a, b) = mock_utp_stream_pairs(0);

        b.writer()
            .write_all(
                &BytesMut::zeroed(14)
                    .iter()
                    .map(|e| e + 1)
                    .collect::<BytesMut>()
                    .freeze(),
            )
            .await
            .unwrap();

        let mut a_data = vec![0; 14];
        a.reader().read_exact(&mut a_data).await.unwrap();

        assert_eq!(a_data[13], 1);
    }
}
