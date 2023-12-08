use crate::stream_format::StreamingFormat;
use axum::response::{IntoResponse, Response};
use futures::Stream;
use futures_util::stream::BoxStream;
use http::HeaderMap;
use std::fmt::Formatter;
use std::pin::Pin;
use std::task::{Context, Poll};
use axum::body::{Body, HttpBody};
use http_body::Frame;

pub struct StreamBodyAs<'a> {
    stream: BoxStream<'a, Result<Frame<axum::body::Bytes>, axum::Error>>,
    trailers: Option<HeaderMap>,
}

impl<'a> std::fmt::Debug for StreamBodyAs<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "StreamBodyWithFormat")
    }
}

impl<'a> StreamBodyAs<'a> {
    /// Create a new `StreamBodyWith` providing a stream of your objects in the specified format.
    pub fn new<S, T, FMT>(stream_format: FMT, stream: S) -> Self
    where
        FMT: StreamingFormat<T>,
        S: Stream<Item = T> + 'a + Send,
    {
        Self {
            stream: stream_format.to_bytes_stream(Box::pin(stream)),
            trailers: stream_format.http_response_trailers(),
        }
    }

    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.trailers = Some(headers);
        self
    }
}

impl IntoResponse for StreamBodyAs<'static> {
    fn into_response(mut self) -> Response {
        let headers = if let Some(trailers) = self.trailers.take() {
            trailers
        } else {
            HeaderMap::new()
        };

        let mut response: Response<Body> = Response::new(Body::new(self));
        *response.headers_mut() = headers;
        response
    }
}

impl<'a> HttpBody for StreamBodyAs<'a> {
    type Data = axum::body::Bytes;
    type Error = axum::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Pin::new(&mut self.stream).poll_next(cx)
    }

}
