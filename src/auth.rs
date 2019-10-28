//! Authentication and authorization data structures

use crate::context::ContextualPayload;
use crate::{ErrorBound, Push};
use futures::FutureExt;
use hyper;
use hyper::body::Payload;
use hyper::header::AUTHORIZATION;
use hyper::service::Service;
use hyper::{HeaderMap, Request, Response};
pub use hyper_old_types::header::Authorization as Header;
use hyper_old_types::header::Header as HeaderTrait;
pub use hyper_old_types::header::{Basic, Bearer};
use hyper_old_types::header::{Raw, Scheme};
use std::collections::BTreeSet;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::string::ToString;
use std::task::{Context, Poll};

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

impl<'a, T, SC, RC, E, ME, S, OB> Service<&'a SC> for MakeAllowAllAuthenticator<T, RC>
where
    RC: RcBound,
    RC::Result: Send + 'static,
    T: Service<
        &'a SC,
        Error = ME,
        Response = S,
        Future = Pin<Box<dyn Future<Output=Result<S, ME>>>>,
    >,
    S: Service<ContextualPayload<hyper::Body, RC::Result>, Error = E, Response = OB>
        + 'static,
    ME: ErrorBound,
    E: ErrorBound,
    S::Future: Send,
    OB: Payload,
{
    type Response = AllowAllAuthenticator<T, RC>;
    type Error = ME;
    type Future = Pin<Box<dyn Future<Output = Result<AllowAllAuthenticator<T, RC>, ME>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, service_ctx: &'a SC) -> Self::Future {
        let subject = self.subject.clone();
        self.inner
            .call(service_ctx)
            .map(|s| AllowAllAuthenticator::new(s, subject))
            .boxed()
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

impl<T, RC> hyper::service::Service<ContextualPayload<hyper::Body, RC>> for AllowAllAuthenticator<T, RC>
where
    RC: RcBound,
    RC::Result: Send + 'static,
    T: Service<ContextualPayload<hyper::Body, RC::Result>>,
    T::Future: Future<Output = Result<Response<T::Response>, T::Error>> + Send + 'static,
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<T::Response>, T::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<ContextualPayload<hyper::Body, RC>>) -> Self::Future {
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
    use hyper::server::conn::AddrStream;
    use hyper::service::{make_service_fn, Service};
    use hyper::{body::Payload, Body, Response};
    use std::fmt::Debug;
    use std::io;

    fn check_inner_type<'a, T, SC: 'a, E, ME, S, F, IB, OB>(_: &T)
    where
        T: Service<
            &'a SC,
            Error = ME,
            Future = F,
            Response = S,
        >,
        E: ErrorBound,
        ME: ErrorBound,
        S: Service<IB, Response = OB, Error = E>,
        F: Future<Output = Result<S, ME>>,
        IB: Payload,
        OB: Payload,
    {
        // This function is here merely to force a type check against the given bounds.
    }

    fn check_type<S, A, B>(_: &S)
    where
        S: Service<A, Response = B>,
        S::Error: ErrorBound,
        B: Payload,
    {
        // This function is here merely to force a type check against the given bounds.
    }

    struct TestService<IB>(std::net::SocketAddr, PhantomData<IB>);

    impl<IB: Debug + Payload> Service<IB> for TestService<IB> {
        type Response = Body;
        type Error = std::io::Error;
        type Future = Pin<Box<dyn Future<Output=Result<Response<Self::Response>, Self::Error>>>>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: Request<IB>) -> Self::Future {
            futures::future::ok(
                Response::new(Body::from(format!("{:?} {}", req, self.0)))
            ).boxed()
        }
    }

    #[test]
    fn test_make_service() {
        let make_svc = make_service_fn(|socket: &AddrStream| {
            futures::future::ok(Ok(TestService(socket.remote_addr(), PhantomData)));
        });

        check_inner_type(&make_svc);

        let a: MakeAllowAllAuthenticator<_, EmptyContext> =
            MakeAllowAllAuthenticator::new(make_svc, "foo");

        check_inner_type(&a);
        check_type(&a);
    }
}
