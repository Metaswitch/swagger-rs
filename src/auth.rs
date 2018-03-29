//! Authentication and authorization data structures

use std::collections::BTreeSet;
use std::io;
use std::marker::PhantomData;
use hyper;
use hyper::{Request, Response, Error};
use super::{Has, ExtendsWith};

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
pub struct NoAuthentication<T, C>
where
    C: Default,
{
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> hyper::server::NewService for NoAuthentication<T, C>
    where
        T: hyper::server::NewService<Request=(Request, C), Response=Response, Error=Error>,
        C: Default,
{
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Instance = NoAuthentication<T::Instance, C>;

    fn new_service(&self) -> Result<Self::Instance, io::Error> {
        self.inner.new_service().map(|s| NoAuthentication{inner: s, marker: PhantomData})
    }
}

impl<T, C> hyper::server::Service for NoAuthentication<T, C>
    where
        T: hyper::server::Service<Request=(Request, C), Response=Response, Error=Error>,
        C: Default,
{
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Future = T::Future;

    fn call(&self, req: Self::Request) -> Self::Future {
        self.inner.call((req, C::default()))
    }
}

/// Dummy Authenticator, that blindly inserts authorization data, allowing all
/// access to an endpoint with the specified subject.
#[derive(Debug)]
pub struct AllowAllAuthenticator<T, C, D> {
    inner: T,
    subject: String,
    marker1: PhantomData<C>,
    marker2: PhantomData<D>,
}

impl<T, C, D> AllowAllAuthenticator<T, C, D> {
    /// Create a middleware that authorizes with the configured subject.
    pub fn new<U: Into<String>>(inner: T, subject: U) -> AllowAllAuthenticator<T, C, D> {
        AllowAllAuthenticator {
            inner,
            subject: subject.into(),
            marker1: PhantomData,
            marker2: PhantomData,
        }
    }
}

impl<T, C, D> hyper::server::NewService for AllowAllAuthenticator<T, C, D>
    where
        T: hyper::server::NewService<Request=(Request, D), Response=Response, Error=Error>,
        C: Has<Option<AuthData>>,
        D: ExtendsWith<Extends=C, Extension=Option<Authorization>>,
{
    type Request = (Request, C);
    type Response = Response;
    type Error = Error;
    type Instance = AllowAllAuthenticator<T::Instance, C, D>;

    fn new_service(&self) -> Result<Self::Instance, io::Error> {
        self.inner.new_service().map(|s| AllowAllAuthenticator::new(s, self.subject.clone()))
    }
}

impl<T, C, D> hyper::server::Service for AllowAllAuthenticator<T, C, D>
    where
        T: hyper::server::Service<Request=(Request,D), Response=Response, Error=Error>,
        C: Has<Option<AuthData>>,
        D: ExtendsWith<Extends=C, Extension=Option<Authorization>>,
{
    type Request = (Request, C);
    type Response = Response;
    type Error = Error;
    type Future = T::Future;

    fn call(&self, (req, context): Self::Request) -> Self::Future {
        let context = D::new(
            context,
            Some(Authorization{
                subject: self.subject.clone(),
                scopes: Scopes::All,
                issuer: None,
            })
        );
        self.inner.call((req, context))
    }
}
