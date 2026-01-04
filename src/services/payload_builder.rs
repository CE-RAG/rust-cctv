//! Qdrant Payload Builder
//!
//! Utilities for building Qdrant payloads with less boilerplate.

use qdrant_client::qdrant::{value::Kind, Value};
use std::collections::HashMap;

/// Type alias for Qdrant payload map
pub type PayloadMap = HashMap<String, Value>;

/// Builder for Qdrant payload with fluent API
#[derive(Default)]
pub struct PayloadBuilder {
    map: PayloadMap,
}

impl PayloadBuilder {
    /// Create a new empty payload builder
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a string value
    #[inline]
    pub fn string(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.map.insert(
            key.into(),
            Value {
                kind: Some(Kind::StringValue(value.into())),
            },
        );
        self
    }

    /// Insert an optional string value (skips if None)
    #[inline]
    #[allow(dead_code)]
    pub fn string_opt(self, key: impl Into<String>, value: Option<impl Into<String>>) -> Self {
        match value {
            Some(v) => self.string(key, v),
            None => self,
        }
    }

    /// Insert an integer value
    #[inline]
    pub fn integer(mut self, key: impl Into<String>, value: i64) -> Self {
        self.map.insert(
            key.into(),
            Value {
                kind: Some(Kind::IntegerValue(value)),
            },
        );
        self
    }

    /// Insert an optional integer value (skips if None)
    #[inline]
    #[allow(dead_code)]
    pub fn integer_opt(self, key: impl Into<String>, value: Option<i64>) -> Self {
        match value {
            Some(v) => self.integer(key, v),
            None => self,
        }
    }

    /// Insert a double/float value
    #[inline]
    pub fn double(mut self, key: impl Into<String>, value: f64) -> Self {
        self.map.insert(
            key.into(),
            Value {
                kind: Some(Kind::DoubleValue(value)),
            },
        );
        self
    }

    /// Build the final payload map
    #[inline]
    pub fn build(self) -> PayloadMap {
        self.map
    }
}

/// Extract string from Qdrant payload value
#[inline]
pub fn extract_string(payload: &PayloadMap, key: &str) -> String {
    payload
        .get(key)
        .and_then(|v| v.kind.as_ref())
        .and_then(|k| match k {
            Kind::StringValue(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_builder() {
        let payload = PayloadBuilder::new()
            .string("image", "test.jpg")
            .integer("frame", 42)
            .double("confidence", 0.95)
            .build();

        assert_eq!(extract_string(&payload, "image"), "test.jpg");
    }
}
