use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core::common::{
        context::{Context, ContextReader, ContextWriter},
        counter::ContextCounter,
    },
    internal::pmc_frame::PMCFrame,
    schema::Payload,
    utp::UTPStream,
};

pub struct PMC<U: UTPStream> {
    counter: Mutex<ContextCounter>,
    frame: Arc<PMCFrame<U>>,
}

impl<S> PMC<S>
where
    S: UTPStream,
{
    pub(crate) fn new(is_server: bool, utp_stream: S) -> Self {
        Self {
            counter: ContextCounter::new(is_server).into(),
            frame: PMCFrame::new(utp_stream).into(),
        }
    }

    pub fn create_context(&self) -> Context<S> {
        let context_id = self.counter.lock().next_context_id();
        self.make_context(context_id, None)
    }

    fn make_context(&self, context_id: u64, initial_payload: Option<Payload>) -> Context<S> {
        let writer = ContextWriter {
            context_id,
            pmc_frame: self.frame.clone(),
        };

        let receiver = self.frame.subscribe_context(context_id, initial_payload);

        let reader = ContextReader {
            receiver: receiver.into(),
        };

        (writer, reader)
    }

    pub async fn next_context(&self) -> Option<Context<S>> {
        let msg = self.frame.next_context_message().await?;

        let ctx = self.make_context(msg.context_id, Some(msg.payload));

        Some(ctx)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        core::common::pmc::PMC, schema::Payload, utp::tests::stream::mock_utp_stream_pairs,
    };

    #[tokio::test]
    async fn test_pmc_mock_pair() {
        let (a, b) = mock_utp_stream_pairs(0);

        let pmc_a = PMC::new(true, a);
        let pmc_b = PMC::new(false, b);

        let (b_tx, b_rx) = pmc_b.create_context();
        b_tx.write(Payload::Ok).await.unwrap();

        let (a_tx, rx) = pmc_a.next_context().await.unwrap();
        let p = rx.read().await.unwrap();

        assert!(matches!(p, Payload::Ok));

        a_tx.write(Payload::Keepalive).await.unwrap();
        let ba = b_rx.read().await.unwrap();
        assert!(matches!(ba, Payload::Keepalive));
    }
}
