//! Authentication and authorization data structures

use std::collections::BTreeSet;
use chrono::{DateTime, Utc};
use iron;
use hyper;

/// Authorization scopes.
#[derive(Clone, Debug, PartialEq)]
pub enum Scopes {
    /// Some set of scopes.
    Some(BTreeSet<String>),
    /// All possible scopes, authorization checking disabled.
    All,
}

/// Storage of authorization parameters for an incoming request, used for
/// REST API authorization.
#[derive(Clone, Debug, PartialEq)]
pub struct Authorization {
    /// Subject of the request.
    pub subject: String,

    /// Authorization scopes available to the subject.
    pub scopes: Scopes,

    /// The authentication mechanism that provided this authorization data.
    ///
    /// In cases where authentication is delegated to other microservices via
    /// assertion headers, this field stores the original authentication
    /// mechanism that initially authenticated the subject.
    pub auth_type: String,

    /// Issuer of this request.
    ///
    /// When a system is operating on behalf of a subject, the subject field
    /// contains the subject of the request, while the issuer field contains
    /// the system that issued the request.
    pub issuer: Option<String>,

    /// Expiry deadline for this authorization data.
    ///
    /// This is used when the authorization data is cached, used to start a
    /// session, or is used to construct a token passed back to the client.
    ///
    /// A `None` indicates that this authorization data must not be cached, and
    /// is considered only valid for the current request.
    pub expiry_deadline: Option<DateTime<Utc>>,
}
impl iron::typemap::Key for Authorization {
    type Value = Authorization;
}

/// Storage of raw authentication data, used both for storing incoming
/// request authentication, and for authenticating outgoing client requests.
#[derive(Clone, Debug, PartialEq)]
pub enum AuthData {
    /// HTTP Basic auth.
    Basic(hyper::header::Basic),
    /// HTTP Bearer auth, used for OAuth2.
    Bearer(hyper::header::Bearer),
    /// Header-based or query parameter-based API key auth.
    ApiKey(String),
}
impl iron::typemap::Key for AuthData {
    type Value = AuthData;
}

/// Dummy implementation of an Iron middleware to insert authorization data,
/// allowing all access to an endpoint with the subject "alice".
#[derive(Debug)]
pub struct AllowAllMiddleware(String);

impl AllowAllMiddleware {
    /// Create a middleware that authorizes with the configured subject.
    pub fn new<S: Into<String>>(subject: S) -> AllowAllMiddleware {
        AllowAllMiddleware(subject.into())
    }
}

impl iron::middleware::BeforeMiddleware for AllowAllMiddleware {
    fn before(&self, req: &mut iron::Request) -> iron::IronResult<()> {
        req.extensions.insert::<Authorization>(Authorization {
            subject: self.0.clone(),
            scopes: Scopes::All,
            auth_type: "bypass".to_string(),
            issuer: None,
            expiry_deadline: None,
        });
        Ok(())
    }
}
