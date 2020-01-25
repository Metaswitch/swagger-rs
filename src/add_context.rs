//! Hyper service that adds a context to an incoming request and passes it on
//! to a wrapped service.

use crate::context::ContextualPayload;
use crate::{ErrorBound, Push, XSpanIdString};
use futures::Future;
use hyper;
use hyper::Request;
use std::marker::PhantomData;

/// Middleware wrapper service, that should be used as the outermost layer in a
/// stack of hyper services. Adds a context to a plain `hyper::Request` that can be
/// used by subsequent layers in the stack.
#[derive(Debug)]
pub struct AddContextMakeService<T, C>
where
    C: Default + Push<XSpanIdString> + 'static + Send,
    C::Result: Send + 'static,
{
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> AddContextMakeService<T, C>
where
    C: Default + Push<XSpanIdString> + 'static + Send,
    C::Result: Send + 'static,
{
    /// Create a new AddContextMakeService struct wrapping a value
    pub fn new(inner: T) -> Self {
        AddContextMakeService {
            inner,
            marker: PhantomData,
        }
    }
}

impl<'a, T, SC, RC, E, ME, S, F> hyper::service::MakeService<&'a SC>
    for AddContextMakeService<T, RC>
where
    RC: Default + Push<XSpanIdString> + 'static + Send,
    RC::Result: Send + 'static,
    T: hyper::service::MakeService<
        &'a SC,
        Service = S,
        ReqBody = ContextualPayload<hyper::Body, RC::Result>,
        ResBody = hyper::Body,
        Error = E,
        MakeError = ME,
        Future = F,
    >,
    S: hyper::service::Service<
            ReqBody = ContextualPayload<hyper::Body, RC::Result>,
            ResBody = hyper::Body,
            Error = E,
        > + 'static,
    ME: ErrorBound,
    E: ErrorBound,
    F: Future<Item = S, Error = ME> + Send + 'static,
    S::Future: Send,
{
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = E;
    type Service = AddContextService<S, RC>;
    type MakeError = ME;
    type Future = Box<dyn Future<Item = Self::Service, Error = ME> + Send>;

    fn make_service(&mut self, service_ctx: &'a SC) -> Self::Future {
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
/// `AddContextMakeService`, which will create `AddContextService` instances as needed.
#[derive(Debug)]
pub struct AddContextService<T, C>
where
    C: Default + Push<XSpanIdString>,
    C::Result: Send + 'static,
{
    inner: T,
    marker: PhantomData<C>,
}

impl<T, C> AddContextService<T, C>
where
    C: Default + Push<XSpanIdString>,
    C::Result: Send + 'static,
{
    /// Create a new AddContextService struct wrapping a value
    pub fn new(inner: T) -> Self {
        AddContextService {
            inner,
            marker: PhantomData,
        }
    }
}

impl<T, C, E> hyper::service::Service for AddContextService<T, C>
where
    C: Default + Push<XSpanIdString>,
    C::Result: Send + 'static,
    T: hyper::service::Service<
        ReqBody = ContextualPayload<hyper::Body, C::Result>,
        ResBody = hyper::Body,
        Error = E,
    >,
    E: ErrorBound,
{
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = E;
    type Future = T::Future;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let x_span_id = XSpanIdString::get_or_generate(&req);
        let (head, body) = req.into_parts();
        let context = C::default().push(x_span_id);

        let body = ContextualPayload {
            inner: body,
            context,
        };
        self.inner.call(hyper::Request::from_parts(head, body))
    }
}
