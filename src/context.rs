//! Module for API context management.

use hyper;
use auth::{Authorization, AuthData};

/// Request context, both as received in a server handler or as sent in a
/// client request. When REST microservices are chained, the Context passes
/// data from the server API to any further HTTP requests.
#[derive(Clone, Debug, Default)]
pub struct Context {
    /// Tracking ID when passing a request to another microservice.
    pub x_span_id: Option<String>,

    /// Authorization data, filled in from middlewares.
    pub authorization: Option<Authorization>,
    /// Raw authentication data, for use in making HTTP requests as a client.
    pub auth_data: Option<AuthData>,
}

impl Context {
    /// Create a new, empty, `Context`.
    pub fn new() -> Context {
        Context::default()
    }

    /// Create a `Context` with a given span ID.
    pub fn new_with_span_id<S: Into<String>>(x_span_id: S) -> Context {
        Context {
            x_span_id: Some(x_span_id.into()),
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
pub struct ContextWrapper<'a, T: 'a> {
    api: &'a T,
    context: Context,
}

impl<'a, T> ContextWrapper<'a, T> {
    /// Create a new ContextWrapper, binding the API and context.
    pub fn new(api: &'a T, context: Context) -> ContextWrapper<'a, T> {
        ContextWrapper { api, context }
    }

    /// Borrows the API.
    pub fn api(&self) -> &T {
        self.api
    }

    /// Borrows the context.
    pub fn context(&self) -> &Context {
        &self.context
    }
}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<'a>
where
    Self: Sized,
{
    /// Binds this API to a context.
    fn with_context(self: &'a Self, context: Context) -> ContextWrapper<'a, Self> {
        ContextWrapper::<Self>::new(self, context)
    }
}
