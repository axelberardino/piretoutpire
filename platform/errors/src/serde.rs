use crate::prelude::AnyError;
use serde::Deserialize;

pub struct SerializableError;

impl SerializableError {
    pub fn serialize<S>(value: &AnyError, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<AnyError, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(AnyError::msg)
    }
}
