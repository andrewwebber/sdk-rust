use axum_lib as axum;

use std::cell::Cell;

use axum::{
    body::Body,
    http::{header, Response, StatusCode},
    response::IntoResponse,
};

use crate::binding::http::{Builder, Serializer};
use crate::event::Event;
use crate::message::{BinaryDeserializer, Error, Result};

impl IntoResponse for Event {
    fn into_response(self) -> Response<Body> {
        match to_response(self) {
            Ok(resp) => {
                let (parts, body) = resp.into_parts();
                Response::from_parts(parts, Body::from(body))
            }
            Err(err) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from(err.to_string()))
                .unwrap(),
        }
    }
}

struct Adapter {
    builder: Cell<axum::http::response::Builder>,
}

impl Builder<Response<Body>> for Adapter {
    fn header(&mut self, key: &str, value: http::header::HeaderValue) {
        self.builder.set(self.builder.take().header(key, value));
    }
    fn body(&mut self, bytes: Vec<u8>) -> Result<Response<Body>> {
        self.builder
            .take()
            .body(Body::from(bytes))
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    }
    fn finish(&mut self) -> Result<Response<Body>> {
        self.body(Vec::new())
    }
}

pub fn to_response(event: Event) -> std::result::Result<Response<Body>, Error> {
    BinaryDeserializer::deserialize_binary(
        event,
        Serializer::new(Adapter {
            builder: Cell::new(http::Response::builder()),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test::fixtures;

    #[test]
    fn axum_test_response() {
        let input = fixtures::v10::minimal_string_extension();

        let resp = input.into_response();

        assert_eq!(
            resp.headers()
                .get("ce-specversion")
                .unwrap()
                .to_str()
                .unwrap(),
            "1.0"
        );
        assert_eq!(
            resp.headers().get("ce-id").unwrap().to_str().unwrap(),
            "0001"
        );
        assert_eq!(
            resp.headers().get("ce-type").unwrap().to_str().unwrap(),
            "test_event.test_application"
        );
        assert_eq!(
            resp.headers().get("ce-source").unwrap().to_str().unwrap(),
            "http://localhost/"
        );
        assert_eq!(
            resp.headers().get("ce-someint").unwrap().to_str().unwrap(),
            "10"
        );
    }

    #[tokio::test]
    async fn axum_test_response_with_full_data() {
        let input = fixtures::v10::full_binary_json_data_string_extension();

        let resp = input.into_response();

        assert_eq!(
            resp.headers()
                .get("ce-specversion")
                .unwrap()
                .to_str()
                .unwrap(),
            "1.0"
        );
        assert_eq!(
            resp.headers().get("ce-id").unwrap().to_str().unwrap(),
            "0001"
        );
        assert_eq!(
            resp.headers().get("ce-type").unwrap().to_str().unwrap(),
            "test_event.test_application"
        );
        assert_eq!(
            resp.headers().get("ce-source").unwrap().to_str().unwrap(),
            "http://localhost/"
        );
        assert_eq!(
            resp.headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
            "application/json"
        );
        assert_eq!(
            resp.headers().get("ce-int_ex").unwrap().to_str().unwrap(),
            "10"
        );

        let (_, body) = resp.into_parts();
        let body = axum::body::to_bytes(body, usize::MAX).await.unwrap();

        assert_eq!(fixtures::json_data_binary(), body);
    }
}
