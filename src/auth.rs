//! Authentication and authorization data structures

use crate::context::Push;
use futures::future::FutureExt;
use headers::authorization::{Basic, Bearer, Credentials};
use headers::Authorization as Header;
use hyper::header::AUTHORIZATION;
use hyper::service::Service;
use hyper::{HeaderMap, Request};
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
    /// HTTP Basic auth - username and password.
    Basic(String, String),
    /// HTTP Bearer auth, used for OAuth2 - token.
    Bearer(String),
    /// Header-based or query parameter-based API key auth.
    ApiKey(String),
}

impl AuthData {
    /// Set Basic authentication
    pub fn basic(username: &str, password: &str) -> Self {
        AuthData::Basic(username.to_owned(), password.to_owned())
    }

    /// Set Bearer token authentication.  Returns None if the token was invalid.
    pub fn bearer(token: &str) -> Option<Self> {
        Some(AuthData::Bearer(
            Header::bearer(token).ok()?.token().to_owned(),
        ))
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

impl<Inner, RC, Target> Service<Target> for MakeAllowAllAuthenticator<Inner, RC>
where
    RC: RcBound,
    RC::Result: Send + 'static,
    Inner: Service<Target>,
    Inner::Future: Send + 'static,
{
    type Error = Inner::Error;
    type Response = AllowAllAuthenticator<Inner::Response, RC>;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, target: Target) -> Self::Future {
        let subject = self.subject.clone();
        Box::pin(
            self.inner
                .call(target)
                .map(|s| Ok(AllowAllAuthenticator::new(s?, subject))),
        )
    }
}

/// Dummy Authenticator, that blindly inserts authorization data, allowing all
/// access to an endpoint with the specified subject.
#[derive(Debug)]
pub struct AllowAllAuthenticator<T, RC>
where
    RC: RcBound,
    RC::Result: Send + 'static,
{
    inner: T,
    subject: String,
    marker: PhantomData<RC>,
}

impl<T, RC> AllowAllAuthenticator<T, RC>
where
    RC: RcBound,
    RC::Result: Send + 'static,
{
    /// Create a middleware that authorizes with the configured subject.
    pub fn new<U: Into<String>>(inner: T, subject: U) -> Self {
        AllowAllAuthenticator {
            inner,
            subject: subject.into(),
            marker: PhantomData,
        }
    }
}

impl<T, RC> Clone for AllowAllAuthenticator<T, RC>
where
    T: Clone,
    RC: RcBound,
    RC::Result: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            subject: self.subject.clone(),
            marker: PhantomData,
        }
    }
}

impl<T, B, RC> Service<(Request<B>, RC)> for AllowAllAuthenticator<T, RC>
where
    RC: RcBound,
    RC::Result: Send + 'static,
    T: Service<(Request<B>, RC::Result)>,
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = T::Future;

    fn call(&self, req: (Request<B>, RC)) -> Self::Future {
        let (request, context) = req;
        let context = context.push(Some(Authorization {
            subject: self.subject.clone(),
            scopes: Scopes::All,
            issuer: None,
        }));

        self.inner.call((request, context))
    }
}

/// Retrieve an authorization scheme data from a set of headers
pub fn from_headers(headers: &HeaderMap) -> Option<AuthData> {
    headers.get(AUTHORIZATION).and_then(|value| {
        if let Ok(value_str) = value.to_str() {
            // Auth schemes in HTTP are case insensitive so we match on lowercase.
            // Ideally we would use decode without checking for a hardcoded string.
            // Unfortunately `decode` has a debug_assert that verifies the header starts with the scheme.
            // We therefore can only call `decode` if we have a header with a matching scheme.
            if value_str.to_lowercase().starts_with("basic ") {
                Basic::decode(value).map(|basic| {
                    AuthData::Basic(basic.username().to_string(), basic.password().to_string())
                })
            } else if value_str.to_lowercase().starts_with("bearer ") {
                Bearer::decode(value).map(|bearer| AuthData::Bearer(bearer.token().to_string()))
            } else {
                None
            }
        } else {
            None
        }
    })
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
    use crate::context::{ContextBuilder, Has};
    use crate::EmptyContext;
    use http_body_util::Full;
    use hyper::body::Bytes;
    use hyper::service::Service;
    use hyper::Response;

    struct MakeTestService;

    type ReqWithAuth = (
        Request<Full<Bytes>>,
        ContextBuilder<Option<Authorization>, EmptyContext>,
    );

    impl<Target> Service<Target> for MakeTestService {
        type Response = TestService;
        type Error = ();
        type Future = futures::future::Ready<Result<Self::Response, Self::Error>>;

        fn call(&self, _target: Target) -> Self::Future {
            futures::future::ok(TestService)
        }
    }

    struct TestService;

    impl Service<ReqWithAuth> for TestService {
        type Response = Response<Full<Bytes>>;
        type Error = String;
        type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

        fn call(&self, req: ReqWithAuth) -> Self::Future {
            Box::pin(async move {
                let auth: &Option<Authorization> = req.1.get();
                let expected = Some(Authorization {
                    subject: "foo".to_string(),
                    scopes: Scopes::All,
                    issuer: None,
                });

                if *auth == expected {
                    Ok(Response::new(Full::default()))
                } else {
                    Err(format!("{:?} != {:?}", auth, expected))
                }
            })
        }
    }

    #[tokio::test]
    async fn test_make_service() {
        let make_svc = MakeTestService;

        let a: MakeAllowAllAuthenticator<_, EmptyContext> =
            MakeAllowAllAuthenticator::new(make_svc, "foo");

        let service = a.call(&()).await.unwrap();

        let response = service
            .call((
                Request::get("http://localhost")
                    .body(Full::default())
                    .unwrap(),
                EmptyContext,
            ))
            .await;

        response.unwrap();
    }

    #[test]
    fn test_from_headers_basic() {
        let mut headers = HeaderMap::new();
        headers.append(
            AUTHORIZATION,
            headers::HeaderValue::from_static("Basic Zm9vOmJhcg=="),
        );
        assert_eq!(
            from_headers(&headers),
            Some(AuthData::Basic("foo".to_string(), "bar".to_string()))
        )
    }

    #[test]
    fn test_from_headers_bearer() {
        let mut headers = HeaderMap::new();
        headers.append(
            AUTHORIZATION,
            headers::HeaderValue::from_static("Bearer foo"),
        );
        assert_eq!(
            from_headers(&headers),
            Some(AuthData::Bearer("foo".to_string()))
        )
    }
}
