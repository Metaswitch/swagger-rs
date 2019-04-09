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
extern crate chrono;
extern crate futures;
extern crate hyper;
extern crate hyper_old_types;
#[cfg(feature = "multipart")]
extern crate mime;
extern crate uuid;

use std::error;
use std::fmt;

/// Module for encoding API properties in base64.
pub mod base64_format;
pub use base64_format::ByteArray;

/// Module for encoding Nullable properties.
pub mod nullable_format;
pub use nullable_format::Nullable;

pub mod auth;
pub use auth::{AuthData, Authorization};

pub mod context;
pub use context::{ContextBuilder, ContextualPayload, ContextWrapper, EmptyContext, Has, Pop, Push};

/// Module with utilities for creating connectors with hyper.
pub mod connector;
pub use connector::{http_connector, https_connector, https_mutual_connector};

pub mod composites;
pub use composites::{CompositeNewService, CompositeService, NotFound};

pub mod add_context;
pub use add_context::AddContextService;

pub mod drop_context;
pub use drop_context::DropContext;

pub mod request_parser;
pub use request_parser::RequestParser;

mod header;
pub use header::{IntoHeaderValue, XSpanIdString};

#[cfg(feature = "multipart")]
pub mod multipart;

/// Wrapper for hyper::Client so that it implements hyper::Service
struct ClientService<C, B>(hyper::Client<C, B>)
where
    B: hyper::body::Payload + Send + 'static,
    B::Data: Send,
    C: hyper::client::connect::Connect + Sync + 'static,
    C::Transport: 'static,
    C::Future: 'static;

impl<C, B> hyper::service::Service for ClientService<C, B>
where
    B: hyper::body::Payload + Send + 'static,
    B::Data: Send,
    C: hyper::client::connect::Connect + Sync + 'static,
    C::Transport: 'static,
    C::Future: 'static,
{
    type ReqBody = B;
    type ResBody = hyper::Body;
    type Error = hyper::Error;
    type Future = hyper::client::ResponseFuture;

    fn call(&mut self, req: hyper::Request<Self::ReqBody>) -> Self::Future {
        self.0.request(req)
    }
}

/// Helper Bound for Errors for MakeService/Service wrappers
pub trait ErrorBound: Into<Box<std::error::Error + Send + Sync>> {}

impl<T> ErrorBound for T where T: Into<Box<std::error::Error + Send + Sync>> {}

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
