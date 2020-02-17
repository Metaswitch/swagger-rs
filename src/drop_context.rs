//! Hyper service that drops a context to an incoming request and passes it on
//! to a wrapped service.

use crate::context::ContextualPayload;
use futures::Future;
use hyper::{Error, Request};
use std::io;
use std::marker::PhantomData;

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

impl<'a, SC, RC, T, S, F> hyper::service::MakeService<&'a SC> for DropContextMakeService<T, RC>
where
    RC: Send + 'static,
    T: hyper::service::MakeService<
        &'a SC,
        ReqBody = hyper::Body,
        ResBody = hyper::Body,
        Error = Error,
        MakeError = io::Error,
        Service = S,
        Future = F,
    >,
    T::Future: 'static,
    S: hyper::service::Service<ReqBody = hyper::Body, ResBody = hyper::Body, Error = Error>
        + 'static,
    F: Future<Item = S, Error = io::Error>,
{
    type ReqBody = ContextualPayload<hyper::Body, RC>;
    type ResBody = hyper::Body;
    type Error = Error;
    type MakeError = io::Error;
    type Future = Box<dyn Future<Item = Self::Service, Error = io::Error>>;
    type Service = DropContextService<S, RC>;

    fn make_service(&mut self, service_ctx: &'a SC) -> Self::Future {
        Box::new(
            self.inner
                .make_service(service_ctx)
                .map(DropContextService::new),
        )
    }
}

/// Swagger Middleware that wraps a `hyper::service::Service` or a `swagger::client::Service`, and
/// drops any contextual information on the request. Servers will normally want to use
/// `DropContextMakeService`, which will create a `DropContextService` to handle each connection,
/// while clients can simply wrap a `hyper::Client` in the middleware.
///
/// ## Client Usage
///
/// ```edition2018
/// # use swagger::{DropContextService, ContextualPayload};
/// # use swagger::client::Service as _;
///
/// let client = hyper::Client::new();
/// let client = DropContextService::new(client);
/// let body = ContextualPayload { inner: hyper::Body::empty(), context: "Some Context".to_string() };
/// let request = hyper::Request::get("http://www.google.com").body(body).unwrap();
///
/// let response = client.request(request);
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

impl<T, C> hyper::service::Service for DropContextService<T, C>
where
    C: Send + 'static,
    T: hyper::service::Service<ReqBody = hyper::Body, ResBody = hyper::Body, Error = Error>,
{
    type ReqBody = ContextualPayload<hyper::Body, C>;
    type ResBody = hyper::Body;
    type Error = Error;
    type Future = T::Future;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let (head, body) = req.into_parts();
        let body = body.inner;
        self.inner.call(Request::from_parts(head, body))
    }
}

impl<T, C> crate::client::Service for DropContextService<T, C>
where
    C: Send + 'static,
    T: crate::client::Service<ReqBody = hyper::Body>,
{
    type ReqBody = ContextualPayload<hyper::Body, C>;
    type Future = T::Future;

    fn request(&self, request: Request<Self::ReqBody>) -> Self::Future {
        let (head, body) = request.into_parts();
        self.inner.request(Request::from_parts(head, body.inner))
    }
}
