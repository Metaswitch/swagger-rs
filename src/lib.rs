//! Support crate for Swagger codegen.

#![warn(missing_docs, missing_debug_implementations)]
#![deny(unused_extern_crates)]

#[cfg(feature = "serdejson")]
extern crate serde;
#[cfg(feature = "serdejson")]
extern crate serde_json;
#[cfg(feature = "serdejson")]
#[cfg(test)]
#[macro_use]
extern crate serde_derive;
extern crate base64;

#[macro_use]
extern crate hyper;
extern crate iron;

use std::fmt;
use std::error;

/// Module for encoding API properties in base64.
pub mod base64_format;
pub use base64_format::ByteArray;

/// Module for encoding Nullable properties.
pub mod nullable_format;
pub use nullable_format::Nullable;

pub mod auth;
pub use auth::{Authorization, AuthData};

/// Request context, both as received in a server handler or as sent in a
/// client request. When REST microservices are chained, the Context passes
/// data from the server API to any further HTTP requests.
#[derive(Clone, Debug, Default)]
pub struct Context {
    /// Tracking ID when passing a request to another microservice.
    pub x_span_id: Option<String>,

    /// Authorization data, filled in from middlewares.
    pub authorization: Option<Authorization>,
    /// Raw authentication data, for use in making HTTP requests as a client.
    pub auth_data: Option<AuthData>,
}

impl Context {
    /// Create a new, empty, `Context`.
    pub fn new() -> Context {
        Context::default()
    }

    /// Create a `Context` with a given span ID.
    pub fn new_with_span_id<S: Into<String>>(x_span_id: S) -> Context {
        Context {
            x_span_id: Some(x_span_id.into()),
            ..Context::default()
        }
    }

    /// Set Basic authentication
    pub fn auth_basic(&mut self, username: &str, password: &str) {
        self.auth_data = Some(AuthData::Basic(hyper::header::Basic {
            username: username.to_owned(),
            password: Some(password.to_owned()),
        }));
    }

    /// Set Bearer token authentication
    pub fn auth_bearer(&mut self, token: &str) {
        self.auth_data = Some(AuthData::Bearer(
            hyper::header::Bearer { token: token.to_owned() },
        ));
    }

    /// Set ApiKey authentication
    pub fn auth_apikey(&mut self, apikey: &str) {
        self.auth_data = Some(AuthData::ApiKey(apikey.to_owned()));
    }
}

header! {
    /// `X-Span-ID` header, used to track a request through a chain of microservices.
    (XSpanId, "X-Span-ID") => [String]
}

/// Very simple error type - just holds a description of the error. This is useful for human
/// diagnosis and troubleshooting, but not for applications to parse. The justification for this
/// is to deny applications visibility into the communication layer, forcing the application code
/// to act solely on the logical responses that the API provides, promoting abstraction in the
/// application code.
#[derive(Clone, Debug)]
pub struct ApiError(pub String);

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let debug: &fmt::Debug = self;
        debug.fmt(f)
    }
}

impl error::Error for ApiError {
    fn description(&self) -> &str {
        "Failed to produce a valid response."
    }
}

impl<'a> From<&'a str> for ApiError {
    fn from(e: &str) -> Self {
        ApiError(e.to_string())
    }
}

impl From<String> for ApiError {
    fn from(e: String) -> Self {
        ApiError(e)
    }
}

#[cfg(feature = "serdejson")]
impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        ApiError(format!("Response body did not match the schema: {}", e))
    }
}
