use std::{io, fmt};
use hyper::server::{Service, NewService};
use hyper::{Request, Response, StatusCode};
use futures::{future, Future};
use context::Context;

pub trait HasPath {
    fn path(&self) -> &str;
}

impl HasPath for Request {
    fn path(&self) -> &str {
        self.path()
    }
}

impl HasPath for (Request, Context) {
    fn path(&self) -> &str {
        self.0.path()
    }
}

pub trait HasNotFound {
    fn not_found() -> Self;
}

impl HasNotFound for Response {
    fn not_found() -> Self {
        Response::new().with_status(StatusCode::NotFound)
    }
}

pub trait BoxedNewService<U, V, W> {
    fn boxed_new_service(
        &self,
    ) -> Result<
        Box<
            Service<
                Request = U,
                Response = V,
                Error = W,
                Future = Box<Future<Item = V, Error = W>>,
            >,
        >,
        io::Error,
    >;
}

impl<T, U, V, W> BoxedNewService<U, V, W> for T
where
    T: NewService<Request = U, Response = V, Error = W>,
    T::Instance: Service<Future = Box<Future<Item = V, Error = W>>>
        + 'static,
{
    fn boxed_new_service(
        &self,
    ) -> Result<
        Box<
            Service<
                Request = U,
                Response = V,
                Error = W,
                Future = Box<Future<Item = V, Error = W>>,
            >,
        >,
        io::Error,
    > {
        let service = self.new_service()?;
        Ok(Box::new(service))
    }
}

pub struct CompositeNewService<U, V, W>(Vec<(&'static str, Box<BoxedNewService<U, V, W>>)>)
where
    U: HasPath,
    V: HasNotFound + 'static,
    W: 'static;

impl<U: HasPath, V: HasNotFound, W> CompositeNewService<U, V, W> {
    pub fn new() -> Self {
        CompositeNewService(Vec::new())
    }

    pub fn append_new_service(
        &mut self,
        base_path: &'static str,
        new_service: Box<BoxedNewService<U, V, W>>,
    ) {
        self.0.push((base_path, new_service));
    }
}

impl<U, V, W> NewService for CompositeNewService<U, V, W>
where
    U: HasPath,
    V: HasNotFound + 'static,
    W: 'static,
{
    type Request = U;
    type Response = V;
    type Error = W;
    type Instance = CompositeService<U, V, W>;

    fn new_service(&self) -> Result<Self::Instance, io::Error> {
        let mut vec = Vec::new();

        for &(base_path, ref new_service) in self.0.iter() {
            vec.push((base_path, new_service.boxed_new_service()?))
        }

        Ok(CompositeService(vec))
    }
}

impl<U, V, W> fmt::Debug for CompositeNewService<U, V, W>
where
    U: HasPath,
    V: HasNotFound + 'static,
    W: 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let str_vec: Vec<&'static str> = self.0.iter().map(|&(base_path, _)| base_path).collect();
        write!(
            f,
            "CompositeNewService accepting paths: {:?}",
            str_vec,
        )
    }
}

pub struct CompositeService<U, V, W>(
    Vec<
        (&'static str,
         Box<
            Service<
                Request = U,
                Response = V,
                Error = W,
                Future = Box<Future<Item = V, Error = W>>,
            >,
        >),
    >
)
where
    U: HasPath,
    V: HasNotFound + 'static,
    W: 'static;

impl<U, V, W> Service for CompositeService<U, V, W>
where
    U: HasPath,
    V: HasNotFound + 'static,
    W: 'static,
{
    type Request = U;
    type Response = V;
    type Error = W;
    type Future = Box<Future<Item = V, Error = W>>;

    fn call(&self, req: Self::Request) -> Self::Future {

        let mut result = None;

        for &(base_path, ref service) in self.0.iter() {
            if req.path().starts_with(base_path) {
                result = Some(service.call(req));
                break;
            }
        }

        if let Some(result) = result {
            result
        } else {
            Box::new(future::ok(V::not_found()))
        }
    }
}

impl<U, V, W> fmt::Debug for CompositeService<U, V, W>
where
    U: HasPath,
    V: HasNotFound + 'static,
    W: 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let str_vec: Vec<&'static str> = self.0.iter().map(|&(base_path, _)| base_path).collect();
        write!(
            f,
            "CompositeService accepting paths: {:?}",
            str_vec,
        )
    }
}
