//! Module for API context management.
//!
//! This module defines traits and structs that can be used  to manage
//! contextual data related to a request, as it is passed through a series of
//! hyper services.
//!
//! See the `context_tests` module below for examples of how to use.

use crate::auth::{AuthData, Authorization};
use crate::XSpanIdString;
use futures::future::Future;
use hyper;
use std::marker::Sized;

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
/// impl<C> hyper::service::Service for MyService<C>
///     where C: Has<MyItem> + Send + 'static
/// {
///     type ReqBody = ContextualPayload<hyper::Body, C>;
///     type ResBody = hyper::Body;
///     type Error = std::io::Error;
///     type Future = Box<dyn Future<Item=hyper::Response<Self::ResBody>, Error=Self::Error>>;
///     fn call(&mut self, req : hyper::Request<Self::ReqBody>) -> Self::Future {
///         let (head, body) = req.into_parts();
///         do_something_with_my_item(Has::<MyItem>::get(&body.context));
///         Box::new(ok(hyper::Response::new(hyper::Body::empty())))
///     }
/// }
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
/// impl<T, C, D, E> hyper::service::Service for MiddlewareService<T, C>
///     where
///         C: Pop<MyItem1, Result=D> + Send + 'static,
///         D: Pop<MyItem2, Result=E>,
///         E: Pop<MyItem3>,
///         E::Result: Send + 'static,
///         T: hyper::service::Service<ReqBody=ContextualPayload<hyper::Body, E::Result>>
/// {
///     type ReqBody = ContextualPayload<hyper::Body, C>;
///     type ResBody = T::ResBody;
///     type Error = T::Error;
///     type Future = T::Future;
///     fn call(&mut self, req : hyper::Request<Self::ReqBody>) -> Self::Future {
///         let (head, body) = req.into_parts();
///         let context = body.context;
///
///         // type annotations optional, included for illustrative purposes
///         let (_, context): (MyItem1, D) = context.pop();
///         let (_, context): (MyItem2, E) = context.pop();
///         let (_, context): (MyItem3, E::Result) = context.pop();
///
///         let req = hyper::Request::from_parts(head, ContextualPayload { inner: body.inner, context });
///         self.inner.call(req)
///     }
/// }
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
/// impl<T, C, D, E> hyper::service::Service for MiddlewareService<T, C>
///     where
///         C: Push<MyItem1, Result=D> + Send + 'static,
///         D: Push<MyItem2, Result=E>,
///         E: Push<MyItem3>,
///         E::Result: Send + 'static,
///         T: hyper::service::Service<ReqBody=ContextualPayload<hyper::Body, E::Result>>
/// {
///     type ReqBody = ContextualPayload<hyper::Body, C>;
///     type ResBody = T::ResBody;
///     type Error = T::Error;
///     type Future = T::Future;
///     fn call(&mut self, req : hyper::Request<Self::ReqBody>) -> Self::Future {
///         let (head, body) = req.into_parts();
///         let context = body.context
///             .push(MyItem1{})
///             .push(MyItem2{})
///             .push(MyItem3{});
///         let req = hyper::Request::from_parts(head, ContextualPayload { inner: body.inner, context });
///         self.inner.call(req)
///     }
/// }
pub trait Push<T> {
    /// The type that results from adding an item.
    type Result;
    /// Inserts a value.
    fn push(self, value: T) -> Self::Result;
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
/// for all types `T` that appear in the list passed to the macro invocation.
///
/// E.g.
///
/// ```edition2018
/// #[derive(Default)]
/// struct MyType1;
/// #[derive(Default)]
/// struct MyType2;
/// #[derive(Default)]
/// struct MyType3;
/// #[derive(Default)]
/// struct MyType4;
///
/// swagger::new_context_type!(MyContext, MyEmpContext, MyType1, MyType2, MyType3);
///
/// fn use_has_my_type_1<T: swagger::Has<MyType1>> (_: &T) {}
/// fn use_has_my_type_2<T: swagger::Has<MyType2>> (_: &T) {}
/// fn use_has_my_type_3<T: swagger::Has<MyType3>> (_: &T) {}
/// fn use_has_my_type_4<T: swagger::Has<MyType4>> (_: &T) {}
///
/// // Will implement `Has<MyType1>` and `Has<MyType2>` because these appear
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
///     # use swagger::Push as _;
///     let context : ExampleContext =
///         MyEmpContext::default()
///             .push(MyType2{})
///             .push(MyType1{});
///
///     use_has_my_type_1(&context);
///     use_has_my_type_2(&context);
///     // use_has_my_type_3(&context);      // will fail
///
///     // Will fail because `MyType4`// was not passed to `new_context_type!`
///     // let context = MyEmpContext::default().push(MyType4{});
///
///     let bad_context: BadContext = BadContext::default();
///     // use_has_my_type_4(&bad_context);  // will fail
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

        // implement `Push<T>` on the empty context type for each type `T` that
        // was passed to the macro
        $(
        impl $crate::Push<$types> for $empty_context_name {
            type Result = $context_name<$types, Self>;
            fn push(self, item: $types) -> Self::Result {
                $context_name{head: item, tail: Self::default()}
            }
        }

        // implement `Has<T>` for a list where `T` is the type of the head
        impl<C> $crate::Has<$types> for $context_name<$types, C> {
            fn set(&mut self, item: $types) {
                self.head = item;
            }

            fn get(&self) -> &$types {
                &self.head
            }

            fn get_mut(&mut self) -> &mut $types {
                &mut self.head
            }
        }

        // implement `Pop<T>` for a list where `T` is the type of the head
        impl<C> $crate::Pop<$types> for $context_name<$types, C> {
            type Result = C;
            fn pop(self) -> ($types, Self::Result) {
                (self.head, self.tail)
            }
        }

        // implement `Push<U>` for non-empty lists, for each type `U` that was passed
        // to the macro
        impl<C, T> $crate::Push<$types> for $context_name<T, C> {
            type Result = $context_name<$types, Self>;
            fn push(self, item: $types) -> Self::Result {
                $context_name{head: item, tail: self}
            }
        }
        )+

        // Add implementations of `Has<T>` and `Pop<T>` when `T` is any type stored in
        // the list, not just the head.
        $crate::new_context_type!(impl extend_has $context_name, $empty_context_name, $($types),+);
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

        $crate::new_context_type!(
            impl extend_has_helper
            $context_name,
            $empty_context_name,
            $head,
            $($tail),+
        );
        $crate::new_context_type!(impl extend_has $context_name, $empty_context_name, $($tail),+);
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

            impl<C> $crate::Pop<$type> for $context_name<$types, C> where C: $crate::Pop<$type> {
                type Result = $context_name<$types, C::Result>;
                fn pop(self) -> ($type, Self::Result) {
                    let (value, tail) = self.tail.pop();
                    (value, $context_name{ head: self.head, tail})
                }
            }

            impl<C> $crate::Pop<$types> for $context_name<$type, C> where C: $crate::Pop<$types> {
                type Result = $context_name<$type, C::Result>;
                fn pop(self) -> ($types, Self::Result) {
                    let (value, tail) = self.tail.pop();
                    (value, $context_name{ head: self.head, tail})
                }
            }
        )+
    };
}

// Create a default context type to export.
new_context_type!(
    ContextBuilder,
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
/// ```
#[macro_export]
macro_rules! make_context_ty {
    ($context_name:ident, $empty_context_name:ident, $type:ty $(, $types:ty)* $(,)* ) => {
        $context_name<$type, $crate::make_context_ty!($context_name, $empty_context_name, $($types),*)>
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
        $crate::make_context!($context_name, $empty_context_name, $($values),*).push($value)
    };
    ($context_name:ident, $empty_context_name:ident $(,)* ) => {
        $empty_context_name::default()
    };
}

/// Context wrapper, to bind an API with a context.
#[derive(Debug)]
pub struct ContextWrapper<'a, T, C> {
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

impl<'a, T, C: Clone> Clone for ContextWrapper<'a, T, C> {
    fn clone(&self) -> Self {
        ContextWrapper {
            api: self.api,
            context: self.context.clone(),
        }
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

/// Trait designed to ensure consistency in context used by swagger middlewares
///
/// ```rust
/// # extern crate hyper;
/// # extern crate swagger;
/// # use swagger::context::*;
/// # use std::marker::PhantomData;
/// # use swagger::auth::{AuthData, Authorization};
/// # use swagger::XSpanIdString;
///
/// struct ExampleMiddleware<T, C> {
///     inner: T,
///     marker: PhantomData<C>,
/// }
///
/// impl<T, C> hyper::service::Service for ExampleMiddleware<T, C>
///     where
///         T: SwaggerService<C>,
///         C: Has<Option<AuthData>> +
///            Has<Option<Authorization>> +
///            Has<XSpanIdString> +
///            Clone +
///            Send +
///            'static,
/// {
///     type ReqBody = ContextualPayload<hyper::Body, C>;
///     type ResBody = T::ResBody;
///     type Error = T::Error;
///     type Future = T::Future;
///     fn call(&mut self, req: hyper::Request<Self::ReqBody>) -> Self::Future {
///         self.inner.call(req)
///     }
/// }
/// ```
pub trait SwaggerService<C>:
    Clone
    + hyper::service::Service<
        ReqBody = ContextualPayload<hyper::Body, C>,
        ResBody = hyper::Body,
        Error = hyper::Error,
        Future = Box<dyn Future<Item = hyper::Response<hyper::Body>, Error = hyper::Error> + Send>,
    >
where
    C: Has<Option<AuthData>>
        + Has<Option<Authorization>>
        + Has<XSpanIdString>
        + Clone
        + 'static
        + Send,
{
}

impl<T, C> SwaggerService<C> for T
where
    T: Clone
        + hyper::service::Service<
            ReqBody = ContextualPayload<hyper::Body, C>,
            ResBody = hyper::Body,
            Error = hyper::Error,
            Future = Box<
                dyn Future<Item = hyper::Response<hyper::Body>, Error = hyper::Error> + Send,
            >,
        >,
    C: Has<Option<AuthData>>
        + Has<Option<Authorization>>
        + Has<XSpanIdString>
        + Clone
        + 'static
        + Send,
{
}

/// This represents context provided as part of the request or the response
#[derive(Clone, Debug)]
pub struct ContextualPayload<P, Ctx>
where
    P: hyper::body::Payload,
    Ctx: Send + 'static,
{
    /// The inner payload for this request/response
    pub inner: P,
    /// Request or Response Context
    pub context: Ctx,
}

impl<P, Ctx> hyper::body::Payload for ContextualPayload<P, Ctx>
where
    P: hyper::body::Payload,
    Ctx: Send + 'static,
{
    type Data = P::Data;
    type Error = P::Error;

    fn poll_data(&mut self) -> futures::Poll<Option<Self::Data>, Self::Error> {
        self.inner.poll_data()
    }
}

#[cfg(test)]
mod context_tests {
    use super::*;
    use futures::future::{ok, Future, FutureResult};
    use hyper::service::{MakeService, Service};
    use hyper::{Body, Error, Method, Request, Response, Uri};
    use std::io;
    use std::marker::PhantomData;
    use std::str::FromStr;

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
        C: Has<ContextItem2> + Pop<ContextItem3> + Send + 'static,
    {
        type ReqBody = ContextualPayload<Body, C>;
        type ResBody = Body;
        type Error = Error;
        type Future = Box<dyn Future<Item = Response<Body>, Error = Error>>;
        fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
            use_item_2(Has::<ContextItem2>::get(&req.body().context));

            let (_, body) = req.into_parts();

            let (item3, _): (ContextItem3, _) = body.context.pop();
            use_item_3_owned(item3);

            Box::new(ok(Response::new(Body::empty())))
        }
    }

    struct InnerMakeService<RC>
    where
        RC: Has<ContextItem2> + Pop<ContextItem3>,
    {
        marker: PhantomData<RC>,
    }

    impl<RC> InnerMakeService<RC>
    where
        RC: Has<ContextItem2> + Pop<ContextItem3>,
    {
        fn new() -> Self {
            InnerMakeService {
                marker: PhantomData,
            }
        }
    }

    impl<RC, SC> MakeService<SC> for InnerMakeService<RC>
    where
        RC: Has<ContextItem2> + Pop<ContextItem3> + Send + 'static,
    {
        type ReqBody = ContextualPayload<Body, RC>;
        type ResBody = Body;
        type Error = Error;
        type Service = InnerService<RC>;
        type Future = FutureResult<Self::Service, Self::MakeError>;
        type MakeError = io::Error;

        fn make_service(&mut self, _: SC) -> FutureResult<Self::Service, io::Error> {
            ok(InnerService {
                marker: PhantomData,
            })
        }
    }

    // Example of a middleware service using contexts, i.e. a hyper service that
    // processes a request (and its context) and passes it on to another wrapped
    // service.
    struct MiddleService<T, RC>
    where
        RC: Pop<ContextItem1>,
        RC::Result: Push<ContextItem2>,
        <RC::Result as Push<ContextItem2>>::Result: Push<ContextItem3>,
        <<RC::Result as Push<ContextItem2>>::Result as Push<ContextItem3>>::Result: Send + 'static,
        T: Service<
            ReqBody = ContextualPayload<
                Body,
                <<RC::Result as Push<ContextItem2>>::Result as Push<ContextItem3>>::Result,
            >,
        >,
    {
        inner: T,
        marker1: PhantomData<RC>,
    }

    // Use trait bounds to indicate what modifications your service will make
    // to the context, chaining them as below.
    impl<T, C, D, E> Service for MiddleService<T, C>
    where
        C: Pop<ContextItem1, Result = D> + Send + 'static,
        D: Push<ContextItem2, Result = E>,
        E: Push<ContextItem3>,
        T: Service<ReqBody = ContextualPayload<Body, E::Result>>,
        E::Result: Send + 'static,
    {
        type ReqBody = ContextualPayload<Body, C>;
        type ResBody = T::ResBody;
        type Error = T::Error;
        type Future = T::Future;
        fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
            let (head, body) = req.into_parts();
            let (item, context) = body.context.pop();
            use_item_1_owned(item);
            let context = context.push(ContextItem2 {}).push(ContextItem3 {});
            let req = Request::from_parts(
                head,
                ContextualPayload {
                    inner: body.inner,
                    context,
                },
            );
            self.inner.call(req)
        }
    }

    struct MiddleMakeService<T, SC, RC>
    where
        RC: Pop<ContextItem1>,
        RC::Result: Push<ContextItem2>,
        <RC::Result as Push<ContextItem2>>::Result: Push<ContextItem3>,
        <<RC::Result as Push<ContextItem2>>::Result as Push<ContextItem3>>::Result: Send + 'static,
        T: MakeService<
            SC,
            ReqBody = ContextualPayload<
                Body,
                <<RC::Result as Push<ContextItem2>>::Result as Push<ContextItem3>>::Result,
            >,
        >,
    {
        inner: T,
        marker1: PhantomData<RC>,
        marker2: PhantomData<SC>,
    }

    impl<T, SC, RC, D, E> MakeService<SC> for MiddleMakeService<T, SC, RC>
    where
        RC: Pop<ContextItem1, Result = D> + Send + 'static,
        D: Push<ContextItem2, Result = E>,
        E: Push<ContextItem3>,
        T: MakeService<SC, ReqBody = ContextualPayload<Body, E::Result>>,
        T::Future: 'static,
        E::Result: Send + 'static,
    {
        type ReqBody = ContextualPayload<Body, RC>;
        type ResBody = T::ResBody;
        type Error = T::Error;
        type Service = MiddleService<T::Service, RC>;
        type Future = Box<dyn Future<Item = Self::Service, Error = Self::MakeError>>;
        type MakeError = T::MakeError;

        fn make_service(&mut self, sc: SC) -> Self::Future {
            Box::new(self.inner.make_service(sc).map(|s| MiddleService {
                inner: s,
                marker1: PhantomData,
            }))
        }
    }

    impl<T, SC, RC, D, E> MiddleMakeService<T, SC, RC>
    where
        RC: Pop<ContextItem1, Result = D>,
        D: Push<ContextItem2, Result = E>,
        E: Push<ContextItem3>,
        T: MakeService<SC, ReqBody = ContextualPayload<Body, E::Result>>,
        E::Result: Send + 'static,
    {
        fn new(inner: T) -> Self {
            MiddleMakeService {
                inner,
                marker1: PhantomData,
                marker2: PhantomData,
            }
        }
    }

    // Example of a top layer service that creates a context to be used by
    // lower layers.
    struct OuterService<T, C>
    where
        C: Default + Push<ContextItem1>,
        T: Service<ReqBody = ContextualPayload<Body, C::Result>>,
        C::Result: Send + 'static,
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
        T: Service<ReqBody = ContextualPayload<Body, C::Result>>,
        C::Result: Send + 'static,
    {
        type ReqBody = Body;
        type ResBody = T::ResBody;
        type Error = T::Error;
        type Future = T::Future;
        fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
            let context = C::default().push(ContextItem1 {});
            let (header, body) = req.into_parts();
            let req = Request::from_parts(
                header,
                ContextualPayload {
                    inner: body,
                    context,
                },
            );
            self.inner.call(req)
        }
    }

    struct OuterMakeService<T, SC, RC>
    where
        RC: Default + Push<ContextItem1>,
        T: MakeService<SC, ReqBody = ContextualPayload<Body, RC::Result>>,
        RC::Result: Send + 'static,
    {
        inner: T,
        marker1: PhantomData<RC>,
        marker2: PhantomData<SC>,
    }

    impl<T, SC, RC> MakeService<SC> for OuterMakeService<T, SC, RC>
    where
        RC: Default + Push<ContextItem1>,
        RC::Result: Send + 'static,
        T: MakeService<SC, ReqBody = ContextualPayload<Body, RC::Result>>,
        T::Future: 'static,
    {
        type ReqBody = Body;
        type ResBody = T::ResBody;
        type Error = T::Error;
        type Service = OuterService<T::Service, RC>;
        type Future = Box<dyn Future<Item = Self::Service, Error = Self::MakeError>>;
        type MakeError = T::MakeError;

        fn make_service(&mut self, sc: SC) -> Self::Future {
            Box::new(self.inner.make_service(sc).map(|s| OuterService {
                inner: s,
                marker: PhantomData,
            }))
        }
    }

    impl<T, SC, RC> OuterMakeService<T, SC, RC>
    where
        RC: Default + Push<ContextItem1>,
        RC::Result: Send + 'static,
        T: MakeService<SC, ReqBody = ContextualPayload<Body, RC::Result>>,
    {
        fn new(inner: T) -> Self {
            OuterMakeService {
                inner,
                marker1: PhantomData,
                marker2: PhantomData,
            }
        }
    }

    // Example of use by a service in its main.rs file. At this point you know
    // all the hyper service layers you will be using, and what requirements
    // their contexts types have. Use the `new_context_type!` macro to create
    // a context type and empty context type that are capable of containing all the
    // types that your hyper services require.
    new_context_type!(
        MyContext,
        MyEmptyContext,
        ContextItem1,
        ContextItem2,
        ContextItem3
    );

    #[test]
    fn send_request() {
        // annotate the outermost service to indicate that the context type it
        // uses is the empty context type created by the above macro invocation.
        // the compiler should infer all the other context types.
        let mut make_service = OuterMakeService::<_, _, MyEmptyContext>::new(
            MiddleMakeService::new(InnerMakeService::new()),
        );

        let req = Request::builder()
            .method(Method::POST)
            .uri(Uri::from_str("127.0.0.1:80").unwrap())
            .body(Body::empty());

        make_service
            .make_service(())
            .wait()
            .expect("Failed to start new service")
            .call(req.unwrap())
            .wait()
            .expect("Service::call returned an error");
    }
}
