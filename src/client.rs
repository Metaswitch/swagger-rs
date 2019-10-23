use async_trait::async_trait;
use crate::ContextualPayload;

pub type HyperResult = Result<hyper::Response<hyper::Body>, hyper::Error>;

/// Common trait for swagger based client middleware
#[async_trait]
pub trait Service<T> {
    /// Request body taken by client.
    /// Likely either `hyper::Body`, `hyper::Chunk` or `swagger::ContextualPayload`.
    type ReqBody: hyper::body::Payload;

    /// Handle the given request
    async fn request(&self, req: hyper::Request<Self::ReqBody>) -> T;
}

#[async_trait]
impl<C, B> Service<HyperResult> for hyper::Client<C, B>
where
    B: hyper::body::Payload + Unpin + Send + 'static,
    B::Data: Send + Unpin,
    C: hyper::client::connect::Connect + Sync + 'static,
    C::Transport: 'static,
    C::Future: 'static,
{
    type ReqBody = B;

    async fn request(&self, req: hyper::Request<Self::ReqBody>) -> HyperResult {
        hyper::Client::request(self, req).await
    }
}

/// Factory trait for creating Services - swagger based client middleware
#[async_trait]
pub trait MakeService<Context> {
    /// Service that this creates
    type Service: Service<ContextualPayload<hyper::Body, Context>>;

    /// Potential error from creating the service.
    type Error;

    /// Handle the given request
    async fn make_service(&self, ctx: Context) -> Result<Self::Service, Self::Error>;
}
