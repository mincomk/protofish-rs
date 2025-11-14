use std::sync::Arc;

use bytes::Bytes;
use thiserror::Error;

use crate::{
    IntegrityType, StreamCreateMeta, StreamOpen,
    core::common::{
        context::{Context, ContextReader, ContextWriter},
        error::ConnectionError,
        stream::ProtofishStream,
    },
    schema::{ArbitaryData, Payload},
    utp::{UTP, UTPStream, error::UTPError},
};

/// An arbitrary data context consisting of a writer and reader pair.
///
/// This type provides a simplified interface for sending and receiving
/// arbitrary binary data through a Protofish context.
pub struct ArbContext<U: UTP> {
    writer: ContextWriter<U::Stream>,
    reader: ContextReader,
    utp: Arc<U>,
}

/// Errors that can occur during arbitrary data operations.
#[derive(Debug, Error)]
pub enum ArbError {
    /// Error from the underlying connection
    #[error("connection error: {0}")]
    Connection(#[from] ConnectionError),

    /// Received a payload that was not `ArbitaryData`
    #[error("unexpected data: {0}")]
    UnexpectedData(String),

    /// UTP Error
    #[error("UTP error: {0}")]
    UTP(#[from] UTPError),
}

impl<U: UTP> ArbContext<U> {
    /// Writes arbitrary binary data to this context.
    ///
    /// The bytes will be wrapped in an `ArbitaryData` payload and sent
    /// to the peer.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying write operation fails.
    pub async fn write(&self, content: Bytes) -> Result<(), ArbError> {
        let payload = Payload::ArbitaryData(ArbitaryData {
            content: content.into(),
        });

        self.writer.write(payload).await?;

        Ok(())
    }

    /// Reads arbitrary binary data from this context.
    ///
    /// This method expects the next payload to be `ArbitaryData` and
    /// extracts the bytes from it.
    ///
    /// # Returns
    ///
    /// Returns the binary content, or an error if the payload is not
    /// `ArbitaryData` or the read fails.
    ///
    /// # Errors
    ///
    /// Returns `ArbError::UnexpectedData` if a non-`ArbitaryData` payload
    /// is received, or `ArbError::Connection` if the read fails.
    pub async fn read(&self) -> Result<Bytes, ArbError> {
        let data_got = self.reader.read().await?;

        if let Payload::ArbitaryData(data) = data_got {
            Ok(Bytes::from(data.content))
        } else {
            Err(ArbError::UnexpectedData("expected ArbitaryData".into()))
        }
    }

    pub async fn wait_stream(&self) -> Result<ProtofishStream<U::Stream>, ArbError> {
        let data_got = self.reader.read().await?;

        if let Payload::StreamOpen(meta) = data_got {
            let utp_stream = self
                .utp
                .wait_stream(meta.stream_id, meta.meta.integrity_type)
                .await?;
            Ok(ProtofishStream::new(utp_stream))
        } else {
            Err(ArbError::UnexpectedData("expected ArbitaryData".into()))
        }
    }
    pub async fn new_stream(
        &self,
        integrity: IntegrityType,
    ) -> Result<ProtofishStream<U::Stream>, ArbError> {
        let stream = self.utp.new_stream(integrity.clone()).await?;
        self.writer
            .write(Payload::StreamOpen(StreamOpen {
                stream_id: stream.id(),
                meta: StreamCreateMeta {
                    integrity_type: integrity,
                },
            }))
            .await?;
        Ok(ProtofishStream::new(stream))
    }
}

/// Converts a generic context into an arbitrary data context.
///
/// This helper function wraps the context writer and reader with the
/// arbitrary data interface.
pub fn make_arbitrary<U: UTP>(utp: Arc<U>, (writer, reader): Context<U::Stream>) -> ArbContext<U> {
    ArbContext {
        utp,
        writer,
        reader,
    }
}
