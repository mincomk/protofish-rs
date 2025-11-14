use std::net::SocketAddr;

use crate::config::QuicConfig;
use crate::error::{Error, Result};

pub struct QuicEndpoint {
    endpoint: quinn::Endpoint,
    is_server: bool,
}

impl QuicEndpoint {
    pub fn client(bind_addr: SocketAddr, config: QuicConfig) -> Result<Self> {
        let client_config = config.into_quinn_client_config()?;
        let mut endpoint = quinn::Endpoint::client(bind_addr)
            .map_err(|e| Error::Config(e.to_string()))?;
        endpoint.set_default_client_config(client_config);

        Ok(Self {
            endpoint,
            is_server: false,
        })
    }

    pub fn server(bind_addr: SocketAddr, config: QuicConfig) -> Result<Self> {
        let server_config = config.into_quinn_server_config()?;
        let endpoint = quinn::Endpoint::server(server_config, bind_addr)
            .map_err(|e| Error::Config(e.to_string()))?;

        Ok(Self {
            endpoint,
            is_server: true,
        })
    }

    pub async fn connect(&self, server_addr: SocketAddr, server_name: &str) -> Result<quinn::Connection> {
        if self.is_server {
            return Err(Error::Config("Cannot connect from server endpoint".to_string()));
        }

        let connection = self
            .endpoint
            .connect(server_addr, server_name)
            .map_err(|e| Error::Config(e.to_string()))?
            .await?;

        Ok(connection)
    }

    pub async fn accept(&self) -> Option<quinn::Connection> {
        if !self.is_server {
            return None;
        }

        let incoming = self.endpoint.accept().await?;
        incoming.await.ok()
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.endpoint
            .local_addr()
            .map_err(|e| Error::Config(e.to_string()))
    }

    pub fn close(&self) {
        self.endpoint.close(0u32.into(), b"endpoint closed");
    }
}

pub struct QuicEndpointBuilder {
    config: QuicConfig,
    bind_addr: SocketAddr,
    role: EndpointRole,
}

enum EndpointRole {
    Client,
    Server,
}

impl QuicEndpointBuilder {
    pub fn new_client(bind_addr: SocketAddr) -> Self {
        Self {
            config: QuicConfig::client_default(),
            bind_addr,
            role: EndpointRole::Client,
        }
    }

    pub fn new_server(bind_addr: SocketAddr) -> Self {
        Self {
            config: QuicConfig::server_default(),
            bind_addr,
            role: EndpointRole::Server,
        }
    }

    pub fn with_config(mut self, config: QuicConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(self) -> Result<QuicEndpoint> {
        match self.role {
            EndpointRole::Client => QuicEndpoint::client(self.bind_addr, self.config),
            EndpointRole::Server => QuicEndpoint::server(self.bind_addr, self.config),
        }
    }
}
