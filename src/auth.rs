//! Authentication and authorization data structures

use std::collections::BTreeSet;
use std::io;
use std::marker::PhantomData;
use hyper;
use hyper::{Request, Response, Error};
use super::Push;

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

impl AuthData {
    /// Set Basic authentication
    pub fn basic(&mut self, username: &str, password: &str) -> Self {
        AuthData::Basic(hyper::header::Basic {
            username: username.to_owned(),
            password: Some(password.to_owned()),
        })
    }

    /// Set Bearer token authentication
    pub fn bearer(&mut self, token: &str) -> Self {
        AuthData::Bearer(hyper::header::Bearer { token: token.to_owned() })
    }

    /// Set ApiKey authentication
    pub fn apikey(&mut self, apikey: &str) -> Self {
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

impl<T, C> hyper::server::NewService for AllowAllAuthenticator<T, C>
    where
        C: Push<Option<Authorization>>,
        T: hyper::server::NewService<Request=(Request, C::Result), Response=Response, Error=Error>,

{
    type Request = (Request, C);
    type Response = Response;
    type Error = Error;
    type Instance = AllowAllAuthenticator<T::Instance, C>;

    fn new_service(&self) -> Result<Self::Instance, io::Error> {
        self.inner.new_service().map(|s| AllowAllAuthenticator::new(s, self.subject.clone()))
    }
}

impl<T, C> hyper::server::Service for AllowAllAuthenticator<T, C>
    where
        C : Push<Option<Authorization>>,
        T: hyper::server::Service<Request=(Request, C::Result), Response=Response, Error=Error>,
{
    type Request = (Request, C);
    type Response = Response;
    type Error = Error;
    type Future = T::Future;

    fn call(&self, (req, context): Self::Request) -> Self::Future {
        let context = context.push(
            Some(Authorization{
                subject: self.subject.clone(),
                scopes: Scopes::All,
                issuer: None,
            }));
        self.inner.call((req, context))
    }
}
