use bytes::Bytes;

use crate::schema::{IntegrityType, Version};

pub type ContextId = u64;
pub type StreamId = u64;

#[derive(Debug, Clone)]
pub struct Message {
    pub context_id: ContextId,
    pub payload: Payload,
}

#[derive(Debug, Clone)]
pub enum Payload {
    ClientHello(ClientHello),
    ServerHello(ServerHello),
    Ok,
    Error(Error),
    StreamOpen(StreamOpen),
    StreamClose(StreamClose),
    ArbitaryData(ArbitaryData),
    Keepalive,
    Close,
    BenchmarkStart(BenchmarkStart),
    BenchmarkEnd,
}

#[derive(Debug, Clone)]
pub struct ClientHello {
    pub version: Version,
    pub resume_connection_token: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ServerHello {
    pub version: Version,
    pub ok: bool,
    pub connection_token: Option<Bytes>,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub error_type: crate::schema::ErrorType,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct StreamOpen {
    pub stream_id: StreamId,
    pub meta: crate::schema::StreamCreateMeta,
}

#[derive(Debug, Clone)]
pub struct StreamClose {
    pub stream_id: StreamId,
}

#[derive(Debug, Clone)]
pub struct ArbitaryData {
    pub content: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkStart {
    pub integrity_type: IntegrityType,
    pub byte_count: u64,
}
