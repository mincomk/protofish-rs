use protofish::{
    IntegrityType, accept, connect,
    utp::{self, UTP},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_protofish() {
    let (usa, usb) = utp::mock_utp_pairs();

    tokio::spawn(server_run(usb));

    client_run(usa).await;
}

async fn client_run<U: UTP>(utp: U) {
    let conn = connect(utp.into()).await.unwrap();

    let arb = conn.new_arb();
    let mut stream = arb.new_stream(IntegrityType::Reliable).await.unwrap();

    stream.write_all(b"muffinmuffin").await.unwrap();

    let mut got = vec![0u8; 8];
    stream.read_exact(&mut got).await.unwrap();
    assert_eq!(got, b"muffin");
}

async fn server_run<U: UTP>(utp: U) {
    let conn = accept(utp.into()).await.unwrap();

    let arb = conn.next_arb().await.unwrap();
    let mut stream = arb.wait_stream().await.unwrap();

    let mut got = vec![0u8; 12];
    stream.read_exact(&mut got).await.unwrap();
    assert_eq!(got, b"muffinmuffin");

    let v = vec![1u8; 8];
    stream.write(&v).await.unwrap();
}
