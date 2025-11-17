use std::sync::Arc;

use crate::{
    core::common::{
        arbitrary::{ArbContext, make_arbitrary},
        pmc::PMC,
    },
    utp::UTP,
};

/// Represents an established Protofish connection.
///
/// A `Connection` provides access to the Primary Messaging Channel (PMC) and
/// methods for creating and handling arbitrary data contexts. Contexts enable
/// request-response patterns with proper grouping and ordering guarantees.
pub struct Connection<U>
where
    U: UTP,
{
    utp: Arc<U>,

    /// The Primary Messaging Channel for this connection
    pub pmc: PMC<U::Stream>,
}

impl<U> Connection<U>
where
    U: UTP,
{
    pub fn new(utp: Arc<U>, pmc: PMC<U::Stream>) -> Self {
        Self { utp, pmc }
    }

    /// Creates a new arbitrary data context for sending messages.
    ///
    /// This creates a new context with a unique context ID following the
    /// parity rules (even for client-initiated, odd for server-initiated).
    /// Use this to initiate a new conversation.
    ///
    /// # Returns
    ///
    /// Returns an `ArbContext` containing a writer and reader for arbitrary binary data.
    pub fn new_arb(&self) -> ArbContext<U> {
        let ctx = self.pmc.create_context();
        make_arbitrary(self.utp.clone(), ctx)
    }

    /// Waits for the next incoming arbitrary data context from the peer.
    ///
    /// This method blocks until a message arrives on a new context. Use this
    /// to handle incoming requests or messages from the peer.
    ///
    /// # Returns
    ///
    /// Returns `Some(ArbContext)` when a new context arrives, or `None` if
    /// the connection is closed.
    pub async fn next_arb(&self) -> Option<ArbContext<U>> {
        let ctx = self.pmc.next_context().await?;
        Some(make_arbitrary(self.utp.clone(), ctx))
    }
}
