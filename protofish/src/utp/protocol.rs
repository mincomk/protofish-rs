use crate::{
    schema::{common::schema::IntegrityType, payload::schema::StreamId},
    utp::error::UTPError,
};

pub trait UTP: Send + Sync + 'static {
    async fn open_stream(&self, integrity: IntegrityType) -> Result<StreamId, UTPError>;
    async fn close_stream(&self, stream_id: StreamId) -> Result<(), UTPError>;

    fn send(
        &self,
        stream_id: StreamId,
        data: &[u8],
    ) -> impl Future<Output = Result<(), UTPError>> + Send;
    fn receive(
        &self,
        stream_id: StreamId,
        data: &mut [u8],
    ) -> impl Future<Output = Result<(), UTPError>> + Send;

    async fn wait_stream_open(&self, stream_id: StreamId) -> Result<(), UTPError>;
}
