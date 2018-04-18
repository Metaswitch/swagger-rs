//! Module for combining hyper services
//!
//! Use by passing `hyper::server::NewService` instances to a `CompositeNewService`
//! together with the base path for requests that should be handled by that service.
use std::{io, fmt};
use std::ops::{Deref, DerefMut};
use hyper::server::{Service, NewService};
use hyper::{Request, Response, StatusCode};
use futures::{future, Future};

/// Trait for getting the path of a request. Must be implemented on the `Request`
/// associated type for `NewService`s being combined in a `CompositeNewService`.
pub trait GetPath {
    /// Retrieve the path
    fn path(&self) -> &str;
}

impl GetPath for Request {
    fn path(&self) -> &str {
        self.path()
    }
}

impl<C> GetPath for (Request, C) {
    fn path(&self) -> &str {
        self.0.path()
    }
}

/// Trait for generating a default "not found" response. Must be implemented on
/// the `Response` associated type for `NewService`s being combined in a
/// `CompositeNewService`.
pub trait NotFound {
    /// Return a "not found" response
    fn not_found() -> Self;
}

impl NotFound for Response {
    fn not_found() -> Self {
        Response::new().with_status(StatusCode::NotFound)
    }
}

type BoxedFuture<V, W> = Box<Future<Item = V, Error = W>>;
type CompositeNewServiceVec<U, V, W> = Vec<(&'static str, Box<BoxedNewService<U, V, W>>)>;
type BoxedService<U, V, W> = Box<
    Service<
        Request = U,
        Response = V,
        Error = W,
        Future = BoxedFuture<V, W>,
    >,
>;

/// Trait for wrapping hyper `NewService`s to make the return type of `new_service` uniform.
/// This is necessary in order for the `NewService`s with different `Instance` types to
/// be stored in a single collection.
pub trait BoxedNewService<U, V, W> {
    /// Create a new `Service` trait object
    fn boxed_new_service(&self) -> Result<BoxedService<U, V, W>, io::Error>;
}

impl<T, U, V, W> BoxedNewService<U, V, W> for T
where
    T: NewService<Request = U, Response = V, Error = W>,
    T::Instance: Service<Future = BoxedFuture<V, W>>
        + 'static,
{
    /// Call the `new_service` method of the wrapped `NewService` and `Box` the result
    fn boxed_new_service(
        &self,
    ) -> Result<
        BoxedService<U, V, W>,
        io::Error,
    > {
        let service = self.new_service()?;
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
/// Usage:
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
pub struct CompositeNewService<U, V, W>(CompositeNewServiceVec<U, V, W>)
where
    U: GetPath,
    V: NotFound + 'static,
    W: 'static;

// Clippy bug? This lint triggers despite having a #[derive(Default)]
#[cfg_attr(feature = "cargo-clippy", allow(new_without_default_derive))]
impl<U: GetPath, V: NotFound, W> CompositeNewService<U, V, W> {
    /// create an empty `CompositeNewService`
    pub fn new() -> Self {
        CompositeNewService(Vec::new())
    }
}

impl<U, V, W> NewService for CompositeNewService<U, V, W>
where
    U: GetPath,
    V: NotFound + 'static,
    W: 'static,
{
    type Request = U;
    type Response = V;
    type Error = W;
    type Instance = CompositeService<U, V, W>;

    fn new_service(&self) -> Result<Self::Instance, io::Error> {
        let mut vec = Vec::new();

        for &(base_path, ref new_service) in &self.0 {
            vec.push((base_path, new_service.boxed_new_service()?))
        }

        Ok(CompositeService(vec))
    }
}

impl<U, V, W> fmt::Debug for CompositeNewService<U, V, W>
where
    U: GetPath,
    V: NotFound + 'static,
    W: 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // Get vector of base paths
        let str_vec: Vec<&'static str> = self.0.iter().map(|&(base_path, _)| base_path).collect();
        write!(
            f,
            "CompositeNewService accepting base paths: {:?}",
            str_vec,
        )
    }
}

impl<U, V, W> Deref for CompositeNewService<U, V, W>
where
    U: GetPath,
    V: NotFound + 'static,
    W: 'static,
{
    type Target = CompositeNewServiceVec<U, V, W>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<U, V, W> DerefMut for CompositeNewService<U, V, W>
where
    U: GetPath,
    V: NotFound + 'static,
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
    U: GetPath,
    V: NotFound + 'static,
    W: 'static;

impl<U, V, W> Service for CompositeService<U, V, W>
where
    U: GetPath,
    V: NotFound + 'static,
    W: 'static,
{
    type Request = U;
    type Response = V;
    type Error = W;
    type Future = Box<Future<Item = V, Error = W>>;

    fn call(&self, req: Self::Request) -> Self::Future {

        let mut result = None;

        for &(base_path, ref service) in &self.0 {
            if req.path().starts_with(base_path) {
                result = Some(service.call(req));
                break;
            }
        }

        result.unwrap_or_else(|| Box::new(future::ok(V::not_found())))
    }
}

impl<U, V, W> fmt::Debug for CompositeService<U, V, W>
where
    U: GetPath,
    V: NotFound + 'static,
    W: 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // Get vector of base paths
        let str_vec: Vec<&'static str> = self.0.iter().map(|&(base_path, _)| base_path).collect();
        write!(
            f,
            "CompositeService accepting base paths: {:?}",
            str_vec,
        )
    }
}

impl<U, V, W> Deref for CompositeService<U, V, W>
where
    U: GetPath,
    V: NotFound + 'static,
    W: 'static,
{
    type Target = Vec<(&'static str, BoxedService<U, V, W>)>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<U, V, W> DerefMut for CompositeService<U, V, W>
where
    U: GetPath,
    V: NotFound + 'static,
    W: 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
