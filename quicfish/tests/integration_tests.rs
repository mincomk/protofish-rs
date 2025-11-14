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

    // Server setup
    let server_config = QuicConfig::server_default().with_server_crypto(server_crypto);
    let server_endpoint = QuicEndpoint::server("127.0.0.1:0".parse().unwrap(), server_config)
        .expect("Failed to create server endpoint");
    let server_addr = server_endpoint.local_addr().unwrap();

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        if let Some(conn) = server_endpoint.accept().await {
            let _utp = QuicUTP::new(conn);
            // Server created successfully
            true
        } else {
            false
        }
    });

    // Client setup
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client_config = QuicConfig::client_default().with_client_crypto(client_crypto);
    let client_endpoint = QuicEndpoint::client("127.0.0.1:0".parse().unwrap(), client_config)
        .expect("Failed to create client endpoint");

    // Connect
    let conn = client_endpoint
        .connect(server_addr, "localhost")
        .await
        .expect("Failed to connect");

    let _client_utp = QuicUTP::new(conn);

    // Verify server accepted
    let server_result = timeout(Duration::from_secs(2), server_handle)
        .await
        .expect("Server timeout")
        .expect("Server task failed");

    assert!(server_result, "Server should have accepted connection");
}

#[tokio::test]
async fn test_reliable_stream() {
    let (server_crypto, client_crypto) = create_test_certs();

    // Server setup
    let server_config = QuicConfig::server_default().with_server_crypto(server_crypto);
    let server_endpoint = QuicEndpoint::server("127.0.0.1:0".parse().unwrap(), server_config)
        .expect("Failed to create server endpoint");
    let server_addr = server_endpoint.local_addr().unwrap();

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        if let Some(conn) = server_endpoint.accept().await {
            let utp = Arc::new(QuicUTP::new(conn));
            let conn = protofish::accept(utp).await.unwrap();

            let arb = conn.next_arb().await.unwrap();
            let mut stream = arb.wait_stream().await.unwrap();

            let mut buf = vec![2; 5];
            stream.read_exact(&mut buf).await.unwrap();
            stream.write(&buf).await.unwrap();

            return true;
        }
        false
    });

    // Client setup
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client_config = QuicConfig::client_default().with_client_crypto(client_crypto);
    let client_endpoint = QuicEndpoint::client("127.0.0.1:0".parse().unwrap(), client_config)
        .expect("Failed to create client endpoint");

    let conn = client_endpoint
        .connect(server_addr, "localhost")
        .await
        .unwrap();
    let client_utp = Arc::new(QuicUTP::new(conn));
    println!("c1");
    let client_conn = protofish::connect(client_utp).await.unwrap();
    println!("c2");
    let arb = client_conn.new_arb();
    println!("c3");
    let mut stream = arb.new_stream(IntegrityType::Reliable).await.unwrap();
    println!("c4");

    let test_data = Bytes::from_static(b"hello");
    stream.write(&test_data).await.unwrap();

    let mut buf = vec![2; 5];
    stream.read_exact(&mut buf).await.unwrap();
    assert_eq!(buf, test_data);

    // Verify server completed
    let server_result = timeout(Duration::from_secs(2), server_handle)
        .await
        .expect("Server timeout")
        .expect("Server task failed");

    assert!(server_result, "Server should have processed stream");
}

#[tokio::test]
async fn test_unreliable_stream() {
    let (server_crypto, client_crypto) = create_test_certs();

    // Server setup
    let server_config = QuicConfig::server_default().with_server_crypto(server_crypto);
    let server_endpoint = QuicEndpoint::server("127.0.0.1:0".parse().unwrap(), server_config)
        .expect("Failed to create server endpoint");
    let server_addr = server_endpoint.local_addr().unwrap();

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        if let Some(conn) = server_endpoint.accept().await {
            let utp = Arc::new(QuicUTP::new(conn));
            let conn = protofish::accept(utp.clone())
                .await
                .expect("failed to accept");

            let arb = conn.next_arb().await.unwrap();

            let mut stream = arb.wait_stream().await.unwrap();
            println!("0-");
            let mut buf = vec![2; 100];

            println!("0");

            timeout(Duration::from_secs(2), stream.read_exact(&mut buf))
                .await
                .expect("Receive timeout")
                .unwrap();

            println!("1");

            stream.write(&buf).await.unwrap();

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
    let client_utp = Arc::new(QuicUTP::new(conn));
    let client_conn = protofish::connect(client_utp).await.unwrap();
    let arb = client_conn.new_arb();

    let mut stream = arb.new_stream(IntegrityType::Unreliable).await.unwrap();

    let test_data = vec![1u8; 200];
    stream.write(&test_data).await.unwrap();

    let mut received = vec![2u8; 100];
    timeout(Duration::from_secs(2), stream.read_exact(&mut received))
        .await
        .expect("Receive timeout")
        .unwrap();

    assert_eq!(received, test_data);

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

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        if let Some(conn) = server_endpoint.accept().await {
            let utp = Arc::new(QuicUTP::new(conn));
            let conn = protofish::accept(utp).await.unwrap();
            let arb = conn.next_arb().await.unwrap();

            // Accept 3 streams
            for _ in 0..3 {
                let mut stream = arb.wait_stream().await.unwrap();

                let mut data = vec![2u8; 10];
                stream.read_exact(&mut data).await.unwrap();
                stream.write(&data).await.unwrap();
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
    let client_utp = Arc::new(QuicUTP::new(conn));

    // Open 3 concurrent streams
    let mut handles = vec![];

    for i in 0..3 {
        let utp = Arc::clone(&client_utp);
        let conn = protofish::connect(utp).await.unwrap();
        let arb = conn.new_arb();

        let handle = tokio::spawn(async move {
            let mut stream = arb.new_stream(IntegrityType::Reliable).await.unwrap();

            let test_data = Bytes::from(format!("streamddd{}", i));
            stream.write(&test_data).await.unwrap();

            let mut received = vec![9u8; 10];
            stream.read_exact(&mut received).await.unwrap();
            assert_eq!(received, test_data);
        });
        handles.push(handle);
    }

    // Wait for all streams
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
