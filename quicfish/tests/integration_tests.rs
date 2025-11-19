use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

use bytes::Bytes;
use protofish::IntegrityType;
use quicfish::{QuicConfig, QuicEndpoint, QuicUTP};

mod common;
use common::create_test_certs;

#[tokio::test]
async fn test_basic_connection() {
    let (server_crypto, client_crypto) = create_test_certs();

    let server_config = QuicConfig::server_default().with_server_crypto(server_crypto);
    let server_endpoint = QuicEndpoint::server("127.0.0.1:0".parse().unwrap(), server_config)
        .expect("Failed to create server endpoint");
    let server_addr = server_endpoint.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        if let Some(conn) = server_endpoint.accept().await {
            let _utp = QuicUTP::new(conn, true);
            true
        } else {
            false
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client_config = QuicConfig::client_default().with_client_crypto(client_crypto);
    let client_endpoint = QuicEndpoint::client("127.0.0.1:0".parse().unwrap(), client_config)
        .expect("Failed to create client endpoint");

    let conn = client_endpoint
        .connect(server_addr, "localhost")
        .await
        .expect("Failed to connect");

    let _client_utp = QuicUTP::new(conn, false);

    let server_result = timeout(Duration::from_secs(2), server_handle)
        .await
        .expect("Server timeout")
        .expect("Server task failed");

    assert!(server_result, "Server should have accepted connection");
}

#[tokio::test]
async fn test_reliable_stream() {
    let (server_crypto, client_crypto) = create_test_certs();

    let server_config = QuicConfig::server_default().with_server_crypto(server_crypto);
    let server_endpoint = QuicEndpoint::server("127.0.0.1:0".parse().unwrap(), server_config)
        .expect("Failed to create server endpoint");
    let server_addr = server_endpoint.local_addr().unwrap();

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();

    let server_handle = tokio::spawn(async move {
        if let Some(conn) = server_endpoint.accept().await {
            let utp = Arc::new(QuicUTP::new(conn, true));
            let conn = protofish::accept(utp).await.unwrap();

            let arb = conn.next_arb().await.unwrap();
            let stream = arb.wait_stream().await.unwrap();
            let (mut writer, mut reader) = stream.split();

            let mut buf = vec![2; 5];
            reader.read_exact(&mut buf).await.unwrap();
            writer.write(&buf).await.unwrap();

            writer.flush().await.unwrap();
            writer.shutdown().await.unwrap();

            ready_rx.await.unwrap();

            return true;
        }
        false
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client_config = QuicConfig::client_default().with_client_crypto(client_crypto);
    let client_endpoint = QuicEndpoint::client("127.0.0.1:0".parse().unwrap(), client_config)
        .expect("Failed to create client endpoint");

    let conn = client_endpoint
        .connect(server_addr, "localhost")
        .await
        .unwrap();
    let client_utp = Arc::new(QuicUTP::new(conn, false));
    let client_conn = protofish::connect(client_utp).await.unwrap();
    let arb = client_conn.new_arb();
    let stream = arb.new_stream(IntegrityType::Reliable).await.unwrap();
    let (mut writer, mut reader) = stream.split();

    let test_data = Bytes::from_static(b"hello");
    writer.write_all(&test_data).await.unwrap();
    writer.flush().await.unwrap();

    let mut buf = vec![2; 5];
    reader.read_exact(&mut buf).await.unwrap();

    ready_tx.send(()).unwrap();

    assert_eq!(buf, test_data);

    let server_result = timeout(Duration::from_secs(2), server_handle)
        .await
        .expect("Server timeout")
        .expect("Server task failed");

    assert!(server_result, "Server should have processed stream");
}

#[tokio::test]
async fn test_unreliable_stream() {
    let (server_crypto, client_crypto) = create_test_certs();

    let server_config = QuicConfig::server_default().with_server_crypto(server_crypto);
    let server_endpoint = QuicEndpoint::server("127.0.0.1:0".parse().unwrap(), server_config)
        .expect("Failed to create server endpoint");
    let server_addr = server_endpoint.local_addr().unwrap();

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();

    let server_handle = tokio::spawn(async move {
        if let Some(conn) = server_endpoint.accept().await {
            let utp = Arc::new(QuicUTP::new(conn, true));
            let conn = protofish::accept(utp.clone())
                .await
                .expect("failed to accept");

            let arb = conn.next_arb().await.unwrap();

            let stream = arb.wait_stream().await.unwrap();
            let (mut writer, mut reader) = stream.split();
            let mut buf = vec![2; 100];

            timeout(Duration::from_secs(2), reader.read_exact(&mut buf))
                .await
                .expect("Receive timeout")
                .unwrap();

            writer.write(&buf).await.unwrap();

            ready_rx.await.unwrap();

            return true;
        }
        false
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client_config = QuicConfig::client_default().with_client_crypto(client_crypto);
    let client_endpoint = QuicEndpoint::client("127.0.0.1:0".parse().unwrap(), client_config)
        .expect("Failed to create client endpoint");

    let conn = client_endpoint
        .connect(server_addr, "localhost")
        .await
        .unwrap();
    let client_utp = Arc::new(QuicUTP::new(conn, false));
    let client_conn = protofish::connect(client_utp).await.unwrap();
    let arb = client_conn.new_arb();

    let stream = arb.new_stream(IntegrityType::Unreliable).await.unwrap();
    let (mut writer, mut reader) = stream.split();

    let test_data = vec![1u8; 200];
    writer.write(&test_data).await.unwrap();

    let mut received = vec![2u8; 100];
    timeout(Duration::from_secs(2), reader.read_exact(&mut received))
        .await
        .expect("Receive timeout")
        .unwrap();

    ready_tx.send(()).unwrap();

    assert!(received.iter().all(|x| *x == 1));

    let server_result = timeout(Duration::from_secs(2), server_handle)
        .await
        .expect("Server timeout")
        .expect("Server task failed");

    assert!(server_result, "Server should have processed datagram");
}

#[tokio::test]
async fn test_multiple_streams() {
    let (server_crypto, client_crypto) = create_test_certs();

    let server_config = QuicConfig::server_default().with_server_crypto(server_crypto);
    let server_endpoint = QuicEndpoint::server("127.0.0.1:0".parse().unwrap(), server_config)
        .expect("Failed to create server endpoint");
    let server_addr = server_endpoint.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        if let Some(conn) = server_endpoint.accept().await {
            let utp = Arc::new(QuicUTP::new(conn, true));
            let conn = protofish::accept(utp).await.unwrap();
            for _ in 0..3 {
                let arb = conn.next_arb().await.unwrap();

                let stream = arb.wait_stream().await.unwrap();
                let (mut writer, mut reader) = stream.split();

                let mut data = vec![2u8; 10];
                reader.read_exact(&mut data).await.unwrap();
                writer.write(&data).await.unwrap();
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
            return true;
        }
        false
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client_config = QuicConfig::client_default().with_client_crypto(client_crypto);
    let client_endpoint = QuicEndpoint::client("127.0.0.1:0".parse().unwrap(), client_config)
        .expect("Failed to create client endpoint");

    let conn = client_endpoint
        .connect(server_addr, "localhost")
        .await
        .unwrap();
    let client_utp = Arc::new(QuicUTP::new(conn, false));

    let mut handles = vec![];

    let utp = Arc::clone(&client_utp);
    let conn = protofish::connect(utp).await.unwrap();

    for _ in 0..3 {
        let arb = conn.new_arb();

        let handle = tokio::spawn(async move {
            let stream = arb.new_stream(IntegrityType::Reliable).await.unwrap();
            let (mut writer, mut reader) = stream.split();

            let test_data = [5u8; 10];
            writer.write(&test_data).await.unwrap();

            let mut received = vec![9u8; 10];
            reader.read_exact(&mut received).await.unwrap();
            assert_eq!(received, test_data);
        });
        handles.push(handle);
    }

    for handle in handles {
        timeout(Duration::from_secs(2), handle)
            .await
            .expect("Stream timeout")
            .expect("Stream task failed");
    }

    let server_result = timeout(Duration::from_secs(3), server_handle)
        .await
        .expect("Server timeout")
        .expect("Server task failed");

    assert!(server_result);
}
