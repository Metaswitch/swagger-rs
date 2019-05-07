/// Common trait for swagger based client middleware
pub trait Service {
    /// Request body taken by client.
    /// Likely either `hyper::Body`, `hyper::Chunk` or `swagger::ContextualPayload`.
    type ReqBody: hyper::body::Payload;

    /// Future response from client service.
    /// Likely: `Future<Item=hyper::Response<hyper::Body>, Error=hyper::Error>`
    type Future: futures::Future;

    /// Handle the given request
    fn request(&self, req: hyper::Request<Self::ReqBody>) -> Self::Future;
}

impl<C, B> Service for hyper::Client<C, B>
where
    B: hyper::body::Payload + Send + 'static,
    B::Data: Send,
    C: hyper::client::connect::Connect + Sync + 'static,
    C::Transport: 'static,
    C::Future: 'static,
{
    type ReqBody = B;
    type Future = hyper::client::ResponseFuture;

    fn request(&self, req: hyper::Request<Self::ReqBody>) -> Self::Future {
        hyper::Client::request(self, req)
    }
}

/// Factory trait for creating Services - swagger based client middleware
pub trait MakeService<Context> {
    /// Future response creating the service.
    /// Likely: `Future<Item=Service, Error=hyper::Error>`
    type Future: futures::Future;

    /// Handle the given request
    fn make_service(&self, ctx: Context) -> Self::Future;
}
