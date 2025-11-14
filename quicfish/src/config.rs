use std::sync::Arc;
use std::time::Duration;

pub struct QuicConfig {
    pub server_name: Option<String>,
    pub max_idle_timeout: Duration,
    pub keep_alive_interval: Duration,
    pub max_concurrent_bidi_streams: u64,
    pub max_concurrent_uni_streams: u64,
    pub max_datagram_size: usize,
    pub crypto: CryptoConfig,
}

pub enum CryptoConfig {
    Client {
        crypto: rustls::ClientConfig,
    },
    Server {
        crypto: rustls::ServerConfig,
    },
}

impl QuicConfig {
    pub fn client_default() -> Self {
        let crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
            .with_no_client_auth();

        Self {
            server_name: None,
            max_idle_timeout: Duration::from_secs(30),
            keep_alive_interval: Duration::from_secs(10),
            max_concurrent_bidi_streams: 100u64.into(),
            max_concurrent_uni_streams: 100u64.into(),
            max_datagram_size: 1200,
            crypto: CryptoConfig::Client { crypto },
        }
    }

    pub fn server_default() -> Self {
        let crypto = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_cert_resolver(Arc::new(DummyCertResolver));

        Self {
            server_name: None,
            max_idle_timeout: Duration::from_secs(30),
            keep_alive_interval: Duration::from_secs(10),
            max_concurrent_bidi_streams: 100u64.into(),
            max_concurrent_uni_streams: 100u64.into(),
            max_datagram_size: 1200,
            crypto: CryptoConfig::Server { crypto },
        }
    }

    pub fn with_server_name(mut self, name: String) -> Self {
        self.server_name = Some(name);
        self
    }

    pub fn with_max_idle_timeout(mut self, timeout: Duration) -> Self {
        self.max_idle_timeout = timeout;
        self
    }

    pub fn with_keep_alive_interval(mut self, interval: Duration) -> Self {
        self.keep_alive_interval = interval;
        self
    }

    pub fn with_max_datagram_size(mut self, size: usize) -> Self {
        self.max_datagram_size = size;
        self
    }

    pub fn with_client_crypto(mut self, crypto: rustls::ClientConfig) -> Self {
        self.crypto = CryptoConfig::Client { crypto };
        self
    }

    pub fn with_server_crypto(mut self, crypto: rustls::ServerConfig) -> Self {
        self.crypto = CryptoConfig::Server { crypto };
        self
    }

    pub(crate) fn into_quinn_client_config(self) -> crate::error::Result<quinn::ClientConfig> {
        let crypto = match self.crypto {
            CryptoConfig::Client { crypto } => crypto,
            CryptoConfig::Server { .. } => {
                return Err(crate::error::Error::Config(
                    "Server crypto config provided for client".to_string(),
                ))
            }
        };

        let mut transport = quinn::TransportConfig::default();
        transport.max_idle_timeout(Some(self.max_idle_timeout.try_into().unwrap()));
        transport.keep_alive_interval(Some(self.keep_alive_interval));
        transport.max_concurrent_bidi_streams(quinn::VarInt::from_u64(self.max_concurrent_bidi_streams).unwrap());
        transport.max_concurrent_uni_streams(quinn::VarInt::from_u64(self.max_concurrent_uni_streams).unwrap());
        transport.datagram_receive_buffer_size(Some(self.max_datagram_size * 100));
        transport.datagram_send_buffer_size(self.max_datagram_size * 100);

        let mut config = quinn::ClientConfig::new(Arc::new(
            quinn::crypto::rustls::QuicClientConfig::try_from(crypto)
                .map_err(|e| crate::error::Error::Tls(e.to_string()))?,
        ));
        config.transport_config(Arc::new(transport));

        Ok(config)
    }

    pub(crate) fn into_quinn_server_config(self) -> crate::error::Result<quinn::ServerConfig> {
        let crypto = match self.crypto {
            CryptoConfig::Server { crypto } => crypto,
            CryptoConfig::Client { .. } => {
                return Err(crate::error::Error::Config(
                    "Client crypto config provided for server".to_string(),
                ))
            }
        };

        let mut transport = quinn::TransportConfig::default();
        transport.max_idle_timeout(Some(self.max_idle_timeout.try_into().unwrap()));
        transport.keep_alive_interval(Some(self.keep_alive_interval));
        transport.max_concurrent_bidi_streams(quinn::VarInt::from_u64(self.max_concurrent_bidi_streams).unwrap());
        transport.max_concurrent_uni_streams(quinn::VarInt::from_u64(self.max_concurrent_uni_streams).unwrap());
        transport.datagram_receive_buffer_size(Some(self.max_datagram_size * 100));
        transport.datagram_send_buffer_size(self.max_datagram_size * 100);

        let mut config = quinn::ServerConfig::with_crypto(Arc::new(
            quinn::crypto::rustls::QuicServerConfig::try_from(crypto)
                .map_err(|e| crate::error::Error::Tls(e.to_string()))?,
        ));
        config.transport_config(Arc::new(transport));

        Ok(config)
    }
}

#[derive(Debug)]
struct SkipServerVerification;

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}

#[derive(Debug)]
struct DummyCertResolver;

impl rustls::server::ResolvesServerCert for DummyCertResolver {
    fn resolve(
        &self,
        _client_hello: rustls::server::ClientHello,
    ) -> Option<Arc<rustls::sign::CertifiedKey>> {
        None
    }
}
