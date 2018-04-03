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
