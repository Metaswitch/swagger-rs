//! Authentication and authorization data structures

use super::Push;
use crate::context::ContextualPayload;
use futures::future::Future;
use hyper;
use hyper::header::AUTHORIZATION;
use hyper::{Error, HeaderMap, Request};
pub use hyper_old_types::header::Authorization as Header;
use hyper_old_types::header::Header as HeaderTrait;
pub use hyper_old_types::header::{Basic, Bearer};
use hyper_old_types::header::{Raw, Scheme};
use std::collections::BTreeSet;
use std::io;
use std::marker::PhantomData;

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
    Basic(Basic),
    /// HTTP Bearer auth, used for OAuth2.
    Bearer(Bearer),
    /// Header-based or query parameter-based API key auth.
    ApiKey(String),
}

impl AuthData {
    /// Set Basic authentication
    pub fn basic(username: &str, password: &str) -> Self {
        AuthData::Basic(Basic {
            username: username.to_owned(),
            password: Some(password.to_owned()),
        })
    }

    /// Set Bearer token authentication
    pub fn bearer(token: &str) -> Self {
        AuthData::Bearer(Bearer {
            token: token.to_owned(),
        })
    }

    /// Set ApiKey authentication
    pub fn apikey(apikey: &str) -> Self {
        AuthData::ApiKey(apikey.to_owned())
    }
}

/// Dummy Authenticator, that blindly inserts authorization data, allowing all
/// access to an endpoint with the specified subject.
#[derive(Debug)]
pub struct AllowAllAuthenticator<T, C>
where
    C: Push<Option<Authorization>>,
{
    inner: T,
    subject: String,
    marker: PhantomData<C>,
}

impl<T, C> AllowAllAuthenticator<T, C>
where
    C: Push<Option<Authorization>>,
{
    /// Create a middleware that authorizes with the configured subject.
    pub fn new<U: Into<String>>(inner: T, subject: U) -> AllowAllAuthenticator<T, C> {
        AllowAllAuthenticator {
            inner,
            subject: subject.into(),
            marker: PhantomData,
        }
    }
}

impl<T, C> hyper::service::MakeService<C> for AllowAllAuthenticator<T, C>
where
    C: Push<Option<Authorization>> + Send + 'static,
    C::Result: Send + 'static,
    T: hyper::service::MakeService<
        C,
        ReqBody = ContextualPayload<hyper::Body, C::Result>,
        ResBody = hyper::Body,
        Error = hyper::Error,
        MakeError = io::Error,
    >,
    T::Future: 'static,
{
    type ReqBody = ContextualPayload<hyper::Body, C>;
    type ResBody = T::ResBody;
    type Error = hyper::Error;
    type MakeError = io::Error;
    type Future = Box<Future<Item = Self::Service, Error = io::Error>>;
    type Service = AllowAllAuthenticator<T::Service, C>;

    fn make_service(&mut self, service_ctx: C) -> Self::Future {
        let subject = self.subject.clone();
        Box::new(
            self.inner
                .make_service(service_ctx)
                .map(|s| AllowAllAuthenticator::new(s, subject)),
        )
    }
}

impl<T, C> hyper::service::Service for AllowAllAuthenticator<T, C>
where
    C: Push<Option<Authorization>> + Send + 'static,
    C::Result: Send + 'static,
    T: hyper::service::Service<
        ReqBody = ContextualPayload<hyper::Body, C::Result>,
        ResBody = hyper::Body,
        Error = Error,
    >,
{
    type ReqBody = ContextualPayload<hyper::Body, C>;
    type ResBody = T::ResBody;
    type Error = Error;
    type Future = T::Future;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let (head, body) = req.into_parts();
        let body = ContextualPayload {
            inner: body.inner,
            context: body.context.push(Some(Authorization {
                subject: self.subject.clone(),
                scopes: Scopes::All,
                issuer: None,
            })),
        };

        self.inner.call(Request::from_parts(head, body))
    }
}

/// Retrieve an authorization scheme data from a set of headers
pub fn from_headers<S: Scheme>(headers: &HeaderMap) -> Option<S>
where
    S: std::str::FromStr + 'static,
    S::Err: 'static,
{
    headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Header::<S>::parse_header(&Raw::from(s)).ok())
        .map(|a| a.0)
}

/// Retrieve an API key from a header
pub fn api_key_from_header(headers: &HeaderMap, header: &'static str) -> Option<String> {
    headers
        .get(header)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}
