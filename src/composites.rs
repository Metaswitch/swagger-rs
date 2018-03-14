
use std::collections::HashMap;
use hyper::server::{NewService, Service};
use futures::future::{Future, err};
use std::io::Error;

pub struct CompositeService{
    new_service_1 : NewService1;
    new_service_2 : NewService2;
}

impl Service for CompositeService {
    type Request = S;
    type Response = T;
    type Error = U;
    type Future = Box<Future<Item = T, Error = U>>;
    fn call(&self, req: Self::Request) -> Self::Future {
        self.new_services
            .get("base_path")
            .unwrap()
            .new_service()
            .unwrap()
            .call(req)
    }
}

struct NewService1 {}

struct Service1 {}

impl NewService for NewService1 {
    type Request = ();
    type Response = ();
    type Error = ();
    type Instance = Box<
        Service<
            Request = (),
            Response = (),
            Error = (),
            Future = Box<Future<Item = (), Error = ()>>,
        >,
    >;
    fn new_service(&self) -> Result<Self::Instance, Error> {
        Ok(Box::new(Service1 {}))
    }
}

impl Service for Service1 {
    type Request = ();
    type Response = ();
    type Error = ();
    type Future = Box<Future<Item = (), Error = ()>>;
    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(err(()))
    }
}

struct NewService2 {}

struct Service2 {}

impl NewService for NewService2 {
    type Request = ();
    type Response = ();
    type Error = ();
    type Instance = Box<
        Service<
            Request = (),
            Response = (),
            Error = (),
            Future = Box<Future<Item = (), Error = ()>>,
        >,
    >;
    fn new_service(&self) -> Result<Self::Instance, Error> {
        Ok(Box::new(Service2 {}))
    }
}

impl Service for Service2 {
    type Request = ();
    type Response = ();
    type Error = ();
    type Future = Box<Future<Item = (), Error = ()>>;
    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(err(()))
    }
}

fn test() {
    let mut hash_map: HashMap<
            &'static str,
            Box<
                NewService<
                    Request = (),
                    Response = (),
                    Error = (),
                    Instance = Box<
                        Service<
                            Request = (),
                            Response = (),
                            Error = (),
                            Future = Box<Future<Item = (), Error = ()>>,
                        >,
                    >,
                >,
            >,
        > = HashMap::new();
    hash_map.insert("service1", Box::new(NewService1 {}));
    hash_map.insert("service2", Box::new(NewService2 {}));
    let composite_service = CompositeService { new_services: hash_map };
    composite_service.call(());

}
