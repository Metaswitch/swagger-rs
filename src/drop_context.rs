//! Hyper service that drops a context to an incoming request and passes it on
//! to a wrapped service.

use auth::ContextualPayload;
use futures::Future;
use hyper;
use hyper::{Error, Request, Response};
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
/// In the following example `SwaggerService` implements `hyper::server::NewService`
/// with `Request = (hyper::Request, SomeContext)`, and `PlainService` implements it
/// with `Request = hyper::Request`
///
/// ```ignore
/// let swagger_service_one = SwaggerService::new();
/// let swagger_service_two = SwaggerService::new();
/// let plain_service = PlainService::new();
///
/// let mut composite_new_service = CompositeNewService::new();
/// composite_new_service.push(("/base/path/1", swagger_service_one));
/// composite_new_service.push(("/base/path/2", swagger_service_two));
/// composite_new_service.push(("/base/path/3", DropContext::new(plain_service)));
/// ```
#[derive(Debug)]
pub struct DropContext<T, C> {
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> DropContext<T, C> {
    /// Create a new DropContext struct wrapping a value
    pub fn new(inner: T) -> Self {
        DropContext {
            inner,
            marker: PhantomData,
        }
    }
}

impl<T, C> hyper::service::MakeService<C> for DropContext<T, C>
where
    C: Send + 'static,
    T: hyper::service::MakeService<
        C,
        ReqBody = hyper::Body,
        ResBody = hyper::Body,
        Error = Error,
        MakeError = io::Error,
    >,
    T::Future: 'static,
    T::Service: 'static,
{
    type ReqBody = ContextualPayload<hyper::Body, C>;
    type ResBody = hyper::Body;
    type Error = Error;
    type MakeError = io::Error;
    type Future = Box<Future<Item = Self::Service, Error = io::Error>>;
    type Service = DropContext<T::Service, C>;

    fn make_service(&mut self, service_ctx: C) -> Self::Future {
        Box::new(self.inner.make_service(service_ctx).map(DropContext::new))
    }
}

impl<T, C> hyper::service::Service for DropContext<T, C>
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
