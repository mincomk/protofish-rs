use bytes::{Bytes, BytesMut};
use prost::Message;

use crate::{
    prost_generated::payload::v1,
    schema::payload::schema::{self},
};

pub fn serialize_message(message: schema::Message) -> Bytes {
    let message_prost: v1::Message = message.into();

    let bytes_len = message_prost.encoded_len();
    let mut bytes = BytesMut::with_capacity(bytes_len);
    message_prost.encode(&mut bytes);

    bytes.into()
}

pub fn deserialize_message(buf: &[u8]) -> Option<schema::Message> {
    v1::Message::decode(buf).ok().map(Into::into)
}
