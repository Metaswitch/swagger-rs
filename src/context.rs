//! Module for API context management.

use auth::{Authorization, AuthData};
use std::marker::Sized;
use super::XSpanIdString;

/// Defines getters and setters for a value of a generic type.
pub trait Has<T> {
    /// Set the value.
    fn set(&mut self, T);
    /// Get an immutable reference to the value.
    fn get(&self) -> &T;
    /// Get a mutable reference to the value.
    fn get_mut(&mut self) -> &mut T;
}

/// Allows one type to act as an extension of another with an extra field added.
pub trait ExtendsWith {
    /// The type being extended.
    type Inner;

    /// The type of the field being added.
    type Ext;

    /// Create a new extended value.
    fn new(inner: Self::Inner, item: Self::Ext) -> Self;

    /// Set the added field.
    fn set(&mut self, Self::Ext);

    /// Get an immutable reference to the added field.
    fn get(&self) -> &Self::Ext;

    /// Get a mutable reference to the added field.
    fn get_mut(&mut self) -> &mut Self::Ext;
}

impl<C, D, T> Has<T> for D
where
    D: ExtendsWith<Inner = C, Ext = T>,
{
    fn set(&mut self, item: T) {
        ExtendsWith::set(self, item);
    }
    fn get(&self) -> &T {
        ExtendsWith::get(self)
    }
    fn get_mut(&mut self) -> &mut T {
        ExtendsWith::get_mut(self)
    }
}

macro_rules! extend_has_impls_helper {
    ($context_name:ident , $type:ty, $($types:ty),+ ) => {
        $(
            impl<C: Has<$type>> Has<$type> for $context_name<C, $types> {
                fn set(&mut self, item: $type) {
                    self.inner.set(item);
                }

                fn get(&self) -> &$type {
                    self.inner.get()
                }

                fn get_mut(&mut self) -> &mut $type {
                    self.inner.get_mut()
                }
            }

            impl<C: Has<$types>> Has<$types> for $context_name<C, $type> {
                fn set(&mut self, item: $types) {
                    self.inner.set(item);
                }

                fn get(&self) -> &$types {
                    self.inner.get()
                }

                fn get_mut(&mut self) -> &mut $types {
                    self.inner.get_mut()
                }
            }
        )+
    }
}

macro_rules! extend_has_impls {
    ($context_name:ident, $head:ty, $($tail:ty),+ ) => {
        extend_has_impls_helper!($context_name, $head, $($tail),+);
        extend_has_impls!($context_name, $($tail),+);
    };
    ($context_name:ident, $head:ty) => {};
}

#[macro_export]
macro_rules! new_context_type {
    ($context_name:ident, $($types:ty),+ ) => {

        /// Wrapper type for building up contexts recursively, adding one item
        /// to the context at a time.
        #[derive(Debug, Clone, Default)]
        pub struct $context_name<C, T> {
            inner: C,
            item: T,
        }

        impl<C, T> ExtendsWith for $context_name<C, T> {
            type Inner = C;
            type Ext = T;

            fn new(inner: C, item: T) -> Self {
                $context_name { inner, item }
            }

            fn set(&mut self, item: Self::Ext) {
                self.item = item;
            }

            fn get(&self) -> &Self::Ext {
                &self.item
            }

            fn get_mut(&mut self) -> &mut Self::Ext {
                &mut self.item
            }
        }

        extend_has_impls!($context_name, $($types),+);
    };

}

/// Create a default context type to export.
new_context_type!(Context, XSpanIdString, Option<AuthData>, Option<Authorization>);


/// Context wrapper, to bind an API with a context.
#[derive(Debug)]
pub struct ContextWrapper<'a, T: 'a, C> {
    api: &'a T,
    context: C,
}

impl<'a, T, C> ContextWrapper<'a, T, C> {
    /// Create a new ContextWrapper, binding the API and context.
    pub fn new(api: &'a T, context: C) -> ContextWrapper<'a, T, C> {
        ContextWrapper { api, context }
    }

    /// Borrows the API.
    pub fn api(&self) -> &T {
        self.api
    }

    /// Borrows the context.
    pub fn context(&self) -> &C {
        &self.context
    }
}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<'a, C>
where
    Self: Sized,
{
    /// Binds this API to a context.
    fn with_context(self: &'a Self, context: C) -> ContextWrapper<'a, Self, C> {
        ContextWrapper::<Self, C>::new(self, context)
    }
}


#[cfg(test)]
mod context_tests {
    use hyper::server::{NewService, Service};
    use hyper::{Response, Request, Error, Method, Uri};
    use std::marker::PhantomData;
    use std::io;
    use std::str::FromStr;
    use futures::future::{Future, ok};
    use super::*;

    struct ContextItem1;
    struct ContextItem2;

    fn do_something_with_item_1(_: &ContextItem1) {}
    fn do_something_with_item_2(_: &ContextItem2) {}

    struct InnerService<C>
        where C: Has<ContextItem1> + Has<ContextItem2>,
    {
        marker: PhantomData<C>,
    }

    impl<C> Service for InnerService<C>
        where C: Has<ContextItem1> + Has<ContextItem2>,
    {
        type Request = (Request, C);
        type Response = Response;
        type Error = Error;
        type Future = Box<Future<Item=Response, Error=Error>>;
        fn call(&self, (_, context): Self::Request) -> Self::Future {
            do_something_with_item_1(Has::<ContextItem1>::get(&context));
            do_something_with_item_2(Has::<ContextItem2>::get(&context));
            Box::new(ok(Response::new()))
        }
    }

    struct InnerNewService<C>
        where C: Has<ContextItem1> + Has<ContextItem2>,
    {
        marker: PhantomData<C>,
    }

    impl<C> InnerNewService<C>
        where C: Has<ContextItem1> + Has<ContextItem2>,
    {
        fn new() -> Self {
            InnerNewService {
                marker: PhantomData,
            }
        }
    }

    impl<C> NewService for InnerNewService<C>
        where C: Has<ContextItem1> + Has<ContextItem2>,
    {
        type Request = (Request, C);
        type Response = Response;
        type Error = Error;
        type Instance = InnerService<C>;
        fn new_service(&self) -> Result<Self::Instance, io::Error> {
            Ok(InnerService{marker: PhantomData})
        }
    }

    struct MiddleService<T, C, D>
        where
            T: Service<Request = (Request, D)>,
            C: Has<ContextItem1>,
            D: ExtendsWith<Inner=C, Ext=ContextItem2>,

    {
        inner: T,
        marker1: PhantomData<C>,
        marker2: PhantomData<D>,
    }

    impl<T, C, D> Service for MiddleService<T, C, D>
        where
            T: Service<Request = (Request, D)>,
            C: Has<ContextItem1>,
            D: ExtendsWith<Inner=C, Ext=ContextItem2>,
    {
        type Request = (Request, C);
        type Response = T::Response;
        type Error = T::Error;
        type Future = T::Future;
        fn call(&self, (req, context): Self::Request) -> Self::Future {
            do_something_with_item_1(Has::<ContextItem1>::get(&context));
            let context = D::new(context, ContextItem2{});
            self.inner.call((req, context))
        }
    }

    struct MiddleNewService<T, C, D>
        where
            T: NewService<Request = (Request, D)>,
            C: Has<ContextItem1>,
            D: ExtendsWith<Inner=C, Ext=ContextItem2>,
    {
        inner: T,
        marker1: PhantomData<C>,
        marker2: PhantomData<D>,
    }

    impl<T, C, D> NewService for MiddleNewService<T, C, D>
        where
            T: NewService<Request = (Request, D)>,
            C: Has<ContextItem1>,
            D: ExtendsWith<Inner=C, Ext=ContextItem2>,
    {
        type Request = (Request, C);
        type Response = T::Response;
        type Error = T::Error;
        type Instance = MiddleService<T::Instance, C, D>;
        fn new_service(&self) -> Result<Self::Instance, io::Error> {
            self.inner.new_service().map(|s| MiddleService{inner:s, marker1: PhantomData, marker2: PhantomData})
        }
    }

    impl<T, C, D> MiddleNewService<T, C, D>
        where
            T: NewService<Request = (Request, D)>,
            C: Has<ContextItem1>,
            D: ExtendsWith<Inner=C, Ext=ContextItem2>,
    {
        fn new(inner: T) -> Self {
            MiddleNewService {
                inner,
                marker1: PhantomData,
                marker2:PhantomData,
            }
        }
    }

    struct OuterService<T, C, D>
        where
            T: Service<Request = (Request, D)>,
            C: Default,
            D: ExtendsWith<Inner=C, Ext=ContextItem1>,

    {
        inner: T,
        marker1: PhantomData<C>,
        marker2: PhantomData<D>,
    }

    impl<T, C, D> Service for OuterService<T, C, D>
        where
            T: Service<Request = (Request, D)>,
            C: Default,
            D: ExtendsWith<Inner=C, Ext=ContextItem1>,
    {
        type Request = Request;
        type Response = T::Response;
        type Error = T::Error;
        type Future = T::Future;
        fn call(&self, req : Self::Request) -> Self::Future {
            let context = D::new(C::default(), ContextItem1 {} );
            self.inner.call((req, context))
        }
    }

    struct OuterNewService<T, C, D>
        where
            T: NewService<Request = (Request, D)>,
            C: Default,
            D: ExtendsWith<Inner=C, Ext=ContextItem1>,
    {
        inner: T,
        marker1: PhantomData<C>,
        marker2: PhantomData<D>,
    }

    impl<T, C, D> NewService for OuterNewService<T, C, D>
        where
            T: NewService<Request = (Request, D)>,
            C: Default,
            D: ExtendsWith<Inner=C, Ext=ContextItem1>,
    {
        type Request = Request;
        type Response = T::Response;
        type Error = T::Error;
        type Instance = OuterService<T::Instance, C, D>;
        fn new_service(&self) -> Result<Self::Instance, io::Error> {
            self.inner.new_service().map(|s| OuterService{inner:s, marker1: PhantomData, marker2: PhantomData})
        }
    }

    impl<T, C, D> OuterNewService<T, C, D>
        where
            T: NewService<Request = (Request, D)>,
            C: Default,
            D: ExtendsWith<Inner=C, Ext=ContextItem1>,
    {
        fn new(inner: T) -> Self {
            OuterNewService {
                inner,
                marker1: PhantomData,
                marker2:PhantomData,
            }
        }
    }

    new_context_type!(MyContext, ContextItem1, ContextItem2);

    type Context1 = MyContext<(), ContextItem1>;
    type Context2 = MyContext<Context1, ContextItem2>;

    type NewService1 = InnerNewService<Context2>;
    type NewService2 = MiddleNewService<NewService1, Context1, Context2>;
    type NewService3 = OuterNewService<NewService2, (), Context1>;

    #[test]
    fn send_request() {

        let new_service : NewService3 =
            OuterNewService::new(
                MiddleNewService::new(
                    InnerNewService::new()
                )
            );

        let req = Request::new(Method::Post, Uri::from_str("127.0.0.1:80").unwrap());
        new_service
            .new_service().expect("Failed to start new service")
            .call(req).wait().expect("Service::call returned an error");
    }
}
