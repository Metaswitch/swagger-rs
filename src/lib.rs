//! Support crate for Swagger codegen.
#![deny(missing_docs, missing_debug_implementations, unused_extern_crates)]

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
pub use context::{
    ContextBuilder, ContextWrapper, ContextualPayload, EmptyContext, Has, Pop, Push,
};

/// Module to support client middleware
pub mod client;

/// Module with utilities for creating connectors with hyper.
pub mod connector;
pub use connector::Connector;

pub mod composites;
pub use composites::{CompositeMakeService, CompositeService, NotFound};

pub mod add_context;
pub use add_context::{AddContextMakeService, AddContextService};

pub mod drop_context;
pub use drop_context::{DropContextMakeService, DropContextService};

pub mod request_parser;
pub use request_parser::RequestParser;

mod header;
pub use header::{IntoHeaderValue, XSpanIdString, X_SPAN_ID};

#[cfg(feature = "multipart")]
pub mod multipart;

pub mod one_any_of;
pub use one_any_of::*;

/// Helper Bound for Errors for MakeService/Service wrappers
pub trait ErrorBound: Into<Box<dyn std::error::Error + Send + Sync>> {}

impl<T> ErrorBound for T where T: Into<Box<dyn std::error::Error + Send + Sync>> {}

/// Very simple error type - just holds a description of the error. This is useful for human
/// diagnosis and troubleshooting, but not for applications to parse. The justification for this
/// is to deny applications visibility into the communication layer, forcing the application code
/// to act solely on the logical responses that the API provides, promoting abstraction in the
/// application code.
#[derive(Clone, Debug)]
pub struct ApiError(pub String);

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let debug: &dyn fmt::Debug = self;
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
