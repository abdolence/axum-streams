use crate::stream_format::StreamingFormat;
use futures::Stream;
use futures_util::stream::BoxStream;
use futures_util::StreamExt;
use http::HeaderMap;

pub struct TextStreamFormat;

impl TextStreamFormat {
    pub fn new() -> Self {
        Self {}
    }
}

impl StreamingFormat<String> for TextStreamFormat {
    fn to_bytes_stream<'a, 'b>(
        &'a self,
        stream: BoxStream<'b, String>,
    ) -> BoxStream<'b, Result<axum::body::Bytes, axum::Error>> {
        fn write_text_record(obj: String) -> Result<Vec<u8>, axum::Error> {
            let obj_vec = obj.as_bytes().to_vec();
            Ok(obj_vec)
        }

        let stream_bytes: BoxStream<Result<axum::body::Bytes, axum::Error>> = Box::pin({
            stream.map(move |obj| {
                let write_text_res = write_text_record(obj);
                write_text_res.map(axum::body::Bytes::from)
            })
        });

        Box::pin(stream_bytes)
    }

    fn http_response_trailers(&self) -> Option<HeaderMap> {
        let mut header_map = HeaderMap::new();
        header_map.insert(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        Some(header_map)
    }
}

impl<'a> crate::StreamBodyAs<'a> {
    pub fn text<S>(stream: S) -> Self
    where
        S: Stream<Item = String> + 'a + Send,
    {
        Self::new(TextStreamFormat::new(), stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_client::*;
    use crate::StreamBodyAs;
    use axum::{routing::*, Router};
    use futures_util::stream;

    #[tokio::test]
    async fn serialize_text_stream_format() {
        #[derive(Clone, prost::Message)]
        struct TestOutputStructure {
            #[prost(string, tag = "1")]
            foo1: String,
            #[prost(string, tag = "2")]
            foo2: String,
        }

        let test_stream_vec = vec![
            String::from("bar1"),
            String::from("bar2"),
            String::from("bar3"),
            String::from("bar4"),
            String::from("bar5"),
            String::from("bar6"),
            String::from("bar7"),
            String::from("bar8"),
            String::from("bar9"),
        ];

        let test_stream = Box::pin(stream::iter(test_stream_vec.clone()));

        let app = Router::new().route(
            "/",
            get(|| async { StreamBodyAs::new(TextStreamFormat::new(), test_stream) }),
        );

        let client = TestClient::new(app);

        let expected_text_buf: Vec<u8> = test_stream_vec
            .iter()
            .flat_map(|obj| {
                let obj_vec = obj.as_bytes().to_vec();
                obj_vec
            })
            .collect();

        let res = client.get("/").send().await.unwrap();
        let body = res.bytes().await.unwrap().to_vec();

        assert_eq!(body, expected_text_buf);
    }
}
