//! Authentication and authorization data structures

use crate::context::ContextualPayload;
use crate::{ErrorBound, Push};
use futures::future::Future;
use hyper;
use hyper::body::Payload;
use hyper::header::AUTHORIZATION;
use hyper::service::{MakeService, Service};
use hyper::{HeaderMap, Request, Response};
pub use hyper_old_types::header::Authorization as Header;
use hyper_old_types::header::Header as HeaderTrait;
pub use hyper_old_types::header::{Basic, Bearer};
use hyper_old_types::header::{Raw, Scheme};
use std::collections::BTreeSet;
use std::marker::PhantomData;
use std::string::ToString;

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

/// Bound for Request Context for MakeService wrappers
pub trait RcBound: Push<Option<Authorization>> + Send + 'static {}

impl<T> RcBound for T where T: Push<Option<Authorization>> + Send + 'static {}

/// Dummy Authenticator, that blindly inserts authorization data, allowing all
/// access to an endpoint with the specified subject.
#[derive(Debug)]
pub struct MakeAllowAllAuthenticator<T, RC>
where
    RC: RcBound,
    RC::Result: Send + 'static,
{
    inner: T,
    subject: String,
    marker: PhantomData<RC>,
}

impl<T, RC> MakeAllowAllAuthenticator<T, RC>
where
    RC: RcBound,
    RC::Result: Send + 'static,
{
    /// Create a middleware that authorizes with the configured subject.
    pub fn new<U: Into<String>>(inner: T, subject: U) -> Self {
        MakeAllowAllAuthenticator {
            inner,
            subject: subject.into(),
            marker: PhantomData,
        }
    }
}

impl<'a, T, SC, RC, E, ME, S, OB, F> MakeService<&'a SC> for MakeAllowAllAuthenticator<T, RC>
where
    RC: RcBound,
    RC::Result: Send + 'static,
    T: MakeService<
        &'a SC,
        Error = E,
        MakeError = ME,
        Service = S,
        ReqBody = ContextualPayload<hyper::Body, RC::Result>,
        ResBody = OB,
        Future = F,
    >,
    S: Service<Error = E, ReqBody = ContextualPayload<hyper::Body, RC::Result>, ResBody = OB>
        + 'static,
    ME: ErrorBound,
    E: ErrorBound,
    F: Future<Item = S, Error = ME> + Send + 'static,
    S::Future: Send,
    OB: Payload,
{
    type ReqBody = ContextualPayload<hyper::Body, RC>;
    type ResBody = OB;
    type Error = E;
    type MakeError = ME;
    type Service = AllowAllAuthenticator<S, RC>;
    type Future = Box<dyn Future<Item = Self::Service, Error = ME> + Send>;

    fn make_service(&mut self, service_ctx: &'a SC) -> Self::Future {
        let subject = self.subject.clone();
        Box::new(
            self.inner
                .make_service(service_ctx)
                .map(|s| AllowAllAuthenticator::new(s, subject)),
        )
    }
}

/// Dummy Authenticator, that blindly inserts authorization data, allowing all
/// access to an endpoint with the specified subject.
#[derive(Debug)]
pub struct AllowAllAuthenticator<T, RC> {
    inner: T,
    subject: String,
    marker: PhantomData<RC>,
}

impl<T, RC> AllowAllAuthenticator<T, RC> {
    /// Create a middleware that authorizes with the configured subject.
    pub fn new<U: Into<String>>(inner: T, subject: U) -> Self {
        AllowAllAuthenticator {
            inner,
            subject: subject.into(),
            marker: PhantomData,
        }
    }
}

impl<T, RC> Service for AllowAllAuthenticator<T, RC>
where
    RC: RcBound,
    RC::Result: Send + 'static,
    T: Service<ReqBody = ContextualPayload<hyper::Body, RC::Result>>,
    T::Future: Future<Item = Response<T::ResBody>, Error = T::Error> + Send + 'static,
{
    type ReqBody = ContextualPayload<hyper::Body, RC>;
    type ResBody = T::ResBody;
    type Error = T::Error;
    type Future = Box<dyn Future<Item = Response<T::ResBody>, Error = T::Error> + Send>;

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

        Box::new(self.inner.call(Request::from_parts(head, body)))
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
pub fn api_key_from_header(headers: &HeaderMap, header: &str) -> Option<String> {
    headers
        .get(header)
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EmptyContext;
    use futures::future;
    use hyper::server::conn::AddrStream;
    use hyper::service::{make_service_fn, MakeService, MakeServiceRef, Service};
    use hyper::{body::Payload, Body, Response};
    use std::fmt::Debug;
    use std::io;

    fn check_inner_type<'a, T, SC: 'a, E, ME, S, F, IB, OB>(_: &T)
    where
        T: MakeService<
            &'a SC,
            Error = E,
            MakeError = ME,
            Service = S,
            Future = F,
            ReqBody = IB,
            ResBody = OB,
        >,
        E: ErrorBound,
        ME: ErrorBound,
        S: Service<ReqBody = IB, ResBody = OB, Error = E>,
        F: Future<Item = S, Error = ME>,
        IB: Payload,
        OB: Payload,
    {
        // This function is here merely to force a type check against the given bounds.
    }

    fn check_type<S, A, B, C>(_: &S)
    where
        S: MakeServiceRef<A, ReqBody = B, ResBody = C>,
        S::Error: ErrorBound,
        S::Service: 'static,
        B: Payload,
        C: Payload,
    {
        // This function is here merely to force a type check against the given bounds.
    }

    struct TestService<IB>(std::net::SocketAddr, PhantomData<IB>);

    impl<IB: Debug + Payload> Service for TestService<IB> {
        type ReqBody = IB;
        type ResBody = Body;
        type Error = std::io::Error;
        type Future = future::FutureResult<Response<Self::ResBody>, Self::Error>;

        fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
            future::ok(Response::new(Body::from(format!("{:?} {}", req, self.0))))
        }
    }

    #[test]
    fn test_make_service() {
        let make_svc = make_service_fn(|socket: &AddrStream| {
            let f: future::FutureResult<TestService<_>, io::Error> =
                future::ok(TestService(socket.remote_addr(), PhantomData));
            f
        });

        check_inner_type(&make_svc);

        let a: MakeAllowAllAuthenticator<_, EmptyContext> =
            MakeAllowAllAuthenticator::new(make_svc, "foo");

        check_inner_type(&a);
        check_type(&a);
    }
}
