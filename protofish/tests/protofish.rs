use protofish::{
    IntegrityType, accept, connect,
    utp::{self, UTP},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_protofish() {
    let (usa, usb) = utp::mock_utp_pairs();

    let handle = tokio::spawn(async move {
        client_run(usb).await;
    });

    server_run(usa).await;

    handle.await.unwrap();
}

async fn client_run<U: UTP>(utp: U) {
    let conn = connect(utp.into()).await.unwrap();

    let arb = conn.new_arb();
    let mut stream = arb.new_stream(IntegrityType::Reliable).await.unwrap();

    stream.write_all(b"muffinmuffin").await.unwrap();

    tokio::task::yield_now().await;

    let mut got = vec![0u8; 8];
    stream.read_exact(&mut got).await.unwrap();
    assert_eq!(got, b"muffinis");
}

async fn server_run<U: UTP>(utp: U) {
    let conn = accept(utp.into()).await.unwrap();

    let arb = conn.next_arb().await.unwrap();
    let mut stream = arb.wait_stream().await.unwrap();

    let mut got = vec![0u8; 12];
    stream.read_exact(&mut got).await.unwrap();
    assert_eq!(got, b"muffinmuffin");

    stream.write_all(b"muffinis").await.unwrap();
}
