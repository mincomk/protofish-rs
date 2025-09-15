use crate::schema::{common::schema::IntegrityType, payload::schema::StreamId};

#[derive(Clone, Debug)]
pub struct Stream {
    pub stream_id: StreamId,
    pub integrity: IntegrityType,
}
