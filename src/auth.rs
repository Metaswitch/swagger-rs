//! Authentication and authorization data structures

use std::collections::BTreeSet;
use hyper;
use hyper::{Request, Response, Error};
use super::Context;

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

/// No Authenticator, that does not insert any authorization data, denying all
/// access to endpoints that require authentication.
#[derive(Debug)]
pub struct NoAuthentication<T>(pub T)
where
    T: hyper::server::Service<
        Request = (Request, Context),
        Response = Response,
        Error = Error,
    >;

impl<T> hyper::server::Service for NoAuthentication<T>
where
    T: hyper::server::Service<
        Request = (Request,
                   Context),
        Response = Response,
        Error = Error,
    >,
{
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Future = T::Future;

    fn call(&self, req: Self::Request) -> Self::Future {
        self.0.call((req, Context::default()))
    }
}

/// Dummy Authenticator, that blindly inserts authorization data, allowing all
/// access to an endpoint with the specified subject.
#[derive(Debug)]
pub struct AllowAllAuthenticator<T>
where
    T: hyper::server::Service<Request = (Request, Context), Response = Response, Error = Error>,
{
    inner: T,
    subject: String,
}

impl<T> AllowAllAuthenticator<T>
where
    T: hyper::server::Service<
        Request = (Request, Context),
        Response = Response,
        Error = Error,
    >,
{
    /// Create a middleware that authorizes with the configured subject.
    pub fn new<S: Into<String>>(inner: T, subject: S) -> AllowAllAuthenticator<T> {
        AllowAllAuthenticator {
            inner,
            subject: subject.into(),
        }
    }
}

impl<T> hyper::server::Service for AllowAllAuthenticator<T>
    where T: hyper::server::Service<Request=(Request,Context), Response=Response, Error=Error> {
    type Request = (Request, Option<AuthData>);
    type Response = Response;
    type Error = Error;
    type Future = T::Future;

    fn call(&self, (req, _): Self::Request) -> Self::Future {
        let mut context = Context::default();
        context.authorization = Some(Authorization{
            subject: self.subject.clone(),
            scopes: Scopes::All,
            issuer: None,
        });
        self.inner.call((req, context))
    }
}
