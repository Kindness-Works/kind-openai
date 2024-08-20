use serde::Deserialize;

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// A string that, when deserialize, bypasses the normal deserialization process
/// and instead returns the raw string. This is useful when deserializing data
/// that might be a string or might be a JSON string, and you want to have a
/// unified interface over both.
pub struct UnstructuredString(String);

impl<'de> Deserialize<'de> for UnstructuredString {
    fn deserialize<D>(deserializer: D) -> Result<UnstructuredString, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(UnstructuredString(s))
    }
}

impl AsRef<str> for UnstructuredString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<UnstructuredString> for String {
    fn from(s: UnstructuredString) -> String {
        s.0
    }
}

impl sealed::Sealed for UnstructuredString {}
