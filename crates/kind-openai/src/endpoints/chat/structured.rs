use kind_openai_schema::{GeneratedOpenAISchema, OpenAISchema};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};

use crate::{endpoints::OpenAIRequestProvider, OpenAIResult, Usage};

use super::{standard::ChatCompletion, FinishReason, UnifiedChatCompletionResponseMessage};

/// A chat completion request who's response conforms to a particular JSON schema.
///
/// All types which are structured must derive `kind_openai::OpenAISchema`, as well as
/// `serde::Deserialize`. Take a look at the docs of that trait for a better idea of how
/// to use it.
#[derive(Serialize)]
pub struct StructuredChatCompletion<'a, S> {
    #[serde(flatten)]
    pub(super) base_request: ChatCompletion<'a>,
    pub(super) response_format: ChatCompletionRequestResponseFormat,
    // tether the schema type to the request so that drifting between the request and response
    // type when deserialization time comes is impossible
    #[serde(skip)]
    pub(super) _phantom: std::marker::PhantomData<S>,
}

/// Enum that serializes itself into the part of the request body where OpenAI expects the schema.
#[derive(Serialize)]
// TODO: fix this so that `content = "json_schema"` is not necessary
#[serde(tag = "type", content = "json_schema", rename_all = "snake_case")]
pub(super) enum ChatCompletionRequestResponseFormat {
    JsonSchema(GeneratedOpenAISchema),
}

impl<S> OpenAIRequestProvider for StructuredChatCompletion<'_, S>
where
    S: OpenAISchema + for<'de> Deserialize<'de>,
{
    type Response = StructuredChatCompletionResponse<S>;

    const METHOD: reqwest::Method = reqwest::Method::POST;

    fn path_with_leading_slash() -> String {
        "/chat/completions".to_string()
    }
}

impl<S> super::super::private::Sealed for StructuredChatCompletion<'_, S> {}

/// A response from a structured chat completion request.
#[derive(Deserialize)]
#[serde(bound(deserialize = "S: DeserializeOwned"))]
pub struct StructuredChatCompletionResponse<S> {
    choices: Vec<StructuredChatCompletionResponseChoice<S>>,
    usage: Usage,
}

impl<S> StructuredChatCompletionResponse<S> {
    /// Takes the first message in the response consumes the response.
    pub fn take_first_choice(self) -> Option<StructuredChatCompletionResponseChoice<S>> {
        self.choices.into_iter().next()
    }

    /// Gives the usage tokens of the response.
    pub fn usage(&self) -> Usage {
        self.usage
    }
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "S: DeserializeOwned"))]
pub struct StructuredChatCompletionResponseChoice<S> {
    finish_reason: FinishReason,
    index: i32,
    message: StructuredChatCompletionResponseMessage<S>,
}

impl<S> StructuredChatCompletionResponseChoice<S> {
    /// Returns your desired type that was produced from OpenAI.
    pub fn message(self) -> OpenAIResult<S> {
        Into::<UnifiedChatCompletionResponseMessage<S>>::into(self.message).into()
    }

    pub fn finish_reason(&self) -> FinishReason {
        self.finish_reason
    }

    pub fn index(&self) -> i32 {
        self.index
    }
}

// leave private, messages should only be interacted with through the unified message type.
#[derive(Deserialize)]
#[serde(bound(deserialize = "S: DeserializeOwned"))]
struct StructuredChatCompletionResponseMessage<S> {
    #[serde(deserialize_with = "de_from_str")]
    content: S,
    refusal: Option<String>,
}

fn de_from_str<'de, D, S>(deserializer: D) -> Result<S, D::Error>
where
    D: Deserializer<'de>,
    S: DeserializeOwned,
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

impl<S> From<StructuredChatCompletionResponseMessage<S>>
    for UnifiedChatCompletionResponseMessage<S>
{
    fn from(value: StructuredChatCompletionResponseMessage<S>) -> Self {
        UnifiedChatCompletionResponseMessage {
            content: value.content,
            refusal: value.refusal,
        }
    }
}
