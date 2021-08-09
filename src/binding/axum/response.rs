use axum_lib as axum;

use axum::{body::Body, http::Response, response::IntoResponse};
use http::{header, StatusCode};

use crate::binding::basichttp::event_to_response;
use crate::event::Event;

impl IntoResponse for Event {
    type Body = Body;
    type BodyError = <Self::Body as axum::body::HttpBody>::Error;

    fn into_response(self) -> Response<Body> {
        match event_to_response(self) {
            Ok(resp) => resp,
            Err(err) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(err.to_string().into())
                .unwrap(),
        }
    }
}
