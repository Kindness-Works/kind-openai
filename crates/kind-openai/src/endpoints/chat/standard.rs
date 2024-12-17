use std::collections::HashMap;

use bon::Builder;
use chat_completion_builder::IsComplete;
use kind_openai_schema::OpenAISchema;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{endpoints::OpenAIRequestProvider, OpenAIResult, Usage};

use super::{
    structured::{ChatCompletionRequestResponseFormat, StructuredChatCompletion},
    FinishReason, Message, Model, UnifiedChatCompletionResponseMessage,
};

/// A standard chat completion request. The response will be a string in any shape and will not
/// be parsed.
#[derive(Serialize, Builder)]
#[builder(start_fn = model, finish_fn = unstructured, state_mod(vis = "pub"))]
pub struct ChatCompletion<'a> {
    #[builder(start_fn)]
    model: Model,
    messages: Vec<Message<'a>>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    store: Option<bool>,
    metadata: Option<HashMap<String, String>>,
    logit_bias: Option<HashMap<i32, i32>>,
}

impl OpenAIRequestProvider for ChatCompletion<'_> {
    type Response = ChatCompletionResponse;

    const METHOD: Method = Method::POST;

    fn path_with_leading_slash() -> String {
        "/chat/completions".to_string()
    }
}

impl super::super::private::Sealed for ChatCompletion<'_> {}

// this is a neat trick where we can take a completed builder and allow it to be "upgraded".
// because of the `finish_fn` specification, we can either resolve and build immediately with
// `.unstructured()`, or we can call `.structured()` and provide a schema. doing it this way
// enables us to nicely represent the `ChatCompletionRequest` without having to specify the
// generic type.
impl<'a, S> ChatCompletionBuilder<'a, S>
where
    S: IsComplete,
{
    /// Upgrades a chat completion request to a structured chat completion request.
    /// Unless the return type can be inferred, you probably want to call this like so:
    /// `.structured::<MySchemadType>();`
    pub fn structured<SS>(self) -> StructuredChatCompletion<'a, SS>
    where
        SS: OpenAISchema,
    {
        StructuredChatCompletion {
            base_request: self.unstructured(),
            response_format: ChatCompletionRequestResponseFormat::JsonSchema(SS::openai_schema()),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// A response from a chat completion request.
#[derive(Deserialize)]
pub struct ChatCompletionResponse {
    choices: Vec<ChatCompletionResponseChoice>,
    usage: Usage,
}

impl ChatCompletionResponse {
    /// Takes the first message in the response consumes the response.
    pub fn take_first_choice(self) -> Option<ChatCompletionResponseChoice> {
        self.choices.into_iter().next()
    }

    /// Gives the usage tokens of the response.
    pub fn usage(&self) -> &Usage {
        &self.usage
    }
}

/// A response choice from a chat completion request.
#[derive(Deserialize)]
pub struct ChatCompletionResponseChoice {
    finish_reason: FinishReason,
    index: i32,
    message: ChatCompletionResponseMessage,
}

impl ChatCompletionResponseChoice {
    /// Takes the message and returns a result that may contain a refusal.
    pub fn message(self) -> OpenAIResult<String> {
        Into::<UnifiedChatCompletionResponseMessage<String>>::into(self.message).into()
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
struct ChatCompletionResponseMessage {
    content: String,
    refusal: Option<String>,
}

impl From<ChatCompletionResponseMessage> for UnifiedChatCompletionResponseMessage<String> {
    fn from(value: ChatCompletionResponseMessage) -> Self {
        UnifiedChatCompletionResponseMessage {
            content: value.content,
            refusal: value.refusal,
        }
    }
}

#[macro_export]
macro_rules! logit_bias {
    () => {
        std::collections::HashMap::new()
    };

    ($($key:tt : $value:expr),+ $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key as i32, $value as i32);
        )+
        map
    }};
}
