//! A procedural macro for deriving an OpenAI-compatible JSON schema for a Rust
//! struct.

use std::fmt::Display;

pub use kind_openai_schema_impl::OpenAISchema;
use serde::{ser::Serializer, Deserialize, Serialize};
use serde_json::value::RawValue;

/// An OpenAI-compatible JSON schema produced by the `OpenAI` schema derive macro.
#[derive(Debug, Clone, Copy)]
pub struct GeneratedOpenAISchema(&'static str);

impl Display for GeneratedOpenAISchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl From<String> for GeneratedOpenAISchema {
    fn from(schema: String) -> Self {
        let schema = Box::leak(schema.into_boxed_str());
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

/// Any type that can be used as a structured chat completion.
///
/// Docstrings on the top level of a type will automatically be consumed and provided to the schema,
/// as well as any docstrings on fields of said struct.
///
/// Additionally, `serde(skip)` and `serde(rename)` on fields works perfectly fine.
///
/// For example:
/// ```rust
/// #[derive(Deserialize, OpenAISchema)]
/// /// Hello friends
/// struct SuperComplexSchema {
///    // The first one.
///    optional_string: Option<String>,
///    #[serde(rename = "not_so_regular_string")]
///    regular_string: String,
///    #[serde(skip)]
///    regular_string_2: String,
///    int: i32,
///    basic_enum: BasicEnum,
/// }
///
/// #[derive(Deserialize, OpenAISchema)]
/// /// A basic enum.
/// enum BasicEnum {
///    #[serde(rename = "variant1")]
///    Variant1,
///    #[serde(skip)]
///    Variant4,
///    Variant2,
/// }
/// ```
/// Will produce the following schema:
/// ```json
/// {
///   "name": "SuperComplexSchema",
///   "description": "Hello friends",
///   "strict": true,
///   "schema": {
///     "type": "object",
///     "additionalProperties": false,
///     "properties": {
///       "optional_string": {
///         "description": "The first one.",
///         "type": ["string", "null"]
///       },
///       "not_so_regular_string": { "type": "string" },
///       "int": { "type": "integer" },
///       "basic_enum": { "enum": ["variant1", "Variant2"], "type": "string" }
///     },
///     "required": ["optional_string", "not_so_regular_string", "int", "basic_enum"]
///   }
/// }
/// ```
///
/// OpenAI's JSON schema implements a stricter and more limited subset of the JSON schema spec
/// to make it easier for the model to interpret. In addition to that, the proc macro implementation
/// is not 100% complete so there are still some things that are supported that we need to implement, too.
///
/// As such, there are some rules which must be followed (most of which are caught by compiler errors. If they
/// aren't, please file an issue!):
///
/// - The derive can be used on both structs and enums, but only structs can be provided to a structured completion;
///   enums must be used as a field in a containing struct.
/// - Enums must be unit variants. Enums with int descriminants (for example `enum MyEnum { Variant1 = 1, Variant2 = 2 }`) are also
///   allowed, but they must be annotated with `repr(i32)` or similar, and derive `Deserialize_repr` from `serde_repr`.
/// - Struct fields are allowed to be any of the following types:
///     - `String`
///     - All int types, (`i32`, `i64`, `u32`, `u64`, `isize`, `usize`, etc.)
///     - `f32` and `f64`
///     - `bool`
///     - Any unit enum type which also derives `OpenAISchema`
///     - `Vec<T>` where `T` is any of the above types
///     - `Option<T>` where `T` is any of the above types
pub trait OpenAISchema: for<'de> Deserialize<'de> {
    fn openai_schema() -> GeneratedOpenAISchema;
}

/// A subordinate type that can be used as a field in an OpenAI schema but not as the schema itself.
/// (`enum`s and eventually `struct`s when supported using `$ref`). This is still derived by `OpenAISchema`,
/// so for all intents and purposes you can pretend that this type doesn't exist.
pub trait SubordinateOpenAISchema {
    /// Partial schema that will be filled in in the top level schema.
    fn subordinate_openai_schema() -> &'static str;
}
