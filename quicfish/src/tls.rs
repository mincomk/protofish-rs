use std::io::BufRead;

use rustls::{
    RootCertStore,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};

pub fn create_client_config(
    pem_reeader: &mut impl BufRead,
) -> crate::error::Result<rustls::ClientConfig> {
    let mut root_cert_store = RootCertStore::empty();

    let certs = rustls_pemfile::certs(pem_reeader);
    for cert in certs {
        root_cert_store.add(cert?).map_err(|e| {
            crate::error::Error::Tls(format!("Failed to add root certificate: {}", e))
        })?;
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    Ok(config)
}

pub fn create_server_config(
    pem_cert_reader: &mut impl BufRead,
    pem_key_reader: &mut impl BufRead,
) -> crate::error::Result<rustls::ServerConfig> {
    let certs = rustls_pemfile::certs(pem_cert_reader)
        .collect::<Result<Vec<CertificateDer<'static>>, _>>()?;

    let private_key = PrivateKeyDer::from_pem_reader(pem_key_reader)
        .map_err(|e| crate::error::Error::Tls(format!("Failed to parse private key: {}", e)))?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)
        .map_err(|e| crate::error::Error::Tls(format!("Failed to create server config: {}", e)))?;

    Ok(config)
}
