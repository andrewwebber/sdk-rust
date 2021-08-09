use axum_lib as axum;

use async_trait::async_trait;
use axum::extract::{FromRequest, RequestParts};
use http::StatusCode;
use http_body::Body;
use hyper::body;

use crate::binding::basichttp::request_to_event;
use crate::event::Event;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
impl<B> FromRequest<B> for Event
where
    B: Body + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let headers = req
            .headers()
            .cloned()
            .ok_or(0)
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid Cloud Event"))?;

        let req_body = req
            .take_body()
            .ok_or(0)
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid Cloud Event"))?;

        let buf = body::to_bytes(req_body)
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, ""))?;

        request_to_event(headers, buf.to_vec())
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid Cloud Event"))
    }
}
