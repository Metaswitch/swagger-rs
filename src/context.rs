//! Module for API context management.
//!
//! This module defines traits and structs that can be used  to manage
//! contextual data related to a request, as it is passed through a series of
//! hyper services.
//!
//! See the `context_tests` module below for examples of how to use.

use auth::{Authorization, AuthData};
use std::marker::Sized;
use super::XSpanIdString;

/// Defines methods for accessing, modifying, adding and removing the data stored
/// in a context. Used to specify the requirements that a hyper service makes on
/// a generic context type that it receives with a request, e.g.
///
/// ```rust
/// # extern crate hyper;
/// # extern crate swagger;
/// # extern crate futures;
/// #
/// # use swagger::context::*;
/// # use futures::future::{Future, ok};
/// # use std::marker::PhantomData;
/// #
/// # struct MyItem;
/// # fn do_something_with_my_item(item: &MyItem) {}
/// #
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
///     type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;
///     fn call(&self, (req, context) : Self::Request) -> Self::Future {
///         do_something_with_my_item(Has::<MyItem>::get(&context));
///         Box::new(ok(hyper::Response::new()))
///     }
/// }
///
/// # fn main() {}
/// ```
pub trait Has<T> {
    /// Get an immutable reference to the value.
    fn get(&self) -> &T;
    /// Get a mutable reference to the value.
    fn get_mut(&mut self) -> &mut T;
    /// Set the value.
    fn set(&mut self, value: T);
}

/// Defines a method for permanently extracting a value, changing the resulting
/// type. Used to specify that a hyper service consumes some data from the context,
/// making it unavailable to later layers, e.g.
///
/// ```rust
/// # extern crate hyper;
/// # extern crate swagger;
/// # extern crate futures;
/// #
/// # use swagger::context::*;
/// # use futures::future::{Future, ok};
/// # use std::marker::PhantomData;
/// #
/// struct MyItem1;
/// struct MyItem2;
/// struct MyItem3;
///
/// struct MiddlewareService<T, C> {
///     inner: T,
///     marker: PhantomData<C>,
/// }
///
/// impl<T, C, D, E> hyper::server::Service for MiddlewareService<T, C>
///     where
///         C: Pop<MyItem1, Result=D>,
///         D: Pop<MyItem2, Result=E>,
///         E: Pop<MyItem3>,
///         T: hyper::server::Service<Request = (hyper::Request, E::Result)>
/// {
///     type Request = (hyper::Request, C);
///     type Response = T::Response;
///     type Error = T::Error;
///     type Future = T::Future;
///     fn call(&self, (req, context) : Self::Request) -> Self::Future {
///
///         // type annotations optional, included for illustrative purposes
///         let (_, context): (MyItem1, D) = context.pop();
///         let (_, context): (MyItem2, E) = context.pop();
///         let (_, context): (MyItem3, E::Result) = context.pop();
///
///         self.inner.call((req, context))
///     }
/// }
///
/// # fn main() {}
pub trait Pop<T> {
    /// The type that remains after the value has been popped.
    type Result;
    /// Extracts a value.
    fn pop(self) -> (T, Self::Result);
}

/// Defines a method for inserting a value, changing the resulting
/// type. Used to specify that a hyper service adds some data from the context,
/// making it available to later layers, e.g.
///
/// ```rust
/// # extern crate hyper;
/// # extern crate swagger;
/// # extern crate futures;
/// #
/// # use swagger::context::*;
/// # use futures::future::{Future, ok};
/// # use std::marker::PhantomData;
/// #
/// struct MyItem1;
/// struct MyItem2;
/// struct MyItem3;
///
/// struct MiddlewareService<T, C> {
///     inner: T,
///     marker: PhantomData<C>,
/// }
///
/// impl<T, C, D, E> hyper::server::Service for MiddlewareService<T, C>
///     where
///         C: Push<MyItem1, Result=D>,
///         D: Push<MyItem2, Result=E>,
///         E: Push<MyItem3>,
///         T: hyper::server::Service<Request = (hyper::Request, E::Result)>
/// {
///     type Request = (hyper::Request, C);
///     type Response = T::Response;
///     type Error = T::Error;
///     type Future = T::Future;
///     fn call(&self, (req, context) : Self::Request) -> Self::Future {
///         let context = context
///             .push(MyItem1{})
///             .push(MyItem2{})
///             .push(MyItem3{});
///         self.inner.call((req, context))
///     }
/// }
///
/// # fn main() {}
pub trait Push<T> {
    /// The type that results from adding an item.
    type Result;
    /// Inserts a value.
    fn push(self, T) -> Self::Result;
}

/// Defines a struct that can be used to build up contexts recursively by
/// adding one item to the context at a time, and a unit struct representing an
/// empty context. The first argument is the name of the newly defined context struct
/// that is used to add an item to the context, the second argument is the name of
/// the empty context struct, and subsequent arguments are the types
/// that can be stored in contexts built using these struct.
///
/// A cons list built using the generated context type will implement Has<T> and Pop<T>
/// for each type T that appears in the list, provided that the list only
/// contains the types that were passed to the macro invocation after the context
/// type name.
///
/// All list types constructed using the generated types will implement `Push<T>`
/// for all `T`, but it should ony be used when `T` is one of the types passed
/// to the macro invocation, otherwise it might not be possible to retrieve the
/// inserted value.
///
/// E.g.
///
/// ```rust
/// # #[macro_use] extern crate swagger;
/// # use swagger::{Has, Pop, Push};
///
/// struct MyType1;
/// struct MyType2;
/// struct MyType3;
/// struct MyType4;
///
/// new_context_type!(MyContext, MyEmpContext, MyType1, MyType2, MyType3);
///
/// fn use_has_my_type_1<T: Has<MyType1>> (_: &T) {}
/// fn use_has_my_type_2<T: Has<MyType2>> (_: &T) {}
/// fn use_has_my_type_3<T: Has<MyType3>> (_: &T) {}
/// fn use_has_my_type_4<T: Has<MyType4>> (_: &T) {}
///
/// // will implement `Has<MyType1>` and `Has<MyType2>` because these appear
/// // in the type, and were passed to `new_context_type!`. Will not implement
/// // `Has<MyType3>` even though it was passed to `new_context_type!`, because
/// // it is not included in the type.
/// type ExampleContext = MyContext<MyType1, MyContext<MyType2,  MyEmpContext>>;
///
/// // Will not implement `Has<MyType4>` even though it appears in the type,
/// // because `MyType4` was not passed to `new_context_type!`.
/// type BadContext = MyContext<MyType1, MyContext<MyType4, MyEmpContext>>;
///
/// fn main() {
///     let context : ExampleContext =
///         MyEmpContext::default()
///         .push(MyType2{})
///         .push(MyType1{});
///
///     use_has_my_type_1(&context);
///     use_has_my_type_2(&context);
///     // use_has_my_type3(&context);      // will fail
///
///     let bad_context: BadContext =
///         MyEmpContext::default()
///         .push(MyType4{})
///         .push(MyType1{});
///     // use_has_my_type_4(&bad_context);     // will fail
///
/// }
/// ```
///
/// See the `context_tests` module for more usage examples.
#[macro_export]
macro_rules! new_context_type {
    ($context_name:ident, $empty_context_name:ident, $($types:ty),+ ) => {

        /// Wrapper type for building up contexts recursively, adding one item
        /// to the context at a time.
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub struct $context_name<T, C> {
            head: T,
            tail: C,
        }

        /// Unit struct representing an empty context with no data in it.
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub struct $empty_context_name;

        // implement `Push<T>` on the empty context type for any `T`, so that
        // items can be added to the context
        impl<U> $crate::Push<U> for $empty_context_name {
            type Result = $context_name<U, Self>;
            fn push(self, item: U) -> Self::Result {
                $context_name{head: item, tail: Self::default()}
            }
        }

        // implement `Has<T>` for a list where `T` is the type of the head
        impl<T, C> $crate::Has<T> for $context_name<T, C> {
            fn set(&mut self, item: T) {
                self.head = item;
            }

            fn get(&self) -> &T {
                &self.head
            }

            fn get_mut(&mut self) -> &mut T {
                &mut self.head
            }
        }

        // implement `Pop<T>` for a list where `T` is the type of the head
        impl<T, C> $crate::Pop<T> for $context_name<T, C> {
            type Result = C;
            fn pop(self) -> (T, Self::Result) {
                (self.head, self.tail)
            }
        }

        // implement `Push<U>` for non-empty lists, for all types `U`
        impl<C, T, U> $crate::Push<U> for $context_name<T, C> {
            type Result = $context_name<U, Self>;
            fn push(self, item: U) -> Self::Result {
                $context_name{head: item, tail: self}
            }
        }

        // Add implementations of `Has<T>` and `Pop<T>` when `T` is any type stored in
        // the list, not just the head.
        new_context_type!(impl extend_has $context_name, $empty_context_name, $($types),+);
    };

    // "HELPER" MACRO CASE - NOT FOR EXTERNAL USE
    // takes a type `Type1` ($head) and a non-empty list of types `Types` ($tail). First calls
    // another helper macro to define the following impls, for each `Type2` in `Types`:
    // ```
    // impl<C: Has<Type1> Has<Type1> for $context_name<Type2, C> {...}
    // impl<C: Has<Type2> Has<Type2> for $context_name<Type1, C> {...}
    // impl<C: Pop<Type1> Pop<Type1> for $context_name<Type2, C> {...}
    // impl<C: Pop<Type2> Pop<Type2> for $context_name<Type1, C> {...}
    // ```
    // then calls itself again with the rest of the list. The end result is to define the above
    // impls for all distinct pairs of types in the original list.
    (impl extend_has $context_name:ident, $empty_context_name:ident, $head:ty, $($tail:ty),+ ) => {

        //
        new_context_type!(
            impl extend_has_helper
            $context_name,
            $empty_context_name,
            $head,
            $($tail),+
        );
        new_context_type!(impl extend_has $context_name, $empty_context_name, $($tail),+);
    };

    // "HELPER" MACRO CASE - NOT FOR EXTERNAL USE
    // base case of the preceding helper macro - was passed an empty list of types, so
    // we don't need to do anything.
    (impl extend_has $context_name:ident, $empty_context_name:ident, $head:ty) => {};

    // "HELPER" MACRO CASE - NOT FOR EXTERNAL USE
    // takes a type `Type1` ($type) and a non-empty list of types `Types` ($types). For
    // each `Type2` in `Types`, defines the following impls:
    // ```
    // impl<C: Has<Type1> Has<Type1> for $context_name<Type2, C> {...}
    // impl<C: Has<Type2> Has<Type2> for $context_name<Type1, C> {...}
    // impl<C: Pop<Type1> Pop<Type1> for $context_name<Type2, C> {...}
    // impl<C: Pop<Type2> Pop<Type2> for $context_name<Type1, C> {...}
    // ```
    //
    (impl extend_has_helper
        $context_name:ident,
        $empty_context_name:ident,
        $type:ty,
        $($types:ty),+
        ) => {
        $(
            impl<C: $crate::Has<$type>> $crate::Has<$type> for $context_name<$types, C> {
                fn set(&mut self, item: $type) {
                    self.tail.set(item);
                }

                fn get(&self) -> &$type {
                    self.tail.get()
                }

                fn get_mut(&mut self) -> &mut $type {
                    self.tail.get_mut()
                }
            }

            impl<C: $crate::Has<$types>> $crate::Has<$types> for $context_name<$type, C> {
                fn set(&mut self, item: $types) {
                    self.tail.set(item);
                }

                fn get(&self) -> &$types {
                    self.tail.get()
                }

                fn get_mut(&mut self) -> &mut $types {
                    self.tail.get_mut()
                }
            }

            impl<C> $crate::Pop<$type> for $context_name<$types, C> where C: Pop<$type> {
                type Result = $context_name<$types, C::Result>;
                fn pop(self) -> ($type, Self::Result) {
                    let (value, tail) = self.tail.pop();
                    (value, $context_name{ head: self.head, tail})
                }
            }

            impl<C> $crate::Pop<$types> for $context_name<$type, C> where C: Pop<$types> {
                type Result = $context_name<$type, C::Result>;
                fn pop(self) -> ($types, Self::Result) {
                    let (value, tail) = self.tail.pop();
                    (value, $context_name{ head: self.head, tail})
                }
            }
        )+
    };
}

/// Create a default context type to export.
new_context_type!(ContextBuilder,
                  EmptyContext,
                  XSpanIdString,
                  Option<AuthData>,
                  Option<Authorization>
);

/// Macro for easily defining context types. The first argument should be a
/// context type created with `new_context_type!` and subsequent arguments are the
/// types to be stored in the context, with the outermost first.
///
/// ```rust
/// # #[macro_use] extern crate swagger;
/// # use swagger::{Has, Pop, Push};
///
/// # struct Type1;
/// # struct Type2;
/// # struct Type3;
///
/// # new_context_type!(MyContext, MyEmptyContext, Type1, Type2, Type3);
///
/// // the following two types are identical
/// type ExampleContext1 = make_context_ty!(MyContext, MyEmptyContext, Type1, Type2, Type3);
/// type ExampleContext2 = MyContext<Type1, MyContext<Type2, MyContext<Type3, MyEmptyContext>>>;
///
/// // e.g. this wouldn't compile if they were different types
/// fn do_nothing(input: ExampleContext1) -> ExampleContext2 {
///     input
/// }
///
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! make_context_ty {
    ($context_name:ident, $empty_context_name:ident, $type:ty $(, $types:ty)* $(,)* ) => {
        $context_name<$type, make_context_ty!($context_name, $empty_context_name, $($types),*)>
    };
    ($context_name:ident, $empty_context_name:ident $(,)* ) => {
        $empty_context_name
    };
}

/// Macro for easily defining context values. The first argument should be a
/// context type created with `new_context_type!` and subsequent arguments are the
/// values to be stored in the context, with the outermost first.
///
/// ```rust
/// # #[macro_use] extern crate swagger;
/// # use swagger::{Has, Pop, Push};
///
/// # #[derive(PartialEq, Eq, Debug)]
/// # struct Type1;
/// # #[derive(PartialEq, Eq, Debug)]
/// # struct Type2;
/// # #[derive(PartialEq, Eq, Debug)]
/// # struct Type3;
///
/// # new_context_type!(MyContext, MyEmptyContext, Type1, Type2, Type3);
///
/// fn main() {
///     // the following are equivalent
///     let context1 = make_context!(MyContext, MyEmptyContext, Type1 {}, Type2 {}, Type3 {});
///     let context2 = MyEmptyContext::default()
///         .push(Type3{})
///         .push(Type2{})
///         .push(Type1{});
///
///     assert_eq!(context1, context2);
/// }
/// ```
#[macro_export]
macro_rules! make_context {
    ($context_name:ident, $empty_context_name:ident, $value:expr $(, $values:expr)* $(,)*) => {
        make_context!($context_name, $empty_context_name, $($values),*).push($value)
    };
    ($context_name:ident, $empty_context_name:ident $(,)* ) => {
        $empty_context_name::default()
    };
}

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
    struct ContextItem3;

    fn use_item_1_owned(_: ContextItem1) {}
    fn use_item_2(_: &ContextItem2) {}
    fn use_item_3_owned(_: ContextItem3) {}

    // Example of a "terminating" hyper service using contexts - i.e. doesn't
    // pass a request and its context on to a wrapped service.
    struct InnerService<C>
    where
        C: Has<ContextItem2> + Pop<ContextItem3>,
    {
        marker: PhantomData<C>,
    }

    // Use trait bounds to indicate what your service will use from the context.
    // use `Pop` if you want to take ownership of a value stored in the context,
    // or `Has` if a reference is enough.
    impl<C> Service for InnerService<C>
    where
        C: Has<ContextItem2> + Pop<ContextItem3>,
    {
        type Request = (Request, C);
        type Response = Response;
        type Error = Error;
        type Future = Box<Future<Item = Response, Error = Error>>;
        fn call(&self, (_, context): Self::Request) -> Self::Future {

            use_item_2(Has::<ContextItem2>::get(&context));

            let (item3, _): (ContextItem3, _) = context.pop();
            use_item_3_owned(item3);

            Box::new(ok(Response::new()))
        }
    }

    struct InnerNewService<C>
    where
        C: Has<ContextItem2> + Pop<ContextItem3>,
    {
        marker: PhantomData<C>,
    }

    impl<C> InnerNewService<C>
    where
        C: Has<ContextItem2> + Pop<ContextItem3>,
    {
        fn new() -> Self {
            InnerNewService { marker: PhantomData }
        }
    }

    impl<C> NewService for InnerNewService<C>
    where
        C: Has<ContextItem2> + Pop<ContextItem3>,
    {
        type Request = (Request, C);
        type Response = Response;
        type Error = Error;
        type Instance = InnerService<C>;
        fn new_service(&self) -> Result<Self::Instance, io::Error> {
            Ok(InnerService { marker: PhantomData })
        }
    }

    // Example of a middleware service using contexts, i.e. a hyper service that
    // processes a request (and its context) and passes it on to another wrapped
    // service.
    struct MiddleService<T, C>
    where
        C: Pop<ContextItem1>,
        C::Result: Push<ContextItem2>,
        <C::Result as Push<ContextItem2>>::Result: Push<ContextItem3>,
        T: Service<Request=(
            Request,
            <<C::Result as Push<ContextItem2>>::Result as Push<ContextItem3>>::Result
        )>,
    {
        inner: T,
        marker1: PhantomData<C>,
    }

    // Use trait bounds to indicate what modifications your service will make
    // to the context, chaining them as below.
    impl<T, C, D, E> Service for MiddleService<T, C>
    where
        C: Pop<ContextItem1, Result = D>,
        D: Push<ContextItem2, Result = E>,
        E: Push<ContextItem3>,
        T: Service<Request = (Request, E::Result)>,
    {
        type Request = (Request, C);
        type Response = T::Response;
        type Error = T::Error;
        type Future = T::Future;
        fn call(&self, (req, context): Self::Request) -> Self::Future {
            let (item, context) = context.pop();
            use_item_1_owned(item);
            let context = context.push(ContextItem2 {}).push(ContextItem3 {});
            self.inner.call((req, context))
        }
    }

    struct MiddleNewService<T, C>
    where
        C: Pop<ContextItem1>,
        C::Result: Push<ContextItem2>,
        <C::Result as Push<ContextItem2>>::Result: Push<ContextItem3>,
        T: NewService<Request=(
            Request,
            <<C::Result as Push<ContextItem2>>::Result as Push<ContextItem3>>::Result
        )>,
    {
        inner: T,
        marker1: PhantomData<C>,
    }

    impl<T, C, D, E> NewService for MiddleNewService<T, C>
    where
        C: Pop<ContextItem1, Result = D>,
        D: Push<ContextItem2, Result = E>,
        E: Push<ContextItem3>,
        T: NewService<Request = (Request, E::Result)>,
    {
        type Request = (Request, C);
        type Response = T::Response;
        type Error = T::Error;
        type Instance = MiddleService<T::Instance, C>;
        fn new_service(&self) -> Result<Self::Instance, io::Error> {
            self.inner.new_service().map(|s| {
                MiddleService {
                    inner: s,
                    marker1: PhantomData,
                }
            })
        }
    }

    impl<T, C, D, E> MiddleNewService<T, C>
    where
        C: Pop<ContextItem1, Result = D>,
        D: Push<ContextItem2, Result = E>,
        E: Push<ContextItem3>,
        T: NewService<Request = (Request, E::Result)>,
    {
        fn new(inner: T) -> Self {
            MiddleNewService {
                inner,
                marker1: PhantomData,
            }
        }
    }

    // Example of a top layer service that creates a context to be used by
    // lower layers.
    struct OuterService<T, C>
    where
        C: Default + Push<ContextItem1>,
        T: Service<Request = (Request, C::Result)>,
    {
        inner: T,
        marker: PhantomData<C>,
    }

    // Use a `Default` trait bound so that the context can be created. Use
    // `Push` trait bounds for each type that you will add to the newly
    // created context.
    impl<T, C> Service for OuterService<T, C>
    where
        C: Default + Push<ContextItem1>,
        T: Service<Request = (Request, C::Result)>,
    {
        type Request = Request;
        type Response = T::Response;
        type Error = T::Error;
        type Future = T::Future;
        fn call(&self, req: Self::Request) -> Self::Future {
            let context = C::default().push(ContextItem1 {});
            self.inner.call((req, context))
        }
    }

    struct OuterNewService<T, C>
    where
        C: Default + Push<ContextItem1>,
        T: NewService<Request = (Request, C::Result)>,
    {
        inner: T,
        marker: PhantomData<C>,
    }

    impl<T, C> NewService for OuterNewService<T, C>
    where
        C: Default + Push<ContextItem1>,
        T: NewService<Request = (Request, C::Result)>,
    {
        type Request = Request;
        type Response = T::Response;
        type Error = T::Error;
        type Instance = OuterService<T::Instance, C>;
        fn new_service(&self) -> Result<Self::Instance, io::Error> {
            self.inner.new_service().map(|s| {
                OuterService {
                    inner: s,
                    marker: PhantomData,
                }
            })
        }
    }

    impl<T, C> OuterNewService<T, C>
    where
        C: Default + Push<ContextItem1>,
        T: NewService<Request = (Request, C::Result)>,
    {
        fn new(inner: T) -> Self {
            OuterNewService {
                inner,
                marker: PhantomData,
            }
        }
    }

    // Example of use by a service in its main.rs file. At this point you know
    // all the hyper service layers you will be using, and what requirements
    // their contexts types have. Use the `new_context_type!` macro to create
    // a context type and empty context type that are capable of containing all the
    // types that your hyper services require.
    new_context_type!(MyContext, MyEmptyContext, ContextItem1, ContextItem2, ContextItem3);

    #[test]
    fn send_request() {

        // annotate the outermost service to indicate that the context type it
        // uses is the empty context type created by the above macro invocation.
        // the compiler should infer all the other context types.
        let new_service = OuterNewService::<_, MyEmptyContext>::new(
            MiddleNewService::new(InnerNewService::new()),
        );

        let req = Request::new(Method::Post, Uri::from_str("127.0.0.1:80").unwrap());
        new_service
            .new_service()
            .expect("Failed to start new service")
            .call(req)
            .wait()
            .expect("Service::call returned an error");
    }
}
