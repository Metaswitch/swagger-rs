use std::io;
use hyper::server::{Service, NewService};
use hyper::{Request, Response, StatusCode};
use futures::Future;
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
    ) -> Box<Service<Request = U, Response = V, Error = W, Future = Box<Future<Item = V, Error = W>>>>;
}

impl<T, U, V, W> BoxedNewService<U, V, W> for T
where
    T: NewService<Request = U, Response = V, Error = W>,
    T::Instance: Service<Future = Box<Future<Item = V, Error = W>>>
        + Sized
        + 'static,
{
    fn boxed_new_service(
        &self,
    ) -> Box<Service<Request = U, Response = V, Error = W, Future = Box<Future<Item = V, Error = W>>>> {
        let new_service = self.new_service().unwrap();
        Box::new(new_service)
    }
}

pub struct CompositeNewService<U: HasPath, V: HasNotFound, W>(Vec<(&'static str, Box<BoxedNewService<U, V, W>>)>);

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

pub struct CompositeService<U: HasPath, V: HasNotFound, W>(
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
);

impl<U: HasPath, V: HasNotFound, W> NewService for CompositeNewService<U, V, W> {
    type Request = U;
    type Response = V;
    type Error = W;
    type Instance = CompositeService<U, V, W>;

    fn new_service(&self) -> Result<Self::Instance, io::Error> {
        let vec = self.0
            .iter()
            .map(|&(base_path, ref value)| {
                (base_path, value.boxed_new_service())
            })
            .collect();

        Ok(CompositeService(vec))
    }
}

impl<U: HasPath, V: HasNotFound, W> Service for CompositeService<U, V, W> {
    type Request = U;
    type Response = V;
    type Error = W;
    type Future = Box<Future<Item = V, Error = W>>;

    fn call(&self, req: Self::Request) -> Self::Future {

        ((self.0)[0].1).call(req)
    }
}
