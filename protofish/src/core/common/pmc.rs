use std::sync::Arc;

use parking_lot::Mutex;

use crate::{
    core::common::{
        context::{Context, ContextReader, ContextWriter},
        counter::ContextCounter,
    },
    internal::pmc_frame::PMCFrame,
    schema::payload::schema::Payload,
    utp::protocol::UTPStream,
};

pub struct PMC<S>
where
    S: UTPStream,
{
    counter: Mutex<ContextCounter>,
    frame: PMCFrame<S>,
    utp_stream: Arc<S>,
}

impl<S> PMC<S>
where
    S: UTPStream,
{
    pub(crate) fn new(is_server: bool, utp_stream: S) -> Self {
        let utp_stream = Arc::new(utp_stream);

        Self {
            counter: ContextCounter::new(is_server).into(),
            frame: PMCFrame::new(utp_stream.clone()),
            utp_stream,
        }
    }

    pub fn create_context(&self) -> Context<S> {
        let context_id = self.counter.lock().next_context_id();
        self.make_context(context_id)
    }

    fn make_context(&self, context_id: u64) -> Context<S> {
        let writer = ContextWriter {
            context_id,
            utp_stream: self.utp_stream.clone(),
        };

        let receiver = self.frame.subscribe_context(context_id);

        let reader = ContextReader {
            receiver: receiver.into(),
        };

        (writer, reader)
    }

    pub async fn next_context(&self) -> Option<(Payload, Context<S>)> {
        let msg = self.frame.next_context_message().await?;

        Some((msg.payload, self.make_context(msg.context_id)))
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        core::common::pmc::PMC, schema::payload::schema::Payload, utp::tests::stream::mock_pairs,
    };

    #[tokio::test]
    async fn test_pmc_mock_pair() {
        let (a, b) = mock_pairs();

        let pmc_a = PMC::new(true, a);
        let pmc_b = PMC::new(false, b);

        let (b_tx, b_rx) = pmc_b.create_context();
        b_tx.write(Payload::Ok).await.unwrap();

        let (p, (a_tx, a_rx)) = pmc_a.next_context().await.unwrap();
        assert!(matches!(p, Payload::Ok));

        a_tx.write(Payload::Keepalive).await.unwrap();
        let ba = b_rx.read().await.unwrap();
        assert!(matches!(ba, Payload::Keepalive));
    }
}
