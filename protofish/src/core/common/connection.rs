use std::{marker::PhantomData, sync::Arc};

use crate::{
    core::common::pmc::PMC,
    utp::protocol::{UTP, UTPStream},
};

pub struct Connection<S, U>
where
    S: UTPStream,
    U: UTP<S>,
{
    utp: Arc<U>,
    pub pmc: PMC<S>,
}

impl<S, U> Connection<S, U>
where
    S: UTPStream,
    U: UTP<S>,
{
    pub fn new(utp: Arc<U>, pmc: PMC<S>) -> Self {
        Self { utp, pmc }
    }
}
