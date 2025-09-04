use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core::common::{
        context::{ContextReader, ContextWriter},
        counter::ContextCounter,
        stream::Stream,
    },
    internal::pmc_frame::PMCFrame,
    utp::protocol::UTP,
};

pub struct PMC<U>
where
    U: UTP,
{
    counter: Mutex<ContextCounter>,
    stream: Stream,
    frame: PMCFrame<U>,
    utp: Arc<U>,
}

impl<U> PMC<U>
where
    U: UTP,
{
    pub(crate) fn new(is_server: bool, utp: U, stream: Stream) -> Self {
        let utp = Arc::new(utp);

        Self {
            counter: ContextCounter::new(is_server).into(),
            frame: PMCFrame::new(utp.clone(), stream.stream_id),
            stream,
            utp,
        }
    }

    pub fn create_context(&self) -> (ContextWriter<U>, ContextReader) {
        let context_id = self.counter.lock().next_context_id();

        let writer = ContextWriter {
            context_id,
            stream_id: self.stream.stream_id,
            utp: self.utp.clone(),
        };

        let receiver = self.frame.subscribe_context(context_id);

        let reader = ContextReader {
            receiver: receiver.into(),
        };

        (writer, reader)
    }
}
