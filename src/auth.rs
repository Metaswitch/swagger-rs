//! Authentication and authorization data structures

use std::collections::BTreeSet;
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
    /// Subject for which authorization is granted
    /// (i.e., what may be accessed.)
    pub subject: String,

    /// Scopes for which authorization is granted
    /// (i.e., what types of access are permitted).
    pub scopes: Scopes,

    /// Identity of the party to whom authorization was granted, if available
    /// (i.e., who is responsible for the access).
    ///
    /// In an OAuth environment, this is the identity of the client which
    /// issued an authorization request to the resource owner (end-user),
    /// and which has been directly authorized by the resource owner
    /// to access the protected resource. If the client delegates that
    /// authorization to another service (e.g., a proxy or other delegate),
    /// the `issuer` is still the original client which was authorized by
    /// the resource owner.
    pub issuer: Option<String>,
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
            issuer: None,
        });
        Ok(())
    }
}
