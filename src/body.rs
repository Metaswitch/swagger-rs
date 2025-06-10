/// Helper methods for processing body
use bytes::Bytes;
use futures::stream::{Stream, StreamExt};

/// Additional function for converting body stream into Vec<u8>
pub trait BodyExt {
    /// Raw body type
    type Raw;

    /// Error if we can't gather up the raw body
    type Error;

    /// Collect the body into a raw form
    fn into_raw(self) -> futures::future::BoxFuture<'static, Result<Self::Raw, Self::Error>>;
}

impl<T, E> BodyExt for T
where
    T: Stream<Item = Result<Bytes, E>> + Unpin + Send + 'static,
{
    type Raw = Vec<u8>;
    type Error = E;

    fn into_raw(mut self) -> futures::future::BoxFuture<'static, Result<Self::Raw, Self::Error>> {
        Box::pin(async {
            let mut raw = Vec::new();
            while let (Some(chunk), rest) = self.into_future().await {
                raw.extend_from_slice(&chunk?);
                self = rest;
            }
            Ok(raw)
        })
    }
}
