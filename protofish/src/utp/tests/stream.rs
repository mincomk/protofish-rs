use tokio::io::{DuplexStream, ReadHalf, WriteHalf};

use crate::{IntegrityType, schema::StreamId, utp::protocol::UTPStream};

pub struct MockUTPStream {
    pub id: StreamId,
    stream: DuplexStream,
}

impl MockUTPStream {
    pub fn new(id: StreamId, stream: DuplexStream) -> Self {
        Self { id, stream }
    }
}

impl UTPStream for MockUTPStream {
    type StreamRead = ReadHalf<DuplexStream>;
    type StreamWrite = WriteHalf<DuplexStream>;

    fn id(&self) -> StreamId {
        self.id
    }

    #[inline(always)]
    fn integrity_type(&self) -> IntegrityType {
        IntegrityType::Reliable
    }

    #[inline(always)]
    fn split(self) -> (Self::StreamWrite, Self::StreamRead) {
        let (a, b) = tokio::io::split(self.stream);
        (b, a)
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

        let (mut a_writer, _) = a.split();
        let (_, mut b_reader) = b.split();

        a_writer
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
        b_reader.read_exact(&mut b_data).await.unwrap();

        assert_eq!(b_data[13], 1);
    }

    #[tokio::test]
    async fn test_mock_pairs_btoa() {
        let (a, b) = mock_utp_stream_pairs(0);

        let (_, mut a_reader) = a.split();
        let (mut b_writer, _) = b.split();

        b_writer
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
        a_reader.read_exact(&mut a_data).await.unwrap();

        assert_eq!(a_data[13], 1);
    }
}
