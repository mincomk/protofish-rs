use std::{pin::Pin, sync::Arc};


use crate::{
    schema::{common::schema::IntegrityType, payload::schema::StreamId},
    utp::{error::UTPError, protocol::UTPStream},
};

pub struct SendStream<S>
where
    S: UTPStream,
{
    pub integrity: IntegrityType,
    pub stream_id: StreamId,

    upstream: Arc<S>,

    pending_fut: Option<Pin<Box<dyn Future<Output = Result<(), UTPError>> + Send>>>,
}

impl<S> SendStream<S>
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
