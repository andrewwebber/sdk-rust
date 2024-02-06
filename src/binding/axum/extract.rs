use axum_lib as axum;

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::{
    body::to_bytes,
    extract::{FromRequest, Request},
    http::request::Parts,
};

use crate::{binding::http::to_event, event::Event};

#[async_trait]
impl<S> FromRequest<S> for Event
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let (Parts { headers, .. }, req_body) = req.into_parts();
        let buf = to_bytes(req_body, usize::MAX)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("{}", e)))?
            .to_vec();

        to_event(&headers, buf).map_err(|e| (StatusCode::BAD_REQUEST, format!("{}", e)))
    }
}

#[cfg(test)]
mod tests {
    use axum_lib as axum;

    use super::*;
    use axum::body::Body;
    use axum::http::{self, Request, StatusCode};

    use crate::test::fixtures;

    #[tokio::test]
    async fn axum_test_request() {
        let expected = fixtures::v10::minimal_string_extension();

        let request = Request::builder()
            .method(http::Method::POST)
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "test_event.test_application")
            .header("ce-source", "http://localhost/")
            .header("ce-someint", "10")
            .body(Body::empty())
            .unwrap();

        let result = Event::from_request(request, &()).await.unwrap();

        assert_eq!(expected, result);
    }

    #[tokio::test]
    async fn axum_test_bad_request() {
        let request = Request::builder()
            .method(http::Method::POST)
            .header("ce-specversion", "BAD SPECIFICATION")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost/")
            .header("ce-someint", "10")
            .header("ce-time", fixtures::time().to_rfc3339())
            .body(Body::empty())
            .unwrap();

        let result = Event::from_request(request, &()).await;
        assert!(result.is_err());
        let rejection = result.unwrap_err();

        let reason = rejection.0;
        assert_eq!(reason, StatusCode::BAD_REQUEST)
    }

    #[tokio::test]
    async fn axum_test_request_with_full_data() {
        let expected = fixtures::v10::full_binary_json_data_string_extension();

        let request = Request::builder()
            .method(http::Method::POST)
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "test_event.test_application")
            .header("ce-source", "http://localhost/")
            .header("ce-subject", "cloudevents-sdk")
            .header("content-type", "application/json")
            .header("ce-string_ex", "val")
            .header("ce-int_ex", "10")
            .header("ce-bool_ex", "true")
            .header("ce-time", &fixtures::time().to_rfc3339())
            .body(Body::from(fixtures::json_data_binary()))
            .unwrap();

        let result = Event::from_request(request, &()).await.unwrap();

        assert_eq!(expected, result);
    }
}
