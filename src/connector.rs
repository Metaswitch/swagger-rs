//! Utility methods for instantiating common connectors for clients.
use std::path::Path;

use hyper;

/// Returns a function which creates an http-connector. Used for instantiating
/// clients with custom connectors
pub fn http_connector() -> Box<dyn Fn() -> hyper::client::HttpConnector + Send + Sync> {
    Box::new(move || hyper::client::HttpConnector::new(4))
}

/// Returns a function which creates an https-connector
///
/// # Arguments
///
/// * `ca_certificate` - Path to CA certificate used to authenticate the server
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
pub fn https_connector<CA>(
    ca_certificate: CA,
) -> Box<dyn Fn() -> hyper_tls::HttpsConnector<hyper::client::HttpConnector> + Send + Sync>
where
    CA: AsRef<Path>,
{
    let ca_certificate = ca_certificate.as_ref().to_owned();
    Box::new(move || {
        // SSL implementation
        let mut ssl =
            openssl::ssl::SslConnectorBuilder::new(openssl::ssl::SslMethod::tls()).unwrap();

        // Server authentication
        ssl.set_ca_file(ca_certificate.clone()).unwrap();

        let builder: native_tls::TlsConnectorBuilder =
            native_tls::backend::openssl::TlsConnectorBuilderExt::from_openssl(ssl);
        let mut connector = hyper::client::HttpConnector::new(4);
        connector.enforce_http(false);
        let connector: hyper_tls::HttpsConnector<hyper::client::HttpConnector> =
            (connector, builder.build().unwrap()).into();
        connector
    })
}

/// Not currently implemented on Mac OS X, iOS and Windows.
/// This function will panic when called.
#[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
pub fn https_connector<CA>(
    _ca_certificate: CA,
) -> Box<dyn Fn() -> hyper_tls::HttpsConnector<hyper::client::HttpConnector> + Send + Sync>
where
    CA: AsRef<Path>,
{
    unimplemented!("See issue #43 (https://github.com/Metaswitch/swagger-rs/issues/43)")
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
) -> Box<dyn Fn() -> hyper_tls::HttpsConnector<hyper::client::HttpConnector> + Send + Sync>
where
    CA: AsRef<Path>,
    K: AsRef<Path>,
    C: AsRef<Path>,
{
    let ca_certificate = ca_certificate.as_ref().to_owned();
    let client_key = client_key.as_ref().to_owned();
    let client_certificate = client_certificate.as_ref().to_owned();
    Box::new(move || {
        // SSL implementation
        let mut ssl =
            openssl::ssl::SslConnectorBuilder::new(openssl::ssl::SslMethod::tls()).unwrap();

        // Server authentication
        ssl.set_ca_file(ca_certificate.clone()).unwrap();

        // Client authentication
        ssl.set_private_key_file(client_key.clone(), openssl::x509::X509_FILETYPE_PEM)
            .unwrap();
        ssl.set_certificate_chain_file(client_certificate.clone())
            .unwrap();
        ssl.check_private_key().unwrap();

        let builder: native_tls::TlsConnectorBuilder =
            native_tls::backend::openssl::TlsConnectorBuilderExt::from_openssl(ssl);
        let mut connector = hyper::client::HttpConnector::new(4);
        connector.enforce_http(false);
        let connector: hyper_tls::HttpsConnector<hyper::client::HttpConnector> =
            (connector, builder.build().unwrap()).into();
        connector
    })
}

/// Not currently implemented on Mac OS X, iOS and Windows.
/// This function will panic when called.
#[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
pub fn https_mutual_connector<CA, K, C>(
    _ca_certificate: CA,
    _client_key: K,
    _client_certificate: C,
) -> Box<dyn Fn() -> hyper_tls::HttpsConnector<hyper::client::HttpConnector> + Send + Sync>
where
    CA: AsRef<Path>,
    K: AsRef<Path>,
    C: AsRef<Path>,
{
    unimplemented!("See issue #43 (https://github.com/Metaswitch/swagger-rs/issues/43)")
}
