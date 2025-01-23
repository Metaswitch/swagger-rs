//! Hyper service that drops a context to an incoming request and passes it on
//! to a wrapped service.

use http::Request;
use std::marker::PhantomData;

use futures::future::FutureExt as _;

/// Middleware wrapper service that drops the context from the incoming request
/// and passes the plain `http::Request` to the wrapped service.
///
/// This service can be used to to include services that take a plain `http::Request`
/// in a `CompositeService` wrapped in an `AddContextService`.
///
/// Example Usage
/// =============
///
/// In the following example `SwaggerService` implements `hyper::service::MakeService`
/// with `Request = (http::Request, SomeContext)`, and `PlainService` implements it
/// with `Request = http::Request`
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
    Inner::Future: Send + 'static,
{
    type Response = DropContextService<Inner::Response, Context>;
    type Error = Inner::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, target: Target) -> Self::Future {
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
/// ```
/// # use swagger::DropContextService;
/// # use hyper::service::Service as _;
/// # use hyper_util::client::legacy::Client;
/// # use hyper_util::rt::TokioExecutor;
/// # use hyper_util::service::TowerToHyperService;
/// # use http_body_util::Empty;
/// # use bytes::Bytes;
/// let client = Client::builder(TokioExecutor::new()).build_http();
/// let client = DropContextService::new(TowerToHyperService::new(client));
/// let request = (http::Request::get("http://www.google.com").body(Empty::<Bytes>::new()).unwrap());
/// let context = "Some Context".to_string();
///
/// let response = client.call((request, context));
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

impl<Inner, Body, Context> hyper::service::Service<(Request<Body>, Context)>
    for DropContextService<Inner, Context>
where
    Context: Send + 'static,
    Inner: hyper::service::Service<Request<Body>>,
{
    type Response = Inner::Response;
    type Error = Inner::Error;
    type Future = Inner::Future;

    fn call(&self, (req, _): (Request<Body>, Context)) -> Self::Future {
        self.inner.call(req)
    }
}
