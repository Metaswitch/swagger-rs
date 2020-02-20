//! Utility methods for instantiating common connectors for clients.
#[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
use std::convert::From as _;
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use std::path::{Path, PathBuf};

use hyper;

/// HTTP Connector construction
#[derive(Debug)]
pub struct Connector;

impl Connector {
    /// Alows building a HTTP(S) connector. Used for instantiating clients with custom
    /// connectors.
    pub fn builder() -> Builder {
        Builder { dns_threads: 4 }
    }
}

/// Builder for HTTP(S) connectors
#[derive(Debug)]
pub struct Builder {
    dns_threads: usize,
}

impl Builder {
    /// Configure the number of threads. Default is 4.
    pub fn dns_threads(mut self, threads: usize) -> Self {
        self.dns_threads = threads;
        self
    }

    /// Use HTTPS instead of HTTP
    pub fn https(self) -> HttpsBuilder {
        HttpsBuilder {
            dns_threads: self.dns_threads,
            #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
            server_cert: None,
            #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
            client_cert: None,
        }
    }

    /// Build a HTTP connector
    pub fn build(self) -> hyper::client::HttpConnector {
        hyper::client::HttpConnector::new(self.dns_threads)
    }
}

/// Builder for HTTPS connectors
#[derive(Debug)]
pub struct HttpsBuilder {
    dns_threads: usize,
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    server_cert: Option<PathBuf>,
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    client_cert: Option<(PathBuf, PathBuf)>,
}

impl HttpsBuilder {
    /// Configure the number of threads. Default is 4.
    pub fn dns_threads(mut self, threads: usize) -> Self {
        self.dns_threads = threads;
        self
    }

    /// Pin the CA certificate for the server's certificate.
    ///
    /// # Arguments
    ///
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn pin_server_certificate<CA>(mut self, ca_certificate: CA) -> Self
    where
        CA: AsRef<Path>,
    {
        self.server_cert = Some(ca_certificate.as_ref().to_owned());
        self
    }

    /// Provide the Client Certificate and Key for the connection for Mutual TLS
    ///
    /// # Arguments
    ///
    /// * `client_key` - Path to the client private key
    /// * `client_certificate` - Path to the client's public certificate associated with the private key
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn client_authentication<K, C>(mut self, client_key: K, client_certificate: C) -> Self
    where
        K: AsRef<Path>,
        C: AsRef<Path>,
    {
        self.client_cert = Some((
            client_key.as_ref().to_owned(),
            client_certificate.as_ref().to_owned(),
        ));
        self
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    /// Build the HTTPS connector. Will fail if the provided certificates/keys can't be loaded
    /// or the SSL connector can't be created
    pub fn build(
        self,
    ) -> Result<
        hyper_openssl::HttpsConnector<hyper::client::HttpConnector>,
        openssl::error::ErrorStack,
    > {
        // SSL implementation
        let mut ssl = openssl::ssl::SslConnector::builder(openssl::ssl::SslMethod::tls())?;

        if let Some(ca_certificate) = self.server_cert {
            // Server authentication
            ssl.set_ca_file(ca_certificate)?;
        }

        if let Some((client_key, client_certificate)) = self.client_cert {
            // Client authentication
            ssl.set_private_key_file(client_key, openssl::ssl::SslFiletype::PEM)?;
            ssl.set_certificate_chain_file(client_certificate)?;
            ssl.check_private_key()?;
        }

        let mut connector = hyper::client::HttpConnector::new(self.dns_threads);
        connector.enforce_http(false);
        hyper_openssl::HttpsConnector::<hyper::client::HttpConnector>::with_connector(
            connector, ssl,
        )
    }

    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
    /// Build the HTTPS connector. Will fail if the SSL connector can't be created.
    pub fn build(
        self,
    ) -> Result<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, native_tls::Error> {
        let tls = native_tls::TlsConnector::new()?.into();
        let mut connector = hyper::client::HttpConnector::new(self.dns_threads);
        connector.enforce_http(false);
        let mut connector = hyper_tls::HttpsConnector::from((connector, tls));
        connector.https_only(true);
        Ok(connector)
    }
}
