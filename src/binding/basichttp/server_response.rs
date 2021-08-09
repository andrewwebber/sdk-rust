use http::response::{Builder, Response};

use crate::binding::{
    http::{header_prefix, SPEC_VERSION_HEADER},
    CLOUDEVENTS_JSON_HEADER,
};
use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinarySerializer, Error, MessageAttributeValue, Result,
    StructuredSerializer,
};
use crate::{str_to_header_value, Event};

pub struct ResponseSerializer<T: From<Vec<u8>> + Default> {
    builder: Builder,
    phantom: std::marker::PhantomData<T>,
}

impl<T: From<Vec<u8>> + Default> ResponseSerializer<T> {
    fn new() -> Self {
        Self {
            builder: http::Response::builder(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T: From<Vec<u8>> + Default> BinarySerializer<Response<T>> for ResponseSerializer<T> {
    fn set_spec_version(mut self, spec_version: SpecVersion) -> Result<Self> {
        self.builder = self
            .builder
            .header(SPEC_VERSION_HEADER, str_to_header_value!(spec_version)?);
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder = self
            .builder
            .header(&header_prefix(name), str_to_header_value!(value)?);
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder = self
            .builder
            .header(&header_prefix(name), str_to_header_value!(value)?);
        Ok(self)
    }

    fn end_with_data(self, bytes: Vec<u8>) -> Result<Response<T>> {
        self.builder
            .body(bytes.into())
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    }

    fn end(self) -> Result<Response<T>> {
        self.builder
            .body(T::default())
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    }
}

impl<T: From<Vec<u8>> + Default> StructuredSerializer<Response<T>> for ResponseSerializer<T> {
    fn set_structured_event(self, bytes: Vec<u8>) -> Result<Response<T>> {
        self.builder
            .header(http::header::CONTENT_TYPE, CLOUDEVENTS_JSON_HEADER)
            .body(bytes.into())
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    }
}

pub fn event_to_response<T: From<Vec<u8>> + Default>(
    event: Event,
) -> std::result::Result<Response<T>, Error> {
    BinaryDeserializer::deserialize_binary(event, ResponseSerializer::<T>::new())
}
