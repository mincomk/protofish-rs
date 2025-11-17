use crate::utp::UTPStream;

pub struct ProtofishStream<U: UTPStream> {
    stream: U,
}

impl<U: UTPStream> ProtofishStream<U> {
    pub(crate) fn new(stream: U) -> Self {
        Self { stream }
    }

    pub fn reader(&self) -> U::StreamRead {
        self.stream.reader()
    }

    pub fn writer(&self) -> U::StreamWrite {
        self.stream.writer()
    }
}
