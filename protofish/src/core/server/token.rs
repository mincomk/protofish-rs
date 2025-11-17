use bytes::Bytes;
use rand::{RngCore, rng};

const CONNECTION_TOKEN_LEN: usize = 32;

pub fn generate_connection_token() -> Bytes {
    let mut buf = [0u8; CONNECTION_TOKEN_LEN];

    rng().fill_bytes(&mut buf);
    Bytes::copy_from_slice(&buf)
}
