use crate::utp::UTPStream;

pub struct ProtofishStream<U: UTPStream> {
    stream: U,
}

impl<U: UTPStream> ProtofishStream<U> {
    pub(crate) fn new(stream: U) -> Self {
        Self { stream }
    }

    #[inline(always)]
    pub fn split(self) -> (U::StreamWrite, U::StreamRead) {
        self.stream.split()
    }
}
