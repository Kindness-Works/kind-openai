use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::{auth, error::OpenAIResponseExt, OpenAI, OpenAIResult};

use super::API_BASE_URL;

#[derive(Serialize, Clone, Copy, Debug)]
pub enum ChatReasoningModel {
    #[serde(rename = "o1-preview")]
    O1Preview,
    #[serde(rename = "o1-mini")]
    O1Mini,
}

#[derive(Serialize, Debug, Clone, bon::Builder)]
pub struct ChatReasoningCompletionRequest<'a> {
    model: ChatReasoningModel,
    messages: Vec<ChatReasoningCompletionRequestMessage<'a>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ChatReasoningCompletionRequestMessage<'a> {
    role: &'a str,
    content: Cow<'a, str>,
}

impl<'a> ChatReasoningCompletionRequestMessage<'a> {
    pub fn user(content: Cow<'a, str>) -> Self {
        Self {
            role: "user",
            content,
        }
    }

    pub fn assistant(content: Cow<'a, str>) -> Self {
        Self {
            role: "assistant",
            content,
        }
    }
}

#[macro_export]
macro_rules! reasoning_user_message {
    ($($arg:tt)*) => {
        ChatReasoningCompletionRequestMessage::user(format!($($arg)*).into());
    };
}

#[macro_export]
macro_rules! reasoning_assistant_message {
    ($($arg:tt)*) => {
        ChatReasoningCompletionRequestMessage::assistant(format!($($arg)*).into());
    };
}

#[derive(Deserialize)]
pub struct ChatReasoningCompletion {
    choices: Vec<ChatReasoningCompletionChoice>,
}

impl ChatReasoningCompletion {
    pub fn take_first_choice(self) -> Option<ChatReasoningCompletionChoice> {
        self.choices.into_iter().next()
    }
}

#[derive(Deserialize)]
pub struct ChatReasoningCompletionChoice {
    message: ChatReasoningCompletionResponseMessage,
}

impl ChatReasoningCompletionChoice {
    pub fn message(self) -> String {
        self.message.content
    }
}

#[derive(Deserialize)]
pub struct ChatReasoningCompletionResponseMessage {
    content: String,
}

pub(crate) async fn create_chat_reasoning_completion<'a, Auth>(
    openai: &OpenAI<Auth>,
    req: &ChatReasoningCompletionRequest<'a>,
) -> OpenAIResult<ChatReasoningCompletion>
where
    Auth: auth::AuthTokenProvider,
{
    openai
        .post(format!("{API_BASE_URL}/chat/completions"), req)
        .await?
        .openai_response_json()
        .await
}
