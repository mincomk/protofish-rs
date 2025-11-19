use bytes::Bytes;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use protofish::IntegrityType;
use quicfish::{QuicConfig, QuicEndpoint, QuicUTP};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Runtime;

#[path = "../tests/common.rs"]
mod common;
use common::create_test_certs;

fn spawn_iter_server(
    server: quicfish::ArbContext,
    size: usize,
    iters: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let stream = server.wait_stream().await.unwrap();
        let (_writer, mut reader) = stream.split();

        let mut data = vec![0; size];
        for _ in 0..iters {
            reader.read_exact(&mut data).await.unwrap();
        }
    })
}

async fn setup_connection() -> (quicfish::ArbContext, quicfish::ArbContext) {
    let _ = tracing_subscriber::fmt().try_init();

    let (server_crypto, client_crypto) = create_test_certs();

    let server_config = QuicConfig::server_default().with_server_crypto(server_crypto);
    let server_endpoint = QuicEndpoint::server("127.0.0.1:0".parse().unwrap(), server_config)
        .expect("Failed to create server endpoint");
    let server_addr = server_endpoint.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        let quic_conn = server_endpoint
            .accept()
            .await
            .expect("Failed to accept connection");

        let conn = protofish::accept(Arc::new(QuicUTP::new(quic_conn, true)))
            .await
            .unwrap();

        let arb = conn.next_arb().await.unwrap();

        arb.wait_stream().await.unwrap(); // dummy stream

        arb
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let client_config = QuicConfig::client_default().with_client_crypto(client_crypto);
    let client_endpoint = QuicEndpoint::client("127.0.0.1:0".parse().unwrap(), client_config)
        .expect("Failed to create client endpoint");
    let client_conn = client_endpoint
        .connect(server_addr, "localhost")
        .await
        .expect("Failed to connect");
    let client_utp = Arc::new(QuicUTP::new(client_conn, false));

    let conn_client = protofish::connect(client_utp).await.unwrap();
    let arb_client = conn_client.new_arb();
    arb_client
        .new_stream(IntegrityType::Reliable)
        .await
        .unwrap();

    let arb_server = server_handle.await.expect("Server task failed");

    (arb_client, arb_server)
}

fn bench_reliable_stream_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("reliable_stream_throughput");

    for size in [1024, 4096, 16384, 65536, 262144].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        group.sample_size(100);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter_custom(|iters| async move {
                let (client, server) = setup_connection().await;

                let server_handle = spawn_iter_server(server, size, iters);

                let stream = client.new_stream(IntegrityType::Reliable).await.unwrap();
                let (mut writer, _reader) = stream.split();
                let data = Bytes::from(vec![0u8; size]);

                let start = Instant::now();

                for _ in 0..iters {
                    writer.write_all(&data).await.unwrap();
                }

                server_handle.await.unwrap();

                start.elapsed()
            });
        });
    }

    group.finish();
}

fn bench_unreliable_stream_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("unreliable_stream_throughput");

    for size in [512, 1024, 2048, 4096].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        group.sample_size(90);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter_custom(|iters| async move {
                let (client, server) = setup_connection().await;

                let lossy_size = size * (9 / 10);
                let server_handle = spawn_iter_server(server, lossy_size, iters);

                let stream = client.new_stream(IntegrityType::Unreliable).await.unwrap();
                let (mut writer, _reader) = stream.split();

                let data = Bytes::from(vec![0u8; size]);

                let start = Instant::now();

                for _ in 0..iters {
                    writer.write_all(&data).await.unwrap();
                }

                server_handle.await.unwrap();

                start.elapsed()
            });
        });
    }

    group.finish();
}

fn bench_concurrent_streams(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_streams");

    for num_streams in [1, 5, 10, 20, 50].iter() {
        let total_bytes = *num_streams * 4096u64;
        group.throughput(Throughput::Bytes(total_bytes));

        group.sample_size(80);

        group.bench_with_input(
            BenchmarkId::from_parameter(num_streams),
            num_streams,
            |b, &num_streams| {
                b.to_async(&rt).iter(|| async {
                    let (client, server) = setup_connection().await;

                    let server_handle = {
                        tokio::spawn(async move {
                            for _ in 0..num_streams {
                                let stream = server.wait_stream().await.unwrap();
                                let (_writer, mut reader) = stream.split();

                                let mut data = vec![0; 4096];
                                reader.read_exact(&mut data).await.unwrap();
                            }
                        })
                    };

                    for _ in 0..num_streams {
                        let stream = client.new_stream(IntegrityType::Reliable).await.unwrap();
                        let (mut writer, _reader) = stream.split();

                        let data = Bytes::from(vec![0u8; 4096]);
                        writer.write_all(&data).await.unwrap();
                    }

                    server_handle.await.unwrap();
                });
            },
        );
    }

    group.finish();
}

fn bench_bulk_transfer(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("bulk_transfer");
    group.sample_size(70);

    // Test transferring large amounts of data
    for total_mb in [1, 5, 10].iter() {
        let total_bytes = *total_mb * 1024 * 1024u64;
        group.throughput(Throughput::Bytes(total_bytes));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}MB", total_mb)),
            total_mb,
            |b, &total_mb| {
                b.to_async(&rt).iter(|| async {
                    let (client, server) = setup_connection().await;
                    let chunk_size: usize = 65536;
                    let num_chunks = (total_mb * 1024 * 1024) / chunk_size as u64;

                    let server_handle = {
                        tokio::spawn(async move {
                            let stream = server.wait_stream().await.unwrap();
                            let (mut writer, mut reader) = stream.split();

                            for _ in 0..num_chunks {
                                let mut data = vec![0; chunk_size];
                                reader.read_exact(&mut data).await.unwrap();

                                writer.write_all(&data).await.unwrap();
                            }
                        })
                    };

                    let stream = client.new_stream(IntegrityType::Reliable).await.unwrap();
                    let (mut writer, mut reader) = stream.split();

                    for _ in 0..num_chunks {
                        let data = Bytes::from(vec![0u8; chunk_size]);
                        writer.write_all(&data).await.unwrap();

                        let mut response = vec![0; chunk_size];
                        reader.read_exact(&mut response).await.unwrap();
                    }

                    server_handle.await.unwrap();
                });
            },
        );
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
