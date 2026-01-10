# QUICfish

A QUIC-based implementation of the Protofish Upstream Transport Protocol (UTP).

## Overview

QUICfish provides a production-ready UTP implementation that leverages QUIC's native features:
- **Multiplexing**: Efficient bidirectional stream management
- **Reliability Options**: Both reliable (streams) and unreliable (datagrams) transports
- **Built-in Security**: TLS 1.3 encryption via QUIC
- **Connection Migration**: Network path changes handled transparently
- **Low Latency**: Zero-RTT connection establishment support

## Architecture

QUICfish implements the Protofish UTP trait using Quinn (QUIC implementation):

- **Reliable Streams**: Uses QUIC bidirectional streams for lossless data transmission
- **Unreliable Streams**: Uses QUIC datagrams with stream ID multiplexing
- **Stream Management**: Automatic stream lifecycle tracking and event notification
- **Zero-Copy**: Efficient buffer management with `bytes::Bytes`

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed design documentation.

## Usage

### Basic Client

```rust
use quicfish::{connect, QuicConfig};
use protofish::IntegrityType;
use std::io::BufReader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to server (requires server certificate for verification)
    let cert_file = std::fs::File::open("cert.pem")?;
    let mut cert_reader = BufReader::new(cert_file);

    let pf_conn = connect(
        "127.0.0.1:4433".parse()?, 
        "localhost", 
        &mut cert_reader
    ).await?;
    
    // Create a new stream context
    let arb = pf_conn.new_arb();
    
    // Open a reliable stream
    let mut stream = arb.new_stream(IntegrityType::Reliable).await?;
    
    Ok(())
}
```

### Basic Server

```rust
use quicfish::create_server_endpoint;
use std::io::BufReader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create server endpoint with certificate and private key
    let cert_file = std::fs::File::open("cert.pem")?;
    let key_file = std::fs::File::open("key.pem")?;
    let mut cert_reader = BufReader::new(cert_file);
    let mut key_reader = BufReader::new(key_file);

    let endpoint = create_server_endpoint(
        "0.0.0.0:4433".parse()?, 
        &mut cert_reader, 
        &mut key_reader
    ).await?;
    
    // Accept connections
    while let Some(conn) = endpoint.accept().await {
        // Upgrade connection...
    }
    
    Ok(())
}
```

### Configuration

```rust
use quicfish::QuicConfig;
use std::time::Duration;

let config = QuicConfig::client_default()
    .with_max_idle_timeout(Duration::from_secs(60))
    .with_keep_alive_interval(Duration::from_secs(15))
    .with_max_datagram_size(1400);
```

## Examples

The repository includes examples for an echo server and client.

### Echo Server

To run the echo server:

```bash
cargo run --example echo_server
```

This will:
1. Generate a self-signed certificate (`cert.pem`)
2. Start a server on `127.0.0.1:4433`
3. Echo back any data received on streams

### Echo Client

To run the echo client (after starting the server):

```bash
cargo run --example echo_client
```

This will:
1. Connect to the server using the generated `cert.pem`
2. Send "Hello from client!"
3. Print the server's response

## Features

- ✅ Reliable bidirectional streams
- ✅ Unreliable datagram-based streams  
- ✅ Stream multiplexing and demultiplexing
- ✅ Automatic event notification
- ✅ Configurable transport parameters
- ✅ Comprehensive error handling
- ✅ Zero-copy buffer management

## Development Status

**Phase 1: Foundation** ✅ Complete
- Core modules implemented
- UTP trait fully implemented
- Unit tests passing

**Phase 2: Integration** 🚧 In Progress
- Integration tests needed
- Example applications needed
- Protofish integration testing needed

**Phase 3: Polish** 📋 Planned
- Performance benchmarks
- Production hardening
- Documentation completion

## Dependencies

- `quinn` - QUIC protocol implementation
- `tokio` - Async runtime
- `bytes` - Efficient byte buffers
- `rustls` - TLS implementation
- `protofish` - Protofish protocol implementation

## Testing

```bash
# Run unit tests
cargo test --lib

# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

## Documentation

```bash
# Generate and open documentation
cargo doc --open
```

## License

[Your License Here]

## References

- [Protofish Specification](https://github.com/zako-ac/protofish/blob/main/protofish/protofish.md)
- [QUICfish Specification](https://github.com/zako-ac/protofish/blob/main/protofish/quicfish.md)
- [QUIC RFC 9000](https://www.rfc-editor.org/rfc/rfc9000.html)
- [Quinn Documentation](https://docs.rs/quinn/)
