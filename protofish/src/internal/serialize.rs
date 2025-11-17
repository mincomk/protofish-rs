use bytes::Bytes;
use prost::Message;

use crate::{prost_generated::payload::v1, schema};

pub fn serialize_message(message: schema::Message) -> Bytes {
    let message_prost: v1::Message = message.into();

    let v = message_prost.encode_to_vec();

    Bytes::copy_from_slice(&v)
}

pub fn deserialize_message(buf: &[u8]) -> Option<schema::Message> {
    v1::Message::decode(buf).ok().map(Into::into)
}

#[cfg(test)]
mod tests {
    use crate::{
        constant::VERSION,
        internal::serialize::{deserialize_message, serialize_message},
        schema::{ClientHello, Message, Payload},
    };

    #[test]
    fn test_serialize() {
        let d = Message {
            context_id: 42,
            payload: Payload::ClientHello(ClientHello {
                version: VERSION,
                resume_connection_token: None,
            }),
        };

        let bytes = serialize_message(d.clone());

        let value = deserialize_message(bytes.as_ref()).unwrap();

        assert_eq!(value.context_id, d.context_id);
    }
}
