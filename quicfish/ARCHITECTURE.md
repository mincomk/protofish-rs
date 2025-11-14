# QUICfish Architecture Plan

## Overview
QUICfish is a QUIC-based implementation of the Protofish Upstream Transport Protocol (UTP). It leverages QUIC's native multiplexing, reliability options, and stream management to provide an elegant, production-ready UTP implementation.

## Design Goals
1. **Elegance**: Clean separation of concerns with minimal abstraction layers
2. **Correctness**: Full compliance with Protofish specification
3. **Performance**: Zero-copy where possible, efficient async operations
4. **Maintainability**: Clear module boundaries, well-documented code
5. **Testability**: Comprehensive unit and integration tests

## Core Components

### 1. Module Structure

```
quicfish/
├── Cargo.toml
├── ARCHITECTURE.md (this file)
├── README.md
├── src/
│   ├── lib.rs                    # Public API exports
│   ├── config.rs                 # QUIC configuration
│   ├── endpoint.rs               # Endpoint management (client/server)
│   ├── connection.rs             # QuicUTP implementation
│   ├── stream.rs                 # QuicUTPStream implementation
│   ├── error.rs                  # Error types
│   ├── datagram.rs               # Unreliable stream (datagram) routing
│   └── tests/
│       ├── mod.rs
│       ├── basic.rs              # Basic stream tests
│       ├── reliable.rs           # Reliable stream tests
│       ├── unreliable.rs         # Unreliable stream tests
│       └── integration.rs        # End-to-end tests
└── examples/
    ├── echo_client.rs            # Simple echo client
    ├── echo_server.rs            # Simple echo server
    └── protofish_integration.rs  # Full Protofish integration
```

### 2. Component Responsibilities

#### `config.rs` - Configuration Management
**Purpose**: Centralize QUIC configuration with sensible defaults

**Key structures**:
```rust
pub struct QuicConfig {
    pub server_name: String,
    pub max_idle_timeout: Duration,
    pub keep_alive_interval: Duration,
    pub max_concurrent_streams: u64,
    pub max_datagram_size: usize,
}

impl QuicConfig {
    pub fn client_default() -> Self;
    pub fn server_default() -> Self;
    pub fn with_certs(self, cert: Certificate, key: PrivateKey) -> Self;
}
```

**Responsibilities**:
- TLS configuration
- Transport parameters
- Endpoint behavior tuning
- Certificate management

#### `endpoint.rs` - Endpoint Management
**Purpose**: Handle QUIC endpoint lifecycle for both client and server roles

**Key structures**:
```rust
pub struct QuicEndpointBuilder {
    config: QuicConfig,
    bind_addr: SocketAddr,
    role: EndpointRole,
}

pub enum EndpointRole {
    Client,
    Server { incoming_tx: mpsc::Sender<Connection> },
}

impl QuicEndpointBuilder {
    pub fn new_client(bind_addr: SocketAddr) -> Self;
    pub fn new_server(bind_addr: SocketAddr) -> Self;
    pub fn with_config(self, config: QuicConfig) -> Self;
    pub fn build(self) -> Result<Endpoint, Error>;
}
```

**Responsibilities**:
- Endpoint creation and configuration
- Certificate/TLS setup
- Bind address management
- Connection acceptance (server role)

#### `connection.rs` - QuicUTP Implementation
**Purpose**: Core UTP trait implementation using QUIC connections

**Key structure**:
```rust
pub struct QuicUTP {
    // Core connection state
    connection: Arc<Mutex<Option<Connection>>>,
    
    // Stream management
    streams: Arc<RwLock<HashMap<StreamId, QuicUTPStream>>>,
    next_stream_id: AtomicU64,
    
    // Event handling
    event_tx: mpsc::UnboundedSender<UTPEvent>,
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<UTPEvent>>>,
    
    // Datagram routing
    datagram_router: Arc<DatagramRouter>,
    
    // Shutdown coordination
    shutdown_tx: broadcast::Sender<()>,
}

impl QuicUTP {
    pub fn new(connection: Connection) -> Self;
    pub fn from_endpoint(endpoint: Endpoint, server_addr: SocketAddr) -> Self;
    
    // Internal helpers
    async fn next_id(&self) -> StreamId;
    fn spawn_stream_listener(&self);
    fn spawn_datagram_listener(&self);
}

impl UTP for QuicUTP {
    type Stream = QuicUTPStream;
    
    async fn connect(&self) -> Result<(), UTPError>;
    async fn next_event(&self) -> UTPEvent;
    async fn open_stream(&self, integrity: IntegrityType) -> Result<Self::Stream, UTPError>;
    async fn wait_stream(&self, id: StreamId) -> Result<Self::Stream, UTPError>;
}
```

**Responsibilities**:
- UTP trait implementation
- Connection state management
- Stream lifecycle coordination
- Event dispatch
- Background task management (listeners)

#### `stream.rs` - QuicUTPStream Implementation
**Purpose**: Individual stream abstraction for both reliable and unreliable streams

**Key structures**:
```rust
pub struct QuicUTPStream {
    id: StreamId,
    inner: StreamInner,
}

enum StreamInner {
    Reliable(ReliableStream),
    Unreliable(UnreliableStream),
}

struct ReliableStream {
    send: Arc<Mutex<SendStream>>,
    recv: Arc<Mutex<RecvStream>>,
}

struct UnreliableStream {
    connection: Weak<Mutex<Option<Connection>>>,
    recv_queue: Arc<Mutex<mpsc::UnboundedReceiver<Bytes>>>,
}

impl UTPStream for QuicUTPStream {
    fn id(&self) -> StreamId;
    async fn send(&self, data: &Bytes) -> Result<(), UTPError>;
    async fn receive(&self, len: usize) -> Result<Bytes, UTPError>;
    async fn close(&self) -> Result<(), UTPError>;
}
```

**Responsibilities**:
- Stream-level send/receive operations
- Reliable vs unreliable stream behavior
- Stream closure
- Error translation

#### `datagram.rs` - Datagram Routing
**Purpose**: Multiplexing and demultiplexing datagrams for unreliable streams

**Key structure**:
```rust
pub struct DatagramRouter {
    // Map stream ID to receive channel
    channels: RwLock<HashMap<StreamId, mpsc::UnboundedSender<Bytes>>>,
    
    // Handle for sending datagrams
    connection: Weak<Mutex<Option<Connection>>>,
}

impl DatagramRouter {
    pub fn new(connection: Weak<Mutex<Option<Connection>>>) -> Self;
    
    pub async fn register_stream(&self, id: StreamId) -> mpsc::UnboundedReceiver<Bytes>;
    pub async fn unregister_stream(&self, id: StreamId);
    
    pub async fn send_datagram(&self, id: StreamId, data: Bytes) -> Result<(), Error>;
    pub async fn route_incoming(&self, datagram: Bytes);
    
    // Frame format: [stream_id: u64][payload: bytes]
    fn encode_datagram(id: StreamId, data: &Bytes) -> Bytes;
    fn decode_datagram(datagram: Bytes) -> Option<(StreamId, Bytes)>;
}
```

**Responsibilities**:
- Datagram framing (prepend stream ID)
- Routing incoming datagrams to correct stream
- Channel management for unreliable streams
- Zero-copy where possible

#### `error.rs` - Error Handling
**Purpose**: Comprehensive error types with context

**Key types**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("QUIC connection error: {0}")]
    Connection(#[from] quinn::ConnectionError),
    
    #[error("QUIC stream error: {0}")]
    Stream(String),
    
    #[error("Datagram error: {0}")]
    Datagram(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),
}

// Conversions to UTPError
impl From<Error> for UTPError { ... }
```

**Responsibilities**:
- Error type definitions
- Conversion to/from protofish `UTPError`
- Error context preservation

## Design Patterns

### 1. Stream ID Management
- Use `AtomicU64` for lock-free stream ID generation
- Client uses even IDs, server uses odd IDs (following QUIC convention)
- StreamId wrapper type for type safety

### 2. Connection Lifecycle
```
Endpoint Created → Connection Established → Listeners Spawned → Streams Active → Shutdown → Cleanup
```

### 3. Event Flow
```
QUIC Event (new stream/datagram) → Background Listener → Channel Send → UTP Event → Application
```

### 4. Graceful Shutdown
- Use `broadcast::Sender<()>` for shutdown signal
- All background tasks subscribe to shutdown channel
- Clean up resources in order: streams → listeners → connection

### 5. Zero-Copy Optimization
- Use `Bytes` throughout for efficient buffer sharing
- Avoid unnecessary allocations in hot paths
- Leverage QUIC's native buffer management

## API Usage Examples

### Client Example
```rust
use quicfish::{QuicConfig, QuicEndpointBuilder, QuicUTP};
use protofish::{connect, utp::UTP};

let config = QuicConfig::client_default();
let endpoint = QuicEndpointBuilder::new_client("0.0.0.0:0".parse()?)
    .with_config(config)
    .build()?;

let connection = endpoint.connect("server.example.com:4433".parse()?, "server.example.com")?
    .await?;

let utp = Arc::new(QuicUTP::new(connection));
let protofish_conn = connect(utp).await?;

// Use protofish connection...
```

### Server Example
```rust
use quicfish::{QuicConfig, QuicEndpointBuilder};

let config = QuicConfig::server_default()
    .with_certs(cert, key);

let endpoint = QuicEndpointBuilder::new_server("0.0.0.0:4433".parse()?)
    .with_config(config)
    .build()?;

while let Some(connection) = endpoint.accept().await {
    tokio::spawn(async move {
        let utp = Arc::new(QuicUTP::new(connection));
        let protofish_conn = accept(utp).await?;
        
        // Handle protofish connection...
    });
}
```

## Testing Strategy

### Unit Tests
- Each module has inline tests
- Mock QUIC connections where appropriate
- Test error paths explicitly

### Integration Tests
- Full client-server scenarios
- Multiple concurrent streams
- Mixed reliable/unreliable streams
- Connection drops and recovery

### Property Tests
- Use `proptest` for stream ID generation
- Datagram framing/parsing roundtrips
- Concurrent access patterns

### Performance Tests
- Benchmark stream creation overhead
- Throughput for reliable vs unreliable
- Latency measurements
- Memory usage under load

## Dependencies

### Required
- `quinn` (0.11+) - QUIC implementation
- `tokio` (1.x) - Async runtime
- `bytes` (1.x) - Efficient byte buffers
- `async-trait` - Trait async methods
- `thiserror` - Error handling

### Development
- `proptest` - Property-based testing
- `criterion` - Benchmarking
- `rcgen` - Certificate generation for tests

## Migration from PoC

The `proposal.rs` PoC contains the core ideas but needs refinement:

### Keep
- Overall structure (QuicUTP and QuicUTPStream)
- Reliable/unreliable stream distinction
- Datagram routing concept
- Event channel pattern

### Improve
- **Connection management**: Better initialization, no `Option<Connection>`
- **Error handling**: Rich error types, not just strings
- **Stream tracking**: Use `RwLock` instead of `Mutex` for read-heavy workload
- **Datagram router**: Extract to separate module
- **Configuration**: Centralized config management
- **Shutdown**: Proper cleanup and resource management
- **Testing**: Comprehensive test suite
- **Documentation**: Inline docs for all public APIs

### Add
- Endpoint builder pattern
- Configuration module
- Examples
- Integration tests
- Performance benchmarks

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Set up Cargo.toml with dependencies
- [ ] Implement `config.rs` with defaults
- [ ] Implement `error.rs` with proper types
- [ ] Implement `endpoint.rs` builder pattern

### Phase 2: Core UTP (Week 2)
- [ ] Implement `connection.rs` (QuicUTP)
- [ ] Implement `stream.rs` (QuicUTPStream)
- [ ] Implement `datagram.rs` router
- [ ] Unit tests for each module

### Phase 3: Integration (Week 3)
- [ ] Write integration tests
- [ ] Create examples (echo client/server)
- [ ] Test with protofish crate
- [ ] Fix bugs and edge cases

### Phase 4: Polish (Week 4)
- [ ] Documentation pass
- [ ] Performance benchmarks
- [ ] Cleanup and refactor
- [ ] README and usage guide

## Open Questions

1. **Stream ID allocation**: Should we track used IDs to prevent reuse within a connection?
   - *Proposal*: Use monotonic counter, rely on QUIC's stream ID space (2^62)

2. **Datagram size limits**: How to handle oversized datagrams?
   - *Proposal*: Return error, document max size in config

3. **Backpressure**: How to handle slow receivers?
   - *Proposal*: Let QUIC handle flow control, expose it through errors

4. **Connection migration**: Support QUIC's connection migration?
   - *Proposal*: Phase 2 feature, not MVP

5. **Multiple connections per endpoint**: Should QuicUTP own the endpoint?
   - *Proposal*: No, take `Connection` in constructor, let user manage endpoint

## References

- [Protofish Specification](https://github.com/zako-ac/protofish/blob/main/protofish/protofish.md)
- [QUICfish Specification](https://github.com/zako-ac/protofish/blob/main/protofish/quicfish.md)
- [Quinn Documentation](https://docs.rs/quinn/)
- [QUIC RFC 9000](https://www.rfc-editor.org/rfc/rfc9000.html)
- [Protofish Rust Implementation](../protofish/)
