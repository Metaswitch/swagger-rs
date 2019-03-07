//! Module for combining hyper services
//!
//! Use by passing `hyper::server::NewService` instances to a `CompositeNewService`
//! together with the base path for requests that should be handled by that service.
use futures::{future, Future};
use hyper::service::{MakeService, Service};
use hyper::{Request, Response, StatusCode};
use std::ops::{Deref, DerefMut};
use std::{fmt, io};

/// Trait for generating a default "not found" response. Must be implemented on
/// the `Response` associated type for `NewService`s being combined in a
/// `CompositeNewService`.
pub trait NotFound<V> {
    /// Return a "not found" response
    fn not_found() -> hyper::Response<V>;
}

impl NotFound<hyper::Body> for hyper::Body {
    fn not_found() -> hyper::Response<hyper::Body> {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(hyper::Body::empty())
            .unwrap()
    }
}

type BoxedFuture<V, W> = Box<Future<Item = V, Error = W>>;
type CompositeNewServiceVec<C, U, V, W> = Vec<(&'static str, Box<BoxedNewService<C, U, V, W>>)>;
type BoxedService<U, V, W> =
    Box<Service<ReqBody = U, ResBody = V, Error = W, Future = BoxedFuture<Response<V>, W>>>;

/// Trait for wrapping hyper `NewService`s to make the return type of `new_service` uniform.
/// This is necessary in order for the `NewService`s with different `Instance` types to
/// be stored in a single collection.
pub trait BoxedNewService<C, U, V, W> {
    /// Create a new `Service` trait object
    fn boxed_new_service(&mut self) -> Result<BoxedService<U, V, W>, io::Error>;
}

impl<C, T, Rq, Rs, Er, S> BoxedNewService<C, Rq, Rs, Er> for T
where
    S: Service<ReqBody = Rq, ResBody = Rs, Error = Er, Future = BoxedFuture<Response<Rs>, Er>>
        + 'static,
    T: MakeService<
        C,
        ReqBody = Rq,
        ResBody = Rs,
        Error = Er,
        Future = futures::future::FutureResult<S, io::Error>,
        Service = S,
        MakeError = io::Error,
    >,
    Rq: hyper::body::Payload,
    Rs: hyper::body::Payload,
    Er: std::error::Error + Send + Sync + 'static,
    C: Default,
{
    /// Call the `new_service` method of the wrapped `NewService` and `Box` the result
    fn boxed_new_service(&mut self) -> Result<BoxedService<Rq, Rs, Er>, io::Error> {
        let service = self.make_service(C::default()).wait()?;
        Ok(Box::new(service))
    }
}

/// Wraps a vector of pairs, each consisting of a base path as a `&'static str`
/// and a `NewService` instance. Implements `Deref<Vec>` and `DerefMut<Vec>` so
/// these can be manipulated using standard `Vec` methods.
///
/// The `Service` returned by calling `new_service()` will pass an incoming
/// request to the first `Service` in the list for which the associated
/// base path is a prefix of the request path.
///
/// Example Usage
/// =============
///
/// ```ignore
/// let my_new_service1 = NewService1::new();
/// let my_new_service2 = NewService2::new();
///
/// let mut composite_new_service = CompositeNewService::new();
/// composite_new_service.push(("/base/path/1", my_new_service1));
/// composite_new_service.push(("/base/path/2", my_new_service2));
///
/// // use as you would any `NewService` instance
/// ```
#[derive(Default)]
pub struct CompositeNewService<C, U, V, W>(CompositeNewServiceVec<C, U, V, W>)
where
    V: NotFound<V> + 'static,
    W: 'static;

// Workaround for https://github.com/rust-lang-nursery/rust-clippy/issues/2226
#[cfg_attr(
    feature = "cargo-clippy",
    allow(
        renamed_and_removed_lints,
        new_without_default_derive,
        clippy::new_without_default_derive
    )
)]
impl<C, U, V: NotFound<V>, W> CompositeNewService<C, U, V, W> {
    /// create an empty `CompositeNewService`
    pub fn new() -> Self {
        CompositeNewService(Vec::new())
    }
}

impl<C, U, V, W> MakeService<C> for CompositeNewService<C, U, V, W>
where
    U: hyper::body::Payload,
    V: NotFound<V> + 'static + hyper::body::Payload,
    W: std::error::Error + Send + Sync + 'static,
{
    type ReqBody = U;
    type ResBody = V;
    type Error = W;
    type Service = CompositeService<U, V, W>;
    type MakeError = io::Error;
    type Future = futures::future::FutureResult<Self::Service, io::Error>;

    fn make_service(
        &mut self,
        _service_ctx: C,
    ) -> futures::future::FutureResult<Self::Service, io::Error> {
        let mut vec = Vec::new();

        for &mut (base_path, ref mut new_service) in &mut self.0 {
            vec.push((base_path, new_service.boxed_new_service().expect("Error")))
        }

        future::FutureResult::from(Ok(CompositeService(vec)))
    }
}

impl<C, U, V, W> fmt::Debug for CompositeNewService<C, U, V, W>
where
    V: NotFound<V> + 'static,
    W: 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // Get vector of base paths
        let str_vec: Vec<&'static str> = self.0.iter().map(|&(base_path, _)| base_path).collect();
        write!(f, "CompositeNewService accepting base paths: {:?}", str_vec,)
    }
}

impl<C, U, V, W> Deref for CompositeNewService<C, U, V, W>
where
    V: NotFound<V> + 'static,
    W: 'static,
{
    type Target = CompositeNewServiceVec<C, U, V, W>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C, U, V, W> DerefMut for CompositeNewService<C, U, V, W>
where
    V: NotFound<V> + 'static,
    W: 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Wraps a vector of pairs, each consisting of a base path as a `&'static str`
/// and a `Service` instance.
pub struct CompositeService<U, V, W>(Vec<(&'static str, BoxedService<U, V, W>)>)
where
    V: NotFound<V> + 'static,
    W: 'static;

impl<U, V, W> Service for CompositeService<U, V, W>
where
    U: hyper::body::Payload,
    V: NotFound<V> + 'static + hyper::body::Payload,
    W: 'static + std::error::Error + Send + Sync,
{
    type ReqBody = U;
    type ResBody = V;
    type Error = W;
    type Future = Box<Future<Item = Response<V>, Error = W>>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let mut result = None;

        for &mut (base_path, ref mut service) in &mut self.0 {
            if req.uri().path().starts_with(base_path) {
                result = Some(service.call(req));
                break;
            }
        }

        result.unwrap_or_else(|| Box::new(future::ok(V::not_found())))
    }
}

impl<U, V, W> fmt::Debug for CompositeService<U, V, W>
where
    V: NotFound<V> + 'static,
    W: 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // Get vector of base paths
        let str_vec: Vec<&'static str> = self.0.iter().map(|&(base_path, _)| base_path).collect();
        write!(f, "CompositeService accepting base paths: {:?}", str_vec,)
    }
}

impl<U, V, W> Deref for CompositeService<U, V, W>
where
    V: NotFound<V> + 'static,
    W: 'static,
{
    type Target = Vec<(&'static str, BoxedService<U, V, W>)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<U, V, W> DerefMut for CompositeService<U, V, W>
where
    V: NotFound<V> + 'static,
    W: 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
