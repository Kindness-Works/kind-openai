//! A procedural macro for deriving an OpenAI-compatible JSON schema for a Rust
//! struct.

use std::fmt::Display;

pub use kind_openai_schema_impl::OpenAISchema;
use serde::{ser::Serializer, Serialize};
use serde_json::value::RawValue;

#[derive(Debug, Clone, Copy)]
pub struct GeneratedOpenAISchema(&'static str);

impl Display for GeneratedOpenAISchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl From<&'static str> for GeneratedOpenAISchema {
    fn from(schema: &'static str) -> Self {
        Self(schema)
    }
}

impl Serialize for GeneratedOpenAISchema {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw = RawValue::from_string(self.0.to_owned()).map_err(serde::ser::Error::custom)?;
        raw.serialize(serializer)
    }
}

pub trait OpenAISchema {
    fn openai_schema() -> GeneratedOpenAISchema;
}
