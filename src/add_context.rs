//! Hyper service that adds a context to an incoming request and passes it on
//! to a wrapped service.

use super::{Push, XSpanIdString};
use hyper;
use hyper::{Error, Request, Response};
use std::io;
use std::marker::PhantomData;

/// Middleware wrapper service, that should be used as the outermost layer in a
/// stack of hyper services. Adds a context to a plain `hyper::Request` that can be
/// used by subsequent layers in the stack.
#[derive(Debug)]
pub struct AddContextNewService<T, C>
where
    C: Default + Push<XSpanIdString>,
    T: hyper::server::NewService<
        Request = (hyper::Request, C::Result),
        Response = hyper::Response,
        Error = hyper::Error,
    >,
{
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> AddContextNewService<T, C>
where
    C: Default + Push<XSpanIdString>,
    T: hyper::server::NewService<
        Request = (hyper::Request, C::Result),
        Response = hyper::Response,
        Error = hyper::Error,
    >,
{
    /// Create a new AddContextNewService struct wrapping a value
    pub fn new(inner: T) -> Self {
        AddContextNewService {
            inner,
            marker: PhantomData,
        }
    }
}

impl<T, C> hyper::server::NewService for AddContextNewService<T, C>
where
    C: Default + Push<XSpanIdString>,
    T: hyper::server::NewService<
        Request = (hyper::Request, C::Result),
        Response = hyper::Response,
        Error = hyper::Error,
    >,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Instance = AddContextService<T::Instance, C>;

    fn new_service(&self) -> Result<Self::Instance, io::Error> {
        self.inner.new_service().map(AddContextService::new)
    }
}

/// Middleware wrapper service, that should be used as the outermost layer in a
/// stack of hyper services. Adds a context to a plain `hyper::Request` that can be
/// used by subsequent layers in the stack. The `AddContextService` struct should
/// not usually be used directly - when constructing a hyper stack use
/// `AddContextNewService`, which will create `AddContextService` instances as needed.
#[derive(Debug)]
pub struct AddContextService<T, C>
where
    C: Default + Push<XSpanIdString>,
    T: hyper::server::Service<
        Request = (hyper::Request, C::Result),
        Response = hyper::Response,
        Error = hyper::Error,
    >,
{
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> AddContextService<T, C>
where
    C: Default + Push<XSpanIdString>,
    T: hyper::server::Service<
        Request = (hyper::Request, C::Result),
        Response = hyper::Response,
        Error = hyper::Error,
    >,
{
    /// Create a new AddContextService struct wrapping a value
    pub fn new(inner: T) -> Self {
        AddContextService {
            inner,
            marker: PhantomData,
        }
    }
}

impl<T, C> hyper::server::Service for AddContextService<T, C>
where
    C: Default + Push<XSpanIdString>,
    T: hyper::server::Service<
        Request = (hyper::Request, C::Result),
        Response = hyper::Response,
        Error = hyper::Error,
    >,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = T::Future;

    fn call(&self, req: Self::Request) -> Self::Future {
        let x_span_id = XSpanIdString::get_or_generate(&req);
        let context = C::default().push(x_span_id);
        self.inner.call((req, context))
    }
}

/// **DEPRECATED - USE `AddContextNewService` AND `AddContextService` INSTEAD**
/// Middleware wrapper service, that should be used as the outermost layer in a
/// stack of hyper services. Adds a context to a plain `hyper::Request` that can be
/// used by subsequent layers in the stack.
#[deprecated(
    since = "2.0.0",
    note = "Replace with `AddContextNewService` or `AddContextService`"
)]
#[derive(Debug)]
pub struct AddContext<T, C>
where
    C: Default + Push<XSpanIdString>,
{
    inner: T,
    marker: PhantomData<C>,
}

#[allow(deprecated)]
impl<T, C> AddContext<T, C>
where
    C: Default + Push<XSpanIdString>,
{
    /// Create a new AddContext struct wrapping a value
    pub fn new(inner: T) -> Self {
        AddContext {
            inner,
            marker: PhantomData,
        }
    }
}

#[allow(deprecated)]
impl<T, C> hyper::server::NewService for AddContext<T, C>
where
    C: Default + Push<XSpanIdString>,
    T: hyper::server::NewService<
        Request = (Request, C::Result),
        Response = Response,
        Error = Error,
    >,
{
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Instance = AddContext<T::Instance, C>;

    fn new_service(&self) -> Result<Self::Instance, io::Error> {
        self.inner.new_service().map(AddContext::new)
    }
}

#[allow(deprecated)]
impl<T, C> hyper::server::Service for AddContext<T, C>
where
    C: Default + Push<XSpanIdString>,
    T: hyper::server::Service<Request = (Request, C::Result), Response = Response, Error = Error>,
{
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Future = T::Future;

    fn call(&self, req: Self::Request) -> Self::Future {
        let x_span_id = XSpanIdString::get_or_generate(&req);
        let context = C::default().push(x_span_id);
        self.inner.call((req, context))
    }
}
