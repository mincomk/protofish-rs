use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use quicfish::{QuicConfig, QuicEndpoint, QuicUTP};
use protofish::utp::{UTP, UTPStream, UTPEvent};
use protofish::IntegrityType;
use bytes::Bytes;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

#[path = "../tests/common.rs"]
mod common;
use common::create_test_certs;

async fn setup_connection() -> (Arc<QuicUTP>, Arc<QuicUTP>) {
    let (server_crypto, client_crypto) = create_test_certs();

    let server_config = QuicConfig::server_default()
        .with_server_crypto(server_crypto);
    let server_endpoint = QuicEndpoint::server("127.0.0.1:0".parse().unwrap(), server_config)
        .expect("Failed to create server endpoint");
    let server_addr = server_endpoint.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        let conn = server_endpoint.accept().await.expect("Failed to accept connection");
        Arc::new(QuicUTP::new(conn))
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let client_config = QuicConfig::client_default()
        .with_client_crypto(client_crypto);
    let client_endpoint = QuicEndpoint::client("127.0.0.1:0".parse().unwrap(), client_config)
        .expect("Failed to create client endpoint");
    let client_conn = client_endpoint.connect(server_addr, "localhost").await
        .expect("Failed to connect");
    let client_utp = Arc::new(QuicUTP::new(client_conn));

    let server_utp = server_handle.await.expect("Server task failed");

    (client_utp, server_utp)
}

fn bench_reliable_stream_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("reliable_stream_throughput");
    
    // Test different payload sizes
    for size in [1024, 4096, 16384, 65536, 262144].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let (client_utp, server_utp) = setup_connection().await;
                
                // Setup server to echo data back
                let server_handle = {
                    let server_utp = server_utp.clone();
                    tokio::spawn(async move {
                        if let UTPEvent::NewStream(stream_id) = server_utp.next_event().await {
                            let stream = server_utp.wait_stream(stream_id).await.unwrap();
                            let data = stream.receive(size).await.unwrap();
                            stream.send(&data).await.unwrap();
                        }
                    })
                };

                // Client sends and receives data
                let stream = client_utp.open_stream(IntegrityType::Reliable).await.unwrap();
                let data = Bytes::from(vec![0u8; size]);
                stream.send(&data).await.unwrap();
                let _response = stream.receive(size).await.unwrap();
                
                server_handle.await.unwrap();
            });
        });
    }
    
    group.finish();
}

fn bench_unreliable_stream_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("unreliable_stream_throughput");
    
    // Test different payload sizes (smaller for unreliable/datagram)
    for size in [512, 1024, 2048, 4096].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let (client_utp, server_utp) = setup_connection().await;
                
                let server_handle = {
                    let server_utp = server_utp.clone();
                    tokio::spawn(async move {
                        if let UTPEvent::NewStream(stream_id) = server_utp.next_event().await {
                            let stream = server_utp.wait_stream(stream_id).await.unwrap();
                            let data = stream.receive(size).await.unwrap();
                            stream.send(&data).await.unwrap();
                        }
                    })
                };

                let stream = client_utp.open_stream(IntegrityType::Unreliable).await.unwrap();
                let data = Bytes::from(vec![0u8; size]);
                stream.send(&data).await.unwrap();
                let _response = stream.receive(size).await.unwrap();
                
                server_handle.await.unwrap();
            });
        });
    }
    
    group.finish();
}

fn bench_concurrent_streams(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_streams");
    
    // Test different numbers of concurrent streams
    for num_streams in [1, 5, 10, 20, 50].iter() {
        let total_bytes = *num_streams * 4096u64;
        group.throughput(Throughput::Bytes(total_bytes));
        
        group.bench_with_input(BenchmarkId::from_parameter(num_streams), num_streams, |b, &num_streams| {
            b.to_async(&rt).iter(|| async {
                let (client_utp, server_utp) = setup_connection().await;
                
                // Server handles multiple streams
                let server_handle = {
                    let server_utp = server_utp.clone();
                    tokio::spawn(async move {
                        let mut handles = vec![];
                        for _ in 0..num_streams {
                            let server_utp = server_utp.clone();
                            let handle = tokio::spawn(async move {
                                if let UTPEvent::NewStream(stream_id) = server_utp.next_event().await {
                                    let stream = server_utp.wait_stream(stream_id).await.unwrap();
                                    let data = stream.receive(4096).await.unwrap();
                                    stream.send(&data).await.unwrap();
                                }
                            });
                            handles.push(handle);
                        }
                        for h in handles {
                            h.await.unwrap();
                        }
                    })
                };

                // Client creates multiple streams
                let mut handles = vec![];
                for _ in 0..num_streams {
                    let client_utp = client_utp.clone();
                    let handle = tokio::spawn(async move {
                        let stream = client_utp.open_stream(IntegrityType::Reliable).await.unwrap();
                        let data = Bytes::from(vec![0u8; 4096]);
                        stream.send(&data).await.unwrap();
                        let _response = stream.receive(4096).await.unwrap();
                    });
                    handles.push(handle);
                }
                
                for h in handles {
                    h.await.unwrap();
                }
                
                server_handle.await.unwrap();
            });
        });
    }
    
    group.finish();
}

fn bench_bulk_transfer(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("bulk_transfer");
    group.sample_size(10); // Fewer samples for long-running tests
    
    // Test transferring large amounts of data
    for total_mb in [1, 5, 10].iter() {
        let total_bytes = *total_mb * 1024 * 1024u64;
        group.throughput(Throughput::Bytes(total_bytes));
        
        group.bench_with_input(BenchmarkId::from_parameter(format!("{}MB", total_mb)), total_mb, |b, &total_mb| {
            b.to_async(&rt).iter(|| async {
                let (client_utp, server_utp) = setup_connection().await;
                let chunk_size: usize = 65536;
                let num_chunks = (total_mb * 1024 * 1024) / chunk_size as u64;
                
                let server_handle = {
                    let server_utp = server_utp.clone();
                    tokio::spawn(async move {
                        if let UTPEvent::NewStream(stream_id) = server_utp.next_event().await {
                            let stream = server_utp.wait_stream(stream_id).await.unwrap();
                            for _ in 0..num_chunks {
                                let data = stream.receive(chunk_size).await.unwrap();
                                stream.send(&data).await.unwrap();
                            }
                        }
                    })
                };

                let stream = client_utp.open_stream(IntegrityType::Reliable).await.unwrap();
                let data = Bytes::from(vec![0u8; chunk_size]);
                for _ in 0..num_chunks {
                    stream.send(&data).await.unwrap();
                    let _response = stream.receive(chunk_size).await.unwrap();
                }
                
                server_handle.await.unwrap();
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_reliable_stream_throughput,
    bench_unreliable_stream_throughput,
    bench_concurrent_streams,
    bench_bulk_transfer
);
criterion_main!(benches);
