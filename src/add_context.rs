//! Hyper service that adds a context to an incoming request and passes it on
//! to a wrapped service.

use super::{Push, XSpanIdString};
use auth::ContextualPayload;
use futures::Future;
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
    C::Result: Send + 'static,
    T: hyper::service::MakeService<
        C,
        ReqBody = ContextualPayload<hyper::Body, C::Result>,
        ResBody = hyper::Body,
        Error = hyper::Error,
    >,
{
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> AddContextNewService<T, C>
where
    C: Default + Push<XSpanIdString>,
    C::Result: Send + 'static,
    T: hyper::service::MakeService<
        C,
        ReqBody = ContextualPayload<hyper::Body, C::Result>,
        ResBody = hyper::Body,
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

impl<T, C> hyper::service::MakeService<C> for AddContextNewService<T, C>
where
    C: Default + Push<XSpanIdString> + 'static + Send,
    C::Result: Send + 'static,
    T: hyper::service::MakeService<
        C,
        ReqBody = ContextualPayload<hyper::Body, C::Result>,
        ResBody = hyper::Body,
        Error = hyper::Error,
        MakeError = io::Error,
    >,
    T::Service: 'static,
    T::Future: 'static,
{
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = hyper::Error;
    type Service = AddContextService<T::Service, C>;
    type MakeError = io::Error;
    type Future = Box<Future<Item = Self::Service, Error = io::Error>>;

    fn make_service(&mut self, service_ctx: C) -> Self::Future {
        Box::new(
            self.inner
                .make_service(service_ctx)
                .map(AddContextService::new),
        )
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
    C::Result: Send + 'static,
    T: hyper::service::Service<
        ReqBody = ContextualPayload<hyper::Body, C::Result>,
        ResBody = hyper::Body,
        Error = hyper::Error,
    >,
{
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> AddContextService<T, C>
where
    C: Default + Push<XSpanIdString>,
    C::Result: Send + 'static,
    T: hyper::service::Service<
        ReqBody = ContextualPayload<hyper::Body, C::Result>,
        ResBody = hyper::Body,
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

impl<T, C> hyper::service::Service for AddContextService<T, C>
where
    C: Default + Push<XSpanIdString>,
    C::Result: Send + 'static,
    T: hyper::service::Service<
        ReqBody = ContextualPayload<hyper::Body, C::Result>,
        ResBody = hyper::Body,
        Error = hyper::Error,
    >,
{
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = hyper::Error;
    type Future = T::Future;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let x_span_id = XSpanIdString::get_or_generate(&req);
        let (head, body) = req.into_parts();
        let context = C::default().push(x_span_id);

        let body = ContextualPayload {
            inner: body,
            context: context,
        };
        self.inner.call(hyper::Request::from_parts(head, body))
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
impl<T, C> hyper::service::MakeService<C> for AddContext<T, C>
where
    C: Default + Push<XSpanIdString> + 'static,
    C::Result: Send + 'static,
    T: hyper::service::MakeService<
        C,
        ReqBody = ContextualPayload<hyper::Body, C::Result>,
        ResBody = hyper::Body,
        Error = Error,
        MakeError = io::Error,
    >,
    T::Future: 'static,
    T::Service: 'static,
{
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = Error;
    type Service = AddContext<T::Service, C>;
    type MakeError = io::Error;
    type Future = Box<Future<Item = Self::Service, Error = io::Error>>;

    fn make_service(&mut self, service_ctx: C) -> Self::Future {
        Box::new(self.inner.make_service(service_ctx).map(AddContext::new))
    }
}

#[allow(deprecated)]
impl<T, C> hyper::service::Service for AddContext<T, C>
where
    C: Default + Push<XSpanIdString>,
    C::Result: Send + 'static,
    T: hyper::service::Service<
        ReqBody = ContextualPayload<hyper::Body, C::Result>,
        ResBody = hyper::Body,
        Error = Error,
    >,
{
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = Error;
    type Future = T::Future;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let x_span_id = XSpanIdString::get_or_generate(&req);
        let (head, body) = req.into_parts();
        let context = C::default().push(x_span_id);

        let body = ContextualPayload {
            inner: body,
            context: context,
        };
        self.inner.call(hyper::Request::from_parts(head, body))
    }
}
