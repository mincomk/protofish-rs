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
use quicfish::{QuicConfig, QuicEndpoint, QuicUTP};
use protofish::connect;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create client endpoint
    let config = QuicConfig::client_default();
    let endpoint = QuicEndpoint::client("0.0.0.0:0".parse()?, config)?;
    
    // Connect to server
    let connection = endpoint.connect("127.0.0.1:4433".parse()?, "localhost").await?;
    
    // Create UTP instance
    let utp = Arc::new(QuicUTP::new(connection));
    
    // Use with Protofish
    let protofish_conn = connect(utp).await?;
    
    // Now use protofish connection...
    let (writer, reader) = protofish_conn.new_arb();
    
    Ok(())
}
```

### Basic Server

```rust
use quicfish::{QuicConfig, QuicEndpoint, QuicUTP};
use protofish::accept;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create server endpoint with TLS config
    let config = QuicConfig::server_default()
        .with_server_crypto(/* your rustls::ServerConfig */);
    
    let endpoint = QuicEndpoint::server("0.0.0.0:4433".parse()?, config)?;
    
    println!("Server listening on {}", endpoint.local_addr()?);
    
    // Accept connections
    while let Some(connection) = endpoint.accept().await {
        tokio::spawn(async move {
            let utp = Arc::new(QuicUTP::new(connection));
            
            match accept(utp).await {
                Ok(protofish_conn) => {
                    // Handle protofish connection
                    loop {
                        match protofish_conn.next_arb().await {
                            Ok(ctx) => {
                                // Handle context
                            }
                            Err(e) => break,
                        }
                    }
                }
                Err(e) => eprintln!("Accept error: {}", e),
            }
        });
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
