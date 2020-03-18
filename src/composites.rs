//! Module for combining hyper services
//!
//! Use by passing `hyper::server::MakeService` instances to a `CompositeMakeService`
//! together with the base path for requests that should be handled by that service.
use futures::future::FutureExt;
use hyper::service::Service;
use hyper::{Request, Response, StatusCode};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::task::{Context, Poll};

/// Trait for generating a default "not found" response. Must be implemented on
/// the `Response` associated type for `MakeService`s being combined in a
/// `CompositeMakeService`.
pub trait NotFound<V> {
    /// Return a "not found" response
    fn not_found() -> hyper::Response<V>;
}

impl<B: Default> NotFound<B> for B {
    fn not_found() -> hyper::Response<B> {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(B::default())
            .unwrap()
    }
}

type CompositedService<ReqBody, ResBody, Error> = Box<
    dyn Service<
            Request<ReqBody>,
            Response = Response<ResBody>,
            Error = Error,
            Future = futures::future::BoxFuture<'static, Result<Response<ResBody>, Error>>,
        > + Send,
>;

type CompositedMakeService<Target, ReqBody, ResBody, Error, MakeError> = Box<
    dyn Service<
        Target,
        Error = MakeError,
        Future = futures::future::BoxFuture<
            'static,
            Result<CompositedService<ReqBody, ResBody, Error>, MakeError>,
        >,
        Response = CompositedService<ReqBody, ResBody, Error>,
    >,
>;

type CompositeMakeServiceVec<Target, ReqBody, ResBody, Error, MakeError> = Vec<(
    &'static str,
    CompositedMakeService<Target, ReqBody, ResBody, Error, MakeError>,
)>;

/// Wraps a vector of pairs, each consisting of a base path as a `&'static str`
/// and a `MakeService` instance. Implements `Deref<Vec>` and `DerefMut<Vec>` so
/// these can be manipulated using standard `Vec` methods.
///
/// The `Service` returned by calling `make_service()` will pass an incoming
/// request to the first `Service` in the list for which the associated
/// base path is a prefix of the request path.
///
/// Example Usage
/// =============
///
/// ```ignore
/// let my_make_service1 = MakeService1::new();
/// let my_make_service2 = MakeService2::new();
///
/// let mut composite_make_service = CompositeMakeService::new();
/// composite_make_service.push(("/base/path/1", my_make_service1));
/// composite_make_service.push(("/base/path/2", my_make_service2));
///
/// // use as you would any `MakeService` instance
/// ```
#[derive(Default)]
pub struct CompositeMakeService<Target, ReqBody, ResBody, Error, MakeError>(
    CompositeMakeServiceVec<Target, ReqBody, ResBody, Error, MakeError>,
)
where
    ResBody: NotFound<ResBody>;

impl<Target, ReqBody, ResBody, Error, MakeError>
    CompositeMakeService<Target, ReqBody, ResBody, Error, MakeError>
where
    ResBody: NotFound<ResBody>,
{
    /// create an empty `CompositeMakeService`
    pub fn new() -> Self {
        CompositeMakeService(Vec::new())
    }
}

impl<Target, ReqBody, ResBody, Error, MakeError> Service<Target>
    for CompositeMakeService<Target, ReqBody, ResBody, Error, MakeError>
where
    ReqBody: 'static,
    ResBody: NotFound<ResBody> + 'static,
    MakeError: Send + 'static,
    Error: 'static,
    Target: Clone,
{
    type Error = MakeError;
    type Response = CompositeService<ReqBody, ResBody, Error>;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for service in &mut self.0 {
            match service.1.poll_ready(cx) {
                Poll::Ready(Ok(_)) => {}
                Poll::Ready(Err(e)) => {
                    return Poll::Ready(Err(e));
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, target: Target) -> Self::Future {
        let services = self.0.iter_mut().map(|(path, service)| {
            let path: &'static str = path;
            service
                .call(target.clone())
                .map(move |res| res.map(move |service| (path, service)))
        });
        Box::pin(futures::future::join_all(services).map(|results| {
            let services: Result<Vec<_>, MakeError> = results.into_iter().collect();

            Ok(CompositeService(services?))
        }))
    }
}

impl<Target, ReqBody, ResBody, Error, MakeError> fmt::Debug
    for CompositeMakeService<Target, ReqBody, ResBody, Error, MakeError>
where
    ResBody: NotFound<ResBody>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        // Get vector of base paths
        let str_vec: Vec<&'static str> = self.0.iter().map(|&(base_path, _)| base_path).collect();
        write!(
            f,
            "CompositeMakeService accepting base paths: {:?}",
            str_vec,
        )
    }
}

impl<Target, ReqBody, ResBody, Error, MakeError> Deref
    for CompositeMakeService<Target, ReqBody, ResBody, Error, MakeError>
where
    ResBody: NotFound<ResBody>,
{
    type Target = CompositeMakeServiceVec<Target, ReqBody, ResBody, Error, MakeError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Target, ReqBody, ResBody, Error, MakeError> DerefMut
    for CompositeMakeService<Target, ReqBody, ResBody, Error, MakeError>
where
    ResBody: NotFound<ResBody>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Wraps a vector of pairs, each consisting of a base path as a `&'static str`
/// and a `Service` instance.
pub struct CompositeService<ReqBody, ResBody, Error>(
    Vec<(&'static str, CompositedService<ReqBody, ResBody, Error>)>,
)
where
    ResBody: NotFound<ResBody>;

impl<ReqBody, ResBody, Error> Service<Request<ReqBody>>
    for CompositeService<ReqBody, ResBody, Error>
where
    Error: Send + 'static,
    ResBody: NotFound<ResBody> + Send + 'static,
{
    type Error = Error;
    type Response = Response<ResBody>;
    type Future = futures::future::BoxFuture<'static, Result<Response<ResBody>, Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for service in &mut self.0 {
            match service.1.poll_ready(cx) {
                Poll::Ready(Ok(_)) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        for &mut (base_path, ref mut service) in &mut self.0 {
            if req.uri().path().starts_with(base_path) {
                return service.call(req);
            }
        }

        Box::pin(futures::future::ok(ResBody::not_found()))
    }
}

impl<ReqBody, ResBody, Error> fmt::Debug for CompositeService<ReqBody, ResBody, Error>
where
    ResBody: NotFound<ResBody>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        // Get vector of base paths
        let str_vec: Vec<&'static str> = self.0.iter().map(|&(base_path, _)| base_path).collect();
        write!(f, "CompositeService accepting base paths: {:?}", str_vec,)
    }
}

impl<ReqBody, ResBody, Error> Deref for CompositeService<ReqBody, ResBody, Error>
where
    ResBody: NotFound<ResBody> + 'static,
    Error: 'static,
{
    type Target = Vec<(&'static str, CompositedService<ReqBody, ResBody, Error>)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<ReqBody, ResBody, Error> DerefMut for CompositeService<ReqBody, ResBody, Error>
where
    ResBody: NotFound<ResBody> + 'static,
    Error: 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
