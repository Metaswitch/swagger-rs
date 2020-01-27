//! Hyper service that drops a context to an incoming request and passes it on
//! to a wrapped service.

use crate::context::ContextualPayload;
use futures::future::FutureExt;
use hyper;
use hyper::Request;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::Poll;

/// Middleware wrapper service that drops the context from the incoming request
/// and passes the plain `hyper::Request` to the wrapped service.
///
/// This service can be used to to include services that take a plain `hyper::Request`
/// in a `CompositeService` wrapped in an `AddContextService`.
///
/// Example Usage
/// =============
///
/// In the following example `SwaggerService` implements `hyper::service::MakeService`
/// with `Request = (hyper::Request, SomeContext)`, and `PlainService` implements it
/// with `Request = hyper::Request`
///
/// ```ignore
/// let swagger_service_one = SwaggerService::new();
/// let swagger_service_two = SwaggerService::new();
/// let plain_service = PlainService::new();
///
/// let mut composite_new_service = CompositeMakeService::new();
/// composite_new_service.push(("/base/path/1", swagger_service_one));
/// composite_new_service.push(("/base/path/2", swagger_service_two));
/// composite_new_service.push(("/base/path/3", DropContextMakeService::new(plain_service)));
/// ```
#[derive(Debug)]
pub struct DropContextMakeService<T, C>
where
    C: Send + 'static,
{
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> DropContextMakeService<T, C>
where
    C: Send + 'static,
{
    /// Create a new DropContextMakeService struct wrapping a value
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<Inner, Context, Target> hyper::service::Service<Target>
    for DropContextMakeService<Inner, Context>
where
    Context: Send + 'static,
    Inner: hyper::service::Service<Target>,
    Inner::Future: 'static,
{
    type Response = DropContextService<Inner::Response, Context>;
    type Error = Inner::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, target: Target) -> Self::Future {
        Box::pin(
            self.inner
                .call(target)
                .map(|s| Ok(DropContextService::new(s?))),
        )
    }
}

/// Swagger Middleware that wraps a `hyper::service::Service` and drops any contextual information
/// on the request. Servers will normally want to use `DropContextMakeService`, which will create a
/// `DropContextService` to handle each connection, while clients can simply wrap a `hyper::Client`
/// in the middleware.
///
/// ## Client Usage
///
/// ```edition2018
/// # use swagger::{DropContextService, ContextualPayload};
/// # use hyper::service::Service as _;
///
/// let client = hyper::Client::new();
/// let client = DropContextService::new(client);
/// let body = ContextualPayload { inner: hyper::Body::empty(), context: "Some Context".to_string() };
/// let request = hyper::Request::get("http://www.google.com").body(body).unwrap();
///
/// let response = client.call(request);
/// ```
#[derive(Debug, Clone)]
pub struct DropContextService<T, C>
where
    C: Send + 'static,
{
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> DropContextService<T, C>
where
    C: Send + 'static,
{
    /// Create a new DropContextService struct wrapping a value
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<Inner, Body, Context> hyper::service::Service<Request<ContextualPayload<Body, Context>>>
    for DropContextService<Inner, Context>
where
    Context: Send + 'static,
    Inner: hyper::service::Service<Request<Body>>,
{
    type Response = Inner::Response;
    type Error = Inner::Error;
    type Future = Inner::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ContextualPayload<Body, Context>>) -> Self::Future {
        let (head, body) = req.into_parts();
        let body = body.inner;
        self.inner.call(Request::from_parts(head, body))
    }
}
