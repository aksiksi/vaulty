use std::future::Future;
use std::pin::Pin;

use bytes::Bytes;
use futures::stream::Stream;

use crate::storage::Error;

// Definition of future types for async use
pub type ClientFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'a>>;

pub trait Client {
    fn upload_stream(
        &self,
        path: &str,
        data: impl Stream<Item = Result<Bytes, crate::Error>> + Send + Sync + 'static,
    ) -> ClientFuture<'_, ()>;
}
