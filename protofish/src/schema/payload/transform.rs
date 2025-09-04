use crate::{
    prost_generated::common::v1 as common_v1,
    prost_generated::payload::v1 as payload_v1,
    schema::common::schema as common_schema,
    schema::payload::schema as payload_schema,
};

impl From<payload_v1::Message> for payload_schema::Message {
    fn from(value: payload_v1::Message) -> Self {
        payload_schema::Message {
            context_id: value.context_id,
            payload: value.payload.unwrap().into(),
        }
    }
}

impl From<payload_schema::Message> for payload_v1::Message {
    fn from(value: payload_schema::Message) -> Self {
        payload_v1::Message {
            context_id: value.context_id,
            payload: Some(value.payload.into()),
        }
    }
}

impl From<payload_v1::Payload> for payload_schema::Payload {
    fn from(value: payload_v1::Payload) -> Self {
        match value.payload.unwrap() {
            payload_v1::payload::Payload::ClientHello(v) => {
                payload_schema::Payload::ClientHello(v.into())
            }
            payload_v1::payload::Payload::Ok(_) => payload_schema::Payload::Ok,
            payload_v1::payload::Payload::Error(v) => payload_schema::Payload::Error(v.into()),
            payload_v1::payload::Payload::StreamOpen(v) => {
                payload_schema::Payload::StreamOpen(v.into())
            }
            payload_v1::payload::Payload::StreamClose(v) => {
                payload_schema::Payload::StreamClose(v.into())
            }
            payload_v1::payload::Payload::ArbitaryData(v) => {
                payload_schema::Payload::ArbitaryData(v.into())
            }
            payload_v1::payload::Payload::Keepalive(_) => payload_schema::Payload::Keepalive,
            payload_v1::payload::Payload::ServerHello(v) => {
                payload_schema::Payload::ServerHello(v.into())
            }
            payload_v1::payload::Payload::Close(_) => payload_schema::Payload::Close,
            payload_v1::payload::Payload::BenchmarkStart(v) => {
                payload_schema::Payload::BenchmarkStart(v.into())
            }
            payload_v1::payload::Payload::BenchmarkEnd(_) => payload_schema::Payload::BenchmarkEnd,
        }
    }
}

impl From<payload_schema::Payload> for payload_v1::Payload {
    fn from(value: payload_schema::Payload) -> Self {
        let payload = match value {
            payload_schema::Payload::ClientHello(v) => {
                payload_v1::payload::Payload::ClientHello(v.into())
            }
            payload_schema::Payload::Ok => payload_v1::payload::Payload::Ok(payload_v1::Ok {}),
            payload_schema::Payload::Error(v) => payload_v1::payload::Payload::Error(v.into()),
            payload_schema::Payload::StreamOpen(v) => {
                payload_v1::payload::Payload::StreamOpen(v.into())
            }
            payload_schema::Payload::StreamClose(v) => {
                payload_v1::payload::Payload::StreamClose(v.into())
            }
            payload_schema::Payload::ArbitaryData(v) => {
                payload_v1::payload::Payload::ArbitaryData(v.into())
            }
            payload_schema::Payload::Keepalive => {
                payload_v1::payload::Payload::Keepalive(payload_v1::Keepalive {})
            }
            payload_schema::Payload::ServerHello(v) => {
                payload_v1::payload::Payload::ServerHello(v.into())
            }
            payload_schema::Payload::Close => {
                payload_v1::payload::Payload::Close(payload_v1::Close {})
            }
            payload_schema::Payload::BenchmarkStart(v) => {
                payload_v1::payload::Payload::BenchmarkStart(v.into())
            }
            payload_schema::Payload::BenchmarkEnd => {
                payload_v1::payload::Payload::BenchmarkEnd(payload_v1::BenchmarkEnd {})
            }
        };

        payload_v1::Payload {
            payload: Some(payload),
        }
    }
}

impl From<payload_v1::ClientHello> for payload_schema::ClientHello {
    fn from(value: payload_v1::ClientHello) -> Self {
        payload_schema::ClientHello {
            version: value.version.unwrap().into(),
            resume_connection_token: value.resume_connection_token,
        }
    }
}

impl From<payload_schema::ClientHello> for payload_v1::ClientHello {
    fn from(value: payload_schema::ClientHello) -> Self {
        payload_v1::ClientHello {
            version: Some(value.version.into()),
            resume_connection_token: value.resume_connection_token,
        }
    }
}

impl From<payload_v1::ServerHello> for payload_schema::ServerHello {
    fn from(value: payload_v1::ServerHello) -> Self {
        payload_schema::ServerHello {
            version: value.version.unwrap().into(),
            ok: value.ok,
            info: value.info.map(|v| v.into()),
        }
    }
}

impl From<payload_schema::ServerHello> for payload_v1::ServerHello {
    fn from(value: payload_schema::ServerHello) -> Self {
        payload_v1::ServerHello {
            version: Some(value.version.into()),
            ok: value.ok,
            info: value.info.map(|v| v.into()),
        }
    }
}

impl From<payload_v1::Error> for payload_schema::Error {
    fn from(value: payload_v1::Error) -> Self {
        payload_schema::Error {
            error_type: common_v1::ErrorType::try_from(value.error_type)
                .unwrap()
                .into(),
            message: value.message,
        }
    }
}

impl From<payload_schema::Error> for payload_v1::Error {
    fn from(value: payload_schema::Error) -> Self {
        payload_v1::Error {
            error_type: value.error_type.into(),
            message: value.message,
        }
    }
}

impl From<payload_v1::StreamOpen> for payload_schema::StreamOpen {
    fn from(value: payload_v1::StreamOpen) -> Self {
        payload_schema::StreamOpen {
            stream_id: value.stream_id,
            meta: value.meta.unwrap().into(),
        }
    }
}

impl From<payload_schema::StreamOpen> for payload_v1::StreamOpen {
    fn from(value: payload_schema::StreamOpen) -> Self {
        payload_v1::StreamOpen {
            stream_id: value.stream_id,
            meta: Some(value.meta.into()),
        }
    }
}

impl From<payload_v1::StreamClose> for payload_schema::StreamClose {
    fn from(value: payload_v1::StreamClose) -> Self {
        payload_schema::StreamClose {
            stream_id: value.stream_id,
        }
    }
}

impl From<payload_schema::StreamClose> for payload_v1::StreamClose {
    fn from(value: payload_schema::StreamClose) -> Self {
        payload_v1::StreamClose {
            stream_id: value.stream_id,
        }
    }
}

impl From<payload_v1::ArbitaryData> for payload_schema::ArbitaryData {
    fn from(value: payload_v1::ArbitaryData) -> Self {
        payload_schema::ArbitaryData {
            content: value.content,
        }
    }
}

impl From<payload_schema::ArbitaryData> for payload_v1::ArbitaryData {
    fn from(value: payload_schema::ArbitaryData) -> Self {
        payload_v1::ArbitaryData {
            content: value.content,
        }
    }
}

impl From<payload_v1::BenchmarkStart> for payload_schema::BenchmarkStart {
    fn from(value: payload_v1::BenchmarkStart) -> Self {
        payload_schema::BenchmarkStart {
            integrity_type: common_v1::IntegrityType::try_from(value.integrity_type)
                .unwrap()
                .into(),
            byte_count: value.byte_count,
        }
    }
}

impl From<payload_schema::BenchmarkStart> for payload_v1::BenchmarkStart {
    fn from(value: payload_schema::BenchmarkStart) -> Self {
        payload_v1::BenchmarkStart {
            integrity_type: value.integrity_type.into(),
            byte_count: value.byte_count,
        }
    }
}

impl From<common_v1::ServerHelloInfo> for common_schema::ServerHelloInfo {
    fn from(value: common_v1::ServerHelloInfo) -> Self {
        common_schema::ServerHelloInfo {
            connection_token: value.connection_token,
            is_resume: value.is_resume,
        }
    }
}

impl From<common_schema::ServerHelloInfo> for common_v1::ServerHelloInfo {
    fn from(value: common_schema::ServerHelloInfo) -> Self {
        common_v1::ServerHelloInfo {
            connection_token: value.connection_token,
            is_resume: value.is_resume,
        }
    }
}

impl From<common_schema::StreamCreateMeta> for common_v1::StreamCreateMeta {
    fn from(value: common_schema::StreamCreateMeta) -> Self {
        common_v1::StreamCreateMeta {
            stream_integrity: value.integrity_type.into(),
        }
    }
}

impl From<common_schema::ErrorType> for i32 {
    fn from(value: common_schema::ErrorType) -> Self {
        (match value {
            common_schema::ErrorType::Unspecified => common_v1::ErrorType::Unspecified,
            common_schema::ErrorType::Timeout => common_v1::ErrorType::Timeout,
        })
        .into()
    }
}

impl From<common_schema::IntegrityType> for i32 {
    fn from(value: common_schema::IntegrityType) -> Self {
        (match value {
            common_schema::IntegrityType::Reliable => common_v1::IntegrityType::Reliable,
            common_schema::IntegrityType::Unreliable => common_v1::IntegrityType::Unreliable,
        })
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::common::schema::{ErrorType, IntegrityType};

    #[test]
    fn test_message_conversion() {
        let proto_message = payload_v1::Message {
            context_id: 123,
            payload: Some(payload_v1::Payload {
                payload: Some(payload_v1::payload::Payload::Ok(payload_v1::Ok {})),
            }),
        };
        let schema_message: payload_schema::Message = proto_message.clone().into();
        assert_eq!(schema_message.context_id, 123);
        assert!(matches!(
            schema_message.payload,
            payload_schema::Payload::Ok
        ));

        let converted_proto: payload_v1::Message = schema_message.into();
        assert_eq!(converted_proto, proto_message);
    }

    #[test]
    fn test_payload_conversion() {
        // Test ClientHello
        let proto_client_hello = payload_v1::ClientHello {
            version: Some(common_v1::Version {
                major: 1,
                minor: 0,
                patch: 0,
            }),
            resume_connection_token: None,
        };
        let payload = payload_v1::Payload {
            payload: Some(payload_v1::payload::Payload::ClientHello(
                proto_client_hello,
            )),
        };
        let schema_payload: payload_schema::Payload = payload.into();
        assert!(matches!(
            schema_payload,
            payload_schema::Payload::ClientHello(_)
        ));

        // Test OK
        let payload = payload_v1::Payload {
            payload: Some(payload_v1::payload::Payload::Ok(payload_v1::Ok {})),
        };
        let schema_payload: payload_schema::Payload = payload.into();
        assert!(matches!(schema_payload, payload_schema::Payload::Ok));
    }

    #[test]
    fn test_client_hello_conversion() {
        let proto_client_hello = payload_v1::ClientHello {
            version: Some(common_v1::Version {
                major: 1,
                minor: 0,
                patch: 0,
            }),
            resume_connection_token: Some(vec![1, 2, 3]),
        };
        let schema_client_hello: payload_schema::ClientHello = proto_client_hello.clone().into();
        assert_eq!(schema_client_hello.version.major, 1);
        assert_eq!(
            schema_client_hello.resume_connection_token,
            Some(vec![1, 2, 3])
        );

        let converted_proto: payload_v1::ClientHello = schema_client_hello.into();
        assert_eq!(converted_proto, proto_client_hello);
    }

    #[test]
    fn test_server_hello_conversion() {
        let proto_server_hello = payload_v1::ServerHello {
            version: Some(common_v1::Version {
                major: 1,
                minor: 0,
                patch: 0,
            }),
            ok: true,
            info: Some(common_v1::ServerHelloInfo {
                connection_token: vec![4, 5, 6],
                is_resume: true,
            }),
        };
        let schema_server_hello: payload_schema::ServerHello = proto_server_hello.clone().into();
        assert_eq!(schema_server_hello.version.major, 1);
        assert!(schema_server_hello.ok);
        assert!(schema_server_hello.info.is_some());

        let converted_proto: payload_v1::ServerHello = schema_server_hello.into();
        assert_eq!(converted_proto, proto_server_hello);
    }

    #[test]
    fn test_error_conversion() {
        let proto_error = payload_v1::Error {
            error_type: common_v1::ErrorType::Timeout.into(),
            message: "Request timed out".to_string(),
        };
        let schema_error: payload_schema::Error = proto_error.clone().into();
        assert!(matches!(schema_error.error_type, ErrorType::Timeout));
        assert_eq!(schema_error.message, "Request timed out");

        let converted_proto: payload_v1::Error = schema_error.into();
        assert_eq!(converted_proto, proto_error);
    }

    #[test]
    fn test_stream_open_conversion() {
        let proto_stream_open = payload_v1::StreamOpen {
            stream_id: 12345,
            meta: Some(common_v1::StreamCreateMeta {
                stream_integrity: common_v1::IntegrityType::Reliable.into(),
            }),
        };
        let schema_stream_open: payload_schema::StreamOpen = proto_stream_open.clone().into();
        assert_eq!(schema_stream_open.stream_id, 12345);
        assert!(matches!(
            schema_stream_open.meta.integrity_type,
            IntegrityType::Reliable
        ));

        let converted_proto: payload_v1::StreamOpen = schema_stream_open.into();
        assert_eq!(converted_proto, proto_stream_open);
    }

    #[test]
    fn test_stream_close_conversion() {
        let proto_stream_close = payload_v1::StreamClose { stream_id: 54321 };
        let schema_stream_close: payload_schema::StreamClose = proto_stream_close.clone().into();
        assert_eq!(schema_stream_close.stream_id, 54321);

        let converted_proto: payload_v1::StreamClose = schema_stream_close.into();
        assert_eq!(converted_proto, proto_stream_close);
    }

    #[test]
    fn test_arbitary_data_conversion() {
        let proto_arbitary_data = payload_v1::ArbitaryData { content: vec![1, 2, 3, 4] };
        let schema_arbitary_data: payload_schema::ArbitaryData = proto_arbitary_data.clone().into();
        assert_eq!(schema_arbitary_data.content, vec![1, 2, 3, 4]);

        let converted_proto: payload_v1::ArbitaryData = schema_arbitary_data.into();
        assert_eq!(converted_proto, proto_arbitary_data);
    }

    #[test]
    fn test_benchmark_start_conversion() {
        let proto_benchmark_start = payload_v1::BenchmarkStart {
            integrity_type: common_v1::IntegrityType::Unreliable.into(),
            byte_count: 1024,
        };
        let schema_benchmark_start: payload_schema::BenchmarkStart = proto_benchmark_start.clone().into();
        assert!(matches!(
            schema_benchmark_start.integrity_type,
            IntegrityType::Unreliable
        ));
        assert_eq!(schema_benchmark_start.byte_count, 1024);

        let converted_proto: payload_v1::BenchmarkStart = schema_benchmark_start.into();
        assert_eq!(converted_proto, proto_benchmark_start);
    }
}