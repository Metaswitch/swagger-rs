//! Utility methods for instantiating common connectors for clients.
use std::path::Path;

use hyper;

/// Returns a function which creates an http-connector. Used for instantiating
/// clients with custom connectors
pub fn http_connector() -> hyper::client::HttpConnector {
    hyper::client::HttpConnector::new(4)
}

/// Returns a function which creates an https-connector which is pinned to a specific
/// CA certificate
///
/// # Arguments
///
/// * `ca_certificate` - Path to CA certificate used to authenticate the server
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
pub fn https_pinned_connector<CA>(
    ca_certificate: CA,
) -> Result<hyper_openssl::HttpsConnector<hyper::client::HttpConnector>, openssl::error::ErrorStack>
where
    CA: AsRef<Path>,
{
    // SSL implementation
    let mut ssl = openssl::ssl::SslConnector::builder(openssl::ssl::SslMethod::tls())?;

    let ca_certificate = ca_certificate.as_ref().to_owned();

    // Server authentication
    ssl.set_ca_file(ca_certificate)?;

    let mut connector = hyper::client::HttpConnector::new(4);
    connector.enforce_http(false);

    hyper_openssl::HttpsConnector::<hyper::client::HttpConnector>::with_connector(connector, ssl)
}

/// Returns a function which creates https-connectors for mutually authenticated connections.
/// # Arguments
///
/// * `ca_certificate` - Path to CA certificate used to authenticate the server
/// * `client_key` - Path to the client private key
/// * `client_certificate` - Path to the client's public certificate associated with the private key
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
pub fn https_mutual_connector<CA, K, C>(
    ca_certificate: CA,
    client_key: K,
    client_certificate: C,
) -> Result<hyper_openssl::HttpsConnector<hyper::client::HttpConnector>, openssl::error::ErrorStack>
where
    CA: AsRef<Path>,
    K: AsRef<Path>,
    C: AsRef<Path>,
{
    // SSL implementation
    let mut ssl = openssl::ssl::SslConnector::builder(openssl::ssl::SslMethod::tls())?;

    // Server authentication
    ssl.set_ca_file(ca_certificate)?;

    // Client authentication
    ssl.set_private_key_file(client_key, openssl::ssl::SslFiletype::PEM)?;
    ssl.set_certificate_chain_file(client_certificate)?;
    ssl.check_private_key()?;

    let mut connector = hyper::client::HttpConnector::new(4);
    connector.enforce_http(false);
    hyper_openssl::HttpsConnector::<hyper::client::HttpConnector>::with_connector(connector, ssl)
}
