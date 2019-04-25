//! Hyper service that drops a context to an incoming request and passes it on
//! to a wrapped service.

use context::ContextualPayload;
use futures::Future;
use hyper;
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
    type Future = Box<Future<Item = Self::Service, Error = io::Error>>;
    type Service = DropContextService<S, RC>;

    fn make_service(&mut self, service_ctx: &'a SC) -> Self::Future {
        Box::new(
            self.inner
                .make_service(service_ctx)
                .map(DropContextService::new),
        )
    }
}

/// Swagger Middleware that wraps a `hyper::service::Service`, and drops any contextual information
/// on the request. Services will normally want to use `DropContextMakeService`, which will create
/// a `DropContextService` to handle each connection.
#[derive(Debug)]
pub struct DropContextService<T, C>
where
    C: Send + 'static,
    T: hyper::service::Service<ReqBody = hyper::Body, ResBody = hyper::Body, Error = Error>,
{
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> DropContextService<T, C>
where
    C: Send + 'static,
    T: hyper::service::Service<ReqBody = hyper::Body, ResBody = hyper::Body, Error = Error>,
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
