use std::{borrow::Cow, collections::HashMap};

use bon::{builder, Builder};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::OpenAIRequestProvider;

/// The model to use to create a chat reasoning completion.
#[derive(Serialize, Clone, Copy, Debug)]
pub enum ReasoningModel {
    #[serde(rename = "o1-preview")]
    O1Preview,
    #[serde(rename = "o1-mini")]
    O1Mini,
}

/// The role in the reasoning completion message (currently doesn't support system messages).
#[derive(Serialize, Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// The user message, containing the instructions for the model _and_ the payload to be used.
    User,
    /// The assistant message, containing the model's response.
    Assistant,
}

/// A chat reasoning completion request. This currently does not support structured outputs.
#[derive(Serialize, Debug, Clone, Builder)]
#[builder(start_fn = model)]
pub struct ChatReasoningCompletion<'a> {
    #[builder(start_fn)]
    model: ReasoningModel,
    messages: Vec<ReasoningMessage<'a>>,
    store: Option<bool>,
    metadata: Option<HashMap<String, String>>,
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
