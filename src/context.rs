//! Module for API context management.

use hyper;
use auth::{Authorization, AuthData};
use std::marker::Sized;
extern crate slog;

/// Request context, both as received in a server handler or as sent in a
/// client request. When REST microservices are chained, the Context passes
/// data from the server API to any further HTTP requests.
#[derive(Clone, Debug, Default)]
pub struct Context {
    /// Tracking ID when passing a request to another microservice.
    pub x_span_id: XSpanIdString,

    /// Authorization data, filled in from middlewares.
    pub authorization: Option<Authorization>,
    /// Raw authentication data, for use in making HTTP requests as a client.
    pub auth_data: Option<AuthData>,
    logger: Option<slog::Logger>,
}

#[derive(Debug, Clone, Default)]
pub struct XSpanIdString(pub String);

pub trait Has<T> {
    fn set(&mut self, T);
    fn get(&self) -> &T;
    fn get_mut(&mut self) -> &mut T;
}

pub trait ExtendsWith {
    type Inner;
    type Ext;
    fn new(inner: Self::Inner, item: Self::Ext) -> Self;
    fn set(&mut self, Self::Ext);
    fn get(&self) -> &Self::Ext;
    fn get_mut(&mut self) -> &mut Self::Ext;
}

impl<S, T> Has<S> for (S, T) {
    fn set(&mut self, item: S) {
        self.0 = item;
    }
    fn get(&self) -> &S {
        &self.0
    }
    fn get_mut(&mut self) -> &mut S {
        &mut self.0
    }
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

// impl<C, D, S, T> Has<S> for D
// where
//     D: ExtendsWith<Inner = C, Ext = T>,
//     T: Has<S>,
// {
//     fn set(&mut self, item: S) {
//         Has::<T>::get_mut(&mut self).set(item);
//     }
//     fn get(&self) -> &S {
//         Has::<T>::get(&self).get()
//     }

//     fn get_mut(&mut self) -> &mut S {
//         Has::<T>::get_mut(&mut self).get_mut()
//     }
// }

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

macro_rules! new_context_type {
    ($context_name:ident, $($types:ty),+ ) => {
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

new_context_type!(ContextExtension, String, u32, bool);



/// Trait for retrieving a logger from a struct.
pub trait HasLogger {
    /// Retrieve the context logger
    fn logger(&self) -> &Option<slog::Logger>;

    /// Set the context logger
    fn set_logger(&mut self, logger: slog::Logger);
}

impl HasLogger for Context {
    fn logger(&self) -> &Option<slog::Logger> {
        &self.logger
    }

    fn set_logger(&mut self, logger: slog::Logger) {
        self.logger = Some(logger);
    }
}

impl Has<XSpanIdString> for Context {
    fn set(&mut self, item: XSpanIdString) {
        self.x_span_id = item;
    }

    fn get(&self) -> &XSpanIdString {
        &self.x_span_id
    }

    fn get_mut(&mut self) -> &mut XSpanIdString {
        &mut self.x_span_id
    }
}

impl Context {
    /// Create a new, empty, `Context`.
    pub fn new() -> Context {
        Context::default()
    }

    /// Create a `Context` with a given span ID.
    pub fn new_with_span_id<S: Into<String>>(x_span_id: S) -> Context {
        Context {
            x_span_id: XSpanIdString(x_span_id.into()),
            ..Context::default()
        }
    }

    /// Set Basic authentication
    pub fn auth_basic(&mut self, username: &str, password: &str) {
        self.auth_data = Some(AuthData::Basic(hyper::header::Basic {
            username: username.to_owned(),
            password: Some(password.to_owned()),
        }));
    }

    /// Set Bearer token authentication
    pub fn auth_bearer(&mut self, token: &str) {
        self.auth_data = Some(AuthData::Bearer(
            hyper::header::Bearer { token: token.to_owned() },
        ));
    }

    /// Set ApiKey authentication
    pub fn auth_apikey(&mut self, apikey: &str) {
        self.auth_data = Some(AuthData::ApiKey(apikey.to_owned()));
    }
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
