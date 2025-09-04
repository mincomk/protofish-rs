use crate::{schema::common::schema::IntegrityType, utp::types::StreamId};

pub struct Stream {
    pub stream_id: StreamId,
    pub integrity: IntegrityType,
}
