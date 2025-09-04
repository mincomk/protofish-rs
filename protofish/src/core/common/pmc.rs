use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core::common::{
        context::{ContextReader, ContextWriter},
        counter::ContextCounter,
        stream::Stream,
    },
    utp::protocol::UTP,
};

pub struct PMC<U>
where
    U: UTP,
{
    counter: Mutex<ContextCounter>,
    stream: Stream,
    utp: Arc<U>,
}

impl<U> PMC<U>
where
    U: UTP,
{
    fn new(is_server: bool, utp: U, stream: Stream) -> Self {
        Self {
            counter: ContextCounter::new(is_server).into(),
            stream,
            utp: utp.into(),
        }
    }

    fn create_context(&self) -> (ContextWriter<U>, ContextReader) {
        let context_id = self.counter.lock().next_context_id();
    }
}
