use http::HeaderMap;

use crate::binding::http::SPEC_VERSION_HEADER;
use crate::event::SpecVersion;
use crate::header_value_to_str;
use crate::message::{
    BinaryDeserializer, BinarySerializer, Encoding, Error, MessageAttributeValue,
    MessageDeserializer, Result, StructuredDeserializer, StructuredSerializer,
};
use crate::{message, Event};
use std::convert::TryFrom;

pub struct RequestDeserializer {
    headers: HeaderMap,
    body: Vec<u8>,
}

impl RequestDeserializer {
    pub fn new(headers: HeaderMap, body: impl Into<Vec<u8>>) -> RequestDeserializer {
        RequestDeserializer {
            headers,
            body: body.into(),
        }
    }
}

impl BinaryDeserializer for RequestDeserializer {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(self, mut visitor: V) -> Result<R> {
        if self.encoding() != Encoding::BINARY {
            return Err(message::Error::WrongEncoding {});
        }

        let spec_version = SpecVersion::try_from(
            self.headers
                .get(SPEC_VERSION_HEADER)
                .map(|a| header_value_to_str!(a))
                .unwrap()?,
        )?;

        visitor = visitor.set_spec_version(spec_version.clone())?;

        let attributes = spec_version.attribute_names();

        for (hn, hv) in self.headers.iter().filter(|(hn, _)| {
            let key = hn.as_str();
            SPEC_VERSION_HEADER.ne(key) && key.starts_with("ce-")
        }) {
            let name = &hn.as_str()["ce-".len()..];

            if attributes.contains(&name) {
                visitor = visitor.set_attribute(
                    name,
                    MessageAttributeValue::String(String::from(header_value_to_str!(hv)?)),
                )?
            } else {
                visitor = visitor.set_extension(
                    name,
                    MessageAttributeValue::String(String::from(header_value_to_str!(hv)?)),
                )?
            }
        }

        if let Some(hv) = self.headers.get("content-type") {
            visitor = visitor.set_attribute(
                "datacontenttype",
                MessageAttributeValue::String(String::from(header_value_to_str!(hv)?)),
            )?
        }

        if !self.body.is_empty() {
            visitor.end_with_data(self.body)
        } else {
            visitor.end()
        }
    }
}

impl StructuredDeserializer for RequestDeserializer {
    fn deserialize_structured<R: Sized, V: StructuredSerializer<R>>(self, visitor: V) -> Result<R> {
        if self.encoding() != Encoding::STRUCTURED {
            return Err(message::Error::WrongEncoding {});
        }
        visitor.set_structured_event(self.body)
    }
}

impl MessageDeserializer for RequestDeserializer {
    fn encoding(&self) -> Encoding {
        if self
            .headers
            .get("content-type")
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or("")
            == "application/cloudevents+json"
        {
            Encoding::STRUCTURED
        } else if self.headers.contains_key(SPEC_VERSION_HEADER) {
            Encoding::BINARY
        } else {
            Encoding::UNKNOWN
        }
    }
}

pub fn request_to_event(
    req: HeaderMap,
    body: impl Into<Vec<u8>>,
) -> std::result::Result<Event, Error> {
    MessageDeserializer::into_event(RequestDeserializer::new(req, body))
}
