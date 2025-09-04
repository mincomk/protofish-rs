use std::sync::Arc;

use tokio::sync::mpsc;

use crate::{
    constant::CHANNEL_BUFFER,
    core::common::context::{ContextReader, ContextWriter},
    utp::protocol::UTP,
};

pub struct Connection<U>
where
    U: UTP,
{
    utp: Arc<U>,
}

impl<U> Connection<U>
where
    U: UTP,
{
    pub fn new(utp: U) -> Self {
        Self { utp: Arc::new(utp) }
    }
}
