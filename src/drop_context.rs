//! Hyper service that drops a context to an incoming request and passes it on
//! to a wrapped service.

use std::io;
use std::marker::PhantomData;
use hyper;
use hyper::{Request, Response, Error};

/// Middleware wrapper service that trops the context from the incoming request
/// and passes the plain `hyper::Request` to the wrapped service.
///
/// This service can be used to to include services that take a plain `hyper::Request`
/// in a `CompositeService` wrapped in an `AddContext` service.
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

impl<T, C> hyper::server::NewService for DropContext<T, C>
    where
        T: hyper::server::NewService<Request=Request, Response=Response, Error=Error>,

{
    type Request = (Request, C);
    type Response = Response;
    type Error = Error;
    type Instance = DropContext<T::Instance, C>;

    fn new_service(&self) -> Result<Self::Instance, io::Error> {
        self.inner.new_service().map(DropContext::new)
    }
}

impl<T, C> hyper::server::Service for DropContext<T, C>
where
    T: hyper::server::Service<
        Request = Request,
        Response = Response,
        Error = Error,
    >,
{
    type Request = (Request, C);
    type Response = Response;
    type Error = Error;
    type Future = T::Future;

    fn call(&self, (req, _): Self::Request) -> Self::Future {
        self.inner.call(req)
    }
}
