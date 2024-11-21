mod standard;
mod structured;

pub use standard::*;
pub use structured::*;

use std::borrow::Cow;

use bon::{builder, Builder};
use serde::{Deserialize, Serialize};

/// The model that can be used for either standard or structured chat completions.
#[derive(Serialize, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Model {
    #[serde(rename = "gpt-4o-2024-11-20")]
    Gpt4o_2024_11_20,
    #[serde(rename = "gpt-4o-2024-08-06")]
    Gpt4o_2024_08_06,
    #[serde(rename = "gpt-4o")]
    Gpt4o,
    #[serde(rename = "gpt-4o-mini")]
    Gpt4oMini,
}

pub use standard::{ChatCompletion, ChatCompletionBuilder};
pub use structured::StructuredChatCompletion;

use crate::{OpenAIError, OpenAIResult};

/// The role of the message used for the chat completion.
#[derive(Serialize, Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// The system message, describing the task to the model. As a tip, when using structured outputs
    /// try and keep this smaller. You don't need to include each field / variant's description in this,
    /// and can instead rely on docstrings to be included in the schema's prompt!
    System,
    /// The user message, i.e. the payload into the model.
    User,
    /// The assistant message, i.e. the model's response.
    Assistant,
}

/// A chat completion message. You can pre-populate the request with user and
/// assistant messages (alongside the system message) to provide context for the
/// completion.
#[derive(Serialize, Debug, Builder)]
#[builder(start_fn = role)]
pub struct Message<'a> {
    #[builder(start_fn)]
    role: Role,
    content: Cow<'a, str>,
    refusal: Option<&'a str>,
    name: Option<Cow<'a, str>>,
}

#[macro_export]
macro_rules! system_message {
    ($($arg:tt)*) => {
        ::kind_openai::endpoints::chat::Message::role(
            ::kind_openai::endpoints::chat::Role::System
        )
        .content(format!($($arg)*).into())
        .build();
    };
}

#[macro_export]
macro_rules! user_message {
    ($($arg:tt)*) => {
        ::kind_openai::endpoints::chat::Message::role(
            ::kind_openai::endpoints::chat::Role::User
        )
        .content(format!($($arg)*).into())
        .build();
    };
}

#[macro_export]
macro_rules! assistant_message {
    ($($arg:tt)*) => {
        ::kind_openai::endpoints::chat::Message::role(
            ::kind_openai::endpoints::chat::Role::Assistant
        )
        .content(format!($($arg)*).into())
        .build();
    };
}

/// A chat completion response message. Don't use this type directly, and instead use the
/// `?` AKA `Try` operator to convert it into a result that can be used.
pub struct UnifiedChatCompletionResponseMessage<T> {
    content: T,
    refusal: Option<String>,
}

impl<T> From<UnifiedChatCompletionResponseMessage<T>> for OpenAIResult<T> {
    fn from(value: UnifiedChatCompletionResponseMessage<T>) -> Self {
        match value.refusal {
            Some(refusal) => Err(OpenAIError::Refusal(refusal)),
            None => Ok(value.content),
        }
    }
}

/// The reason the response was terminated.
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
}
