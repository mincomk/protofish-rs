use async_trait::async_trait;
use bytes::{Bytes, BytesMut};

use crate::{
    schema::{common::schema::IntegrityType, payload::schema::StreamId},
    utp::error::UTPError,
};

#[async_trait]
pub trait UTPStream: Send + Sync + 'static {
    fn id(&self) -> StreamId;
    async fn send(&self, data: &Bytes) -> Result<(), UTPError>;
    async fn receive(&self, data: &mut BytesMut) -> Result<usize, UTPError>;
    async fn close(&self) -> Result<(), UTPError>;
}

#[async_trait]
pub trait UTP<S: UTPStream>: Send + Sync + 'static {
    async fn connect(&self) -> Result<(), UTPError>;
    async fn next_event(&self) -> UTPEvent;

    async fn open_stream(&self, integrity: IntegrityType) -> Result<S, UTPError>;
    async fn wait_stream(&self, id: StreamId) -> Result<S, UTPError>;
}

pub enum UTPEvent {
    UnexpectedClose,
    NewStream(StreamId),
}
