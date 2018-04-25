//! Hyper service that drops a context to an incoming request and passes it on
//! to a wrapped service.

use std::io;
use std::marker::PhantomData;
use hyper;
use hyper::{Request, Response, Error};
use super::{Push, XSpanIdString};

/// Middleware wrapper service, that can be used to include services that take a plain
/// `hyper::Request` in a `CompositeService` wrapped in an `AddContext` service.
/// Drops the context from the incoming request and passes the plain `hyper::Request`
/// to the wrapped service.
#[derive(Debug)]
pub struct DropContext<T, C>
where
    C: Default + Push<XSpanIdString>,
{
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> DropContext<T, C>
where
    C: Default + Push<XSpanIdString>,
{
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
        C: Default + Push<XSpanIdString>,
        T: hyper::server::NewService<Request=Request, Response=Response, Error=Error>,

{
    type Request = (Request, C::Result);
    type Response = Response;
    type Error = Error;
    type Instance = DropContext<T::Instance, C>;

    fn new_service(&self) -> Result<Self::Instance, io::Error> {
        self.inner.new_service().map(DropContext::new)
    }
}

impl<T, C> hyper::server::Service for DropContext<T, C>
where
    C: Default + Push<XSpanIdString>,
    T: hyper::server::Service<
        Request = Request,
        Response = Response,
        Error = Error,
    >,
{
    type Request = (Request, C::Result);
    type Response = Response;
    type Error = Error;
    type Future = T::Future;

    fn call(&self, (req, _): Self::Request) -> Self::Future {
        self.inner.call(req)
    }
}
