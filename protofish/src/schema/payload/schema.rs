use crate::schema::common::schema::{IntegrityType, ServerHelloInfo, Version};

#[derive(Debug, Clone)]
pub struct Message {
    pub context_id: u64,
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
    pub info: Option<ServerHelloInfo>,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub error_type: crate::schema::common::schema::ErrorType,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct StreamOpen {
    pub stream_id: u64,
    pub meta: crate::schema::common::schema::StreamCreateMeta,
}

#[derive(Debug, Clone)]
pub struct StreamClose {
    pub stream_id: u64,
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