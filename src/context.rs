//! Module for API context management.
//!
//! This module defines traits and structs that can be used  to manage
//! contextual data related to a request, as it is passed through a series of
//! hyper services.

use auth::{Authorization, AuthData};
use std::marker::Sized;
use super::XSpanIdString;

/// Defines getters and setters for a value of a generic type.
///
/// Used to specify the requirements that a hyper service makes on a generic
/// context type that it receives with a request, e.g.
///
/// ```rust
/// extern crate hyper;
///
/// use std::marker::PhantomData;
/// use context::*;
///
/// struct MyItem;
/// fn do_something_with_my_item(item: &MyItem) {};
///
/// struct MyService<C> {
///     marker: PhantomData<C>,
/// }
///
/// impl<C> hyper::server::Service for MyService<C>
///     where C: Has<MyItem>,
/// {
///     type Request = (hyper::Request, C);
///     type Response = hyper::Response;
///     type Error = hyper::Error;
///     type Future = Box<Future<Item=Response, Error=Error>>;
///     fn call(&self, (req, context) : Self::Request) -> Self::Future {
///         do_something_with_my_item(Has::<MyItem>::get(&context));
///         //...
///     }
/// }
/// ```
pub trait Has<T> {
    /// The type that is left after removing the T value.
    type Remainder;
    /// Set the value.
    fn set(&mut self, T);
    /// Get an immutable reference to the value.
    fn get(&self) -> &T;
    /// Get a mutable reference to the value.
    fn get_mut(&mut self) -> &mut T;
    /// Split into a the value and the remainder.
    fn deconstruct(self) -> (T, Self::Remainder);
    /// Constructor out of a value and remainder.
    fn construct(T, Self::Remainder) -> Self;
}

/// Defines a struct that can be used to build up contexts recursively by
/// adding one item to the context at a time. The first argument is the name
/// of the newly defined context struct, and subsequent arguments are the types
/// that can be stored in contexts built using this struct. That is,
///
/// ```rust
/// # #[macro_use] extern crate swagger;
/// struct MyType1;
/// struct MyType2;
/// struct MyType3;
/// new_context_type!(MyContext, MyType1, MyType2, MyType3);
/// # fn main() {
/// # }
/// ```
///
/// will define a new struct `MyContext<C, T>`, which implements:
/// - `Has<T>`,
/// - `ExtendsWith<Inner=C, Ext=T>`,
/// - `Has<S>` whenever `S` is one of `MyType1`, `MyType2` or `MyType3`, AND
///   `C` implements `Has<S>`.
///
/// See the `context_tests` module for more usage examples.
#[macro_export]
macro_rules! new_context_type {
    ($context_name:ident, $($types:ty),+ ) => {

        /// Wrapper type for building up contexts recursively, adding one item
        /// to the context at a time.
        #[derive(Debug, Clone, Default)]
        pub struct $context_name<T, C> {
            head: T,
            tail: C,
        }

        impl<T, C> Has<T> for $context_name<T, C> {
            type Remainder = C;

            fn set(&mut self, item: T) {
                self.head = item;
            }

            fn get(&self) -> &T {
                &self.head
            }

            fn get_mut(&mut self) -> &mut T {
                &mut self.head
            }

            fn deconstruct(self) -> (T, Self::Remainder){
                (self.head, self.tail)
            }

            fn construct(item: T, remainder: Self::Remainder) -> Self {
                $context_name{ head: item, tail: remainder}
            }
        }

        new_context_type!(impl extend_has $context_name, $($types),+);
    };
    (impl extend_has $context_name:ident, $head:ty, $($tail:ty),+ ) => {
        new_context_type!(impl extend_has_helper $context_name, $head, $($tail),+);
        new_context_type!(impl extend_has $context_name, $($tail),+);
    };
    (impl extend_has $context_name:ident, $head:ty) => {};
    (impl extend_has_helper $context_name:ident , $type:ty, $($types:ty),+ ) => {
        $(
            impl<C: $crate::Has<$type>> $crate::Has<$type> for $context_name<$types, C> {
                type Remainder = $context_name<$types, C::Remainder>;

                fn set(&mut self, item: $type) {
                    self.tail.set(item);
                }

                fn get(&self) -> &$type {
                    self.tail.get()
                }

                fn get_mut(&mut self) -> &mut $type {
                    self.tail.get_mut()
                }

                fn deconstruct(self) -> ($type, Self::Remainder) {
                    let (item, remainder) = self.tail.deconstruct();
                    (item, $context_name { head: self.head, tail: remainder})
                }

                fn construct(item: $type, remainder: Self::Remainder) -> Self {
                    $context_name { head: remainder.head, tail: C::construct(item, remainder.tail)}
                }
            }

            impl<C: $crate::Has<$types>> $crate::Has<$types> for $context_name<$type, C> {
                type Remainder = $context_name<$type, C::Remainder>;

                fn set(&mut self, item: $types) {
                    self.tail.set(item);
                }

                fn get(&self) -> &$types {
                    self.tail.get()
                }

                fn get_mut(&mut self) -> &mut $types {
                    self.tail.get_mut()
                }

                fn deconstruct(self) -> ($types, Self::Remainder) {
                    let (item, remainder) = self.tail.deconstruct();
                    (item, $context_name { head: self.head, tail: remainder})
                }

                fn construct(item: $types, remainder: Self::Remainder) -> Self {
                    $context_name { head: remainder.head, tail: C::construct(item, remainder.tail)}
                }
            }
        )+
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
            D: Has<ContextItem2, Remainder=C>,

    {
        inner: T,
        marker1: PhantomData<C>,
        marker2: PhantomData<D>,
    }

    impl<T, C, D> Service for MiddleService<T, C, D>
        where
            T: Service<Request = (Request, D)>,
            C: Has<ContextItem1>,
            D: Has<ContextItem2, Remainder=C>,
    {
        type Request = (Request, C);
        type Response = T::Response;
        type Error = T::Error;
        type Future = T::Future;
        fn call(&self, (req, context): Self::Request) -> Self::Future {
            do_something_with_item_1(Has::<ContextItem1>::get(&context));
            let context = D::construct(ContextItem2{}, context);
            self.inner.call((req, context))
        }
    }

    struct MiddleNewService<T, C, D>
        where
            T: NewService<Request = (Request, D)>,
            C: Has<ContextItem1>,
            D: Has<ContextItem2, Remainder=C>,
    {
        inner: T,
        marker1: PhantomData<C>,
        marker2: PhantomData<D>,
    }

    impl<T, C, D> NewService for MiddleNewService<T, C, D>
        where
            T: NewService<Request = (Request, D)>,
            C: Has<ContextItem1>,
            D: Has<ContextItem2, Remainder=C>,
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
            D: Has<ContextItem2, Remainder=C>,
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
            D: Has<ContextItem1, Remainder=C>,

    {
        inner: T,
        marker1: PhantomData<C>,
        marker2: PhantomData<D>,
    }

    impl<T, C, D> Service for OuterService<T, C, D>
        where
            T: Service<Request = (Request, D)>,
            C: Default,
            D: Has<ContextItem1, Remainder=C>,
    {
        type Request = Request;
        type Response = T::Response;
        type Error = T::Error;
        type Future = T::Future;
        fn call(&self, req : Self::Request) -> Self::Future {
            let context = D::construct(ContextItem1 {}, C::default() );
            self.inner.call((req, context))
        }
    }

    struct OuterNewService<T, C, D>
        where
            T: NewService<Request = (Request, D)>,
            C: Default,
            D: Has<ContextItem1, Remainder=C>,
    {
        inner: T,
        marker1: PhantomData<C>,
        marker2: PhantomData<D>,
    }

    impl<T, C, D> NewService for OuterNewService<T, C, D>
        where
            T: NewService<Request = (Request, D)>,
            C: Default,
            D: Has<ContextItem1, Remainder=C>,
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
            D: Has<ContextItem1, Remainder=C>,
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

    type Context1 = MyContext<ContextItem1, ()>;
    type Context2 = MyContext<ContextItem2, Context1>;

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
