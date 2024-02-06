use http::Response;
use std::cell::Cell;

use crate::binding::http::{Builder, Serializer};
use crate::message::{BinaryDeserializer, Error, Result};
use crate::Event;

struct Adapter {
    builder: Cell<http::response::Builder>,
}

type HttpBody = Vec<u8>;

impl Builder<Response<HttpBody>> for Adapter {
    fn header(&mut self, key: &str, value: http::header::HeaderValue) {
        self.builder.set(self.builder.take().header(key, value));
    }

    fn body(&mut self, bytes: Vec<u8>) -> Result<Response<HttpBody>> {
        self.builder
            .take()
            .body(bytes)
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    }

    fn finish(&mut self) -> Result<Response<HttpBody>> {
        self.body(Vec::new())
    }
}

pub fn to_response(event: Event) -> std::result::Result<Response<HttpBody>, Error> {
    BinaryDeserializer::deserialize_binary(
        event,
        Serializer::new(Adapter {
            builder: Cell::new(http::Response::builder()),
        }),
    )
}
