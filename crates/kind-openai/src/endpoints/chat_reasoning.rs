use std::{borrow::Cow, collections::HashMap};

use bon::{builder, Builder};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::OpenAIRequestProvider;

/// The model to use to create a chat reasoning completion.
#[derive(Serialize, Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
pub enum ReasoningModel {
    #[serde(rename = "o1-preview")]
    O1Preview,
    #[serde(rename = "o1-mini")]
    O1Mini,
    #[serde(rename = "o1-mini-2024-09-12")]
    O1Mini_2024_09_12,
    #[serde(rename = "o1")]
    O1,
    #[serde(rename = "o1-2024-12-17")]
    O1_2024_12_17,
    #[serde(rename = "o3-mini")]
    O3Mini,
    #[serde(rename = "o3-mini-2025-01-31")]
    O3Mini_2025_01_31,
}

/// The role in the reasoning completion message (currently doesn't support system messages).
#[derive(Serialize, Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// The user message, containing the instructions for the model _and_ the payload to be used.
    User,
    /// The assistant message, containing the model's response.
    Assistant,
    /// The developer message which provides instructions to the model to follow.
    Developer,
}

/// The amount of effort the model puts into the reasoning. This is essentially the length of the reasoning tokens.
/// Default is medium.
#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

/// A chat reasoning completion request. This currently does not support structured outputs.
#[derive(Serialize, Debug, Clone, Builder)]
#[builder(start_fn = model, state_mod(vis = "pub"))]
pub struct ChatReasoningCompletion<'a> {
    #[builder(start_fn)]
    model: ReasoningModel,
    messages: Vec<ReasoningMessage<'a>>,
    store: Option<bool>,
    metadata: Option<HashMap<String, String>>,
    reasoning_effort: Option<ReasoningEffort>,
}

impl OpenAIRequestProvider for ChatReasoningCompletion<'_> {
    type Response = ChatReasoningCompletionResponse;

    const METHOD: Method = Method::POST;

    fn path_with_leading_slash() -> String {
        "/chat/completions".to_string()
    }
}

impl super::private::Sealed for ChatReasoningCompletion<'_> {}

/// A chat reasoning completion message. This currently does not support structured outputs.
#[derive(Serialize, Debug, Clone, Builder)]
#[builder(start_fn = role)]
pub struct ReasoningMessage<'a> {
    #[builder(start_fn)]
    role: Role,
    content: Cow<'a, str>,
}

#[macro_export]
macro_rules! reasoning_developer_message {
    ($($arg:tt)*) => {
        ::kind_openai::endpoints::chat_reasoning::ReasoningMessage::role(
            ::kind_openai::endpoints::chat_reasoning::Role::Developer
        )
        .content(format!($($arg)*).into())
        .build();
    };
}

#[macro_export]
macro_rules! reasoning_user_message {
    ($($arg:tt)*) => {
        ::kind_openai::endpoints::chat_reasoning::ReasoningMessage::role(
            ::kind_openai::endpoints::chat_reasoning::Role::User
        )
        .content(format!($($arg)*).into())
        .build();
    };
}

#[macro_export]
macro_rules! reasoning_assistant_message {
    ($($arg:tt)*) => {
        ::kind_openai::endpoints::chat_reasoning::ReasoningMessage::role(
            ::kind_openai::endpoints::chat_reasoning::Role::Assistant
        )
        .content(format!($($arg)*).into())
        .build();
    };
}

#[derive(Deserialize)]
pub struct ChatReasoningCompletionResponse {
    choices: Vec<ChatReasoningCompletionResponseChoice>,
}

impl ChatReasoningCompletionResponse {
    pub fn take_first_choice(self) -> Option<ChatReasoningCompletionResponseChoice> {
        self.choices.into_iter().next()
    }
}

#[derive(Deserialize)]
pub struct ChatReasoningCompletionResponseChoice {
    message: ChatReasoningCompletionResponseMessage,
}

impl ChatReasoningCompletionResponseChoice {
    pub fn message(self) -> String {
        self.message.content
    }
}

#[derive(Deserialize)]
pub struct ChatReasoningCompletionResponseMessage {
    content: String,
}
