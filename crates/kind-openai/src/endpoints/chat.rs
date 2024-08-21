use kind_openai_schema::{GeneratedOpenAISchema, OpenAISchema};
use serde::{
    de::{self},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;

use super::API_BASE_URL;
use crate::{
    auth,
    error::{OpenAIAPIError, OpenAIResponseExt, OpenAIResult},
    util, OpenAI, OpenAIError,
};

#[derive(Serialize, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum ChatCompletionModel {
    #[serde(rename = "gpt-4o-2024-08-06")]
    Gpt4o_2024_08_06,
    #[serde(rename = "gpt-4o")]
    Gpt4o,
    #[serde(rename = "gpt-4o-mini")]
    Gpt4oMini,
}

/// A text chat completion request.
#[derive(Serialize)]
pub struct ChatCompletionRequest<'a, S> {
    model: ChatCompletionModel,
    messages: Vec<ChatCompletionRequestMessage<'a>>,
    temperature: Option<f32>,
    response_format: Option<ChatCompletionRequestResponseFormat>,
    #[serde(skip)]
    _maybe_schema: std::marker::PhantomData<S>,
}

impl<'a, S> ChatCompletionRequest<'a, S> {
    /// Creates a new chat completion request for a given model.
    /// Schema type must be internally defined and as such not an
    /// `OpenAISchema`, meaning that it will not be a structured response.
    ///
    /// 99% of the time, you want to use `kopenai::UnstructuredString` here as
    /// `S`
    pub fn new(model: ChatCompletionModel) -> Self
    // by sealing `S`, we can ensure that this can only be used on non-external
    // schema types, effectively creating `!OpenAISchema`.
    where
        S: util::sealed::Sealed,
    {
        Self {
            model,
            messages: Vec::new(),
            temperature: None,
            response_format: None,
            _maybe_schema: std::marker::PhantomData,
        }
    }

    pub fn new_structured(model: ChatCompletionModel) -> Self
    where
        S: OpenAISchema,
    {
        Self {
            model,
            messages: Vec::new(),
            temperature: None,
            response_format: Some(ChatCompletionRequestResponseFormat::JsonSchema(
                S::openai_schema(),
            )),
            _maybe_schema: std::marker::PhantomData,
        }
    }

    /// Adds a message to the request.
    pub fn message(mut self, message: ChatCompletionRequestMessage<'a>) -> Self {
        self.messages.push(message);
        self
    }

    /// Sets the request temperature.
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

#[derive(Serialize)]
// TODO: fix this so that `content = "json_schema"` is not necessary
#[serde(tag = "type", content = "json_schema", rename_all = "snake_case")]
enum ChatCompletionRequestResponseFormat {
    JsonSchema(GeneratedOpenAISchema),
}

/// A chat completion message. You can pre-populate the request with user and
/// assistant messages (alongside the system message) to provide context for the
/// completion.
#[derive(Serialize, Debug)]
pub struct ChatCompletionRequestMessage<'a> {
    role: &'a str,
    content: &'a str,
    refusal: Option<&'a str>,
    name: Option<&'a str>,
}

impl<'a> ChatCompletionRequestMessage<'a> {
    /// Creates a new system message.
    pub fn system(content: &'a str) -> Self {
        Self {
            role: "system",
            content,
            refusal: None,
            name: None,
        }
    }

    /// Creates a new user message.
    pub fn user(content: &'a str) -> Self {
        Self {
            role: "user",
            content,
            refusal: None,
            name: None,
        }
    }

    /// Creates a new assistant message.
    pub fn assistant(content: &'a str) -> Self {
        Self {
            role: "assistant",
            content,
            refusal: None,
            name: None,
        }
    }

    /// Adds a name to the message, which can provide context to the model when
    /// multiple participants are present in the conversation.
    pub fn named(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }
}

#[macro_export]
macro_rules! system_message {
    ($($arg:tt)*) => {
        ChatCompletionRequestMessage::system(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! user_message {
    ($($arg:tt)*) => {
        ChatCompletionRequestMessage::user(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! assistant_message {
    ($($arg:tt)*) => {
        ChatCompletionRequestMessage::assistant(&format!($($arg)*))
    };
}
/// A chat completion response.
#[derive(Deserialize)]
pub struct ChatCompletion<T> {
    // id: String,
    choices: Vec<ChatCompletionChoice<T>>,
}

impl<T> ChatCompletion<T> {
    /// Takes the first choice given from the response.
    pub fn take_first_choice(self) -> OpenAIResult<ChatCompletionChoice<T>> {
        match self.choices.into_iter().next() {
            Some(choice) => Ok(choice),
            None => Err(OpenAIError::API(OpenAIAPIError::NoChoices)),
        }
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct ChatCompletionChoice<T> {
    finish_reason: ChatCompletionFinishReason,
    index: i32,
    message: ChatCompletionResponseMessage<T>,
}

impl<T> ChatCompletionChoice<T> {
    pub fn message(self) -> OpenAIResult<T> {
        self.message.into()
    }
}

#[allow(dead_code)]
struct ChatCompletionResponseMessage<T> {
    content: T,
    refusal: Option<String>,
}

// `content` is a string that contains json inside of it, but we want to unravel
// it into just that inner json. implementing this is an entire deserializer
// instead of a single-function/single-field deserializer is so that we aren't
// required to constrain `T` to deserialize which makes the signatures
// everywhere else cleaner.
impl<'de, T> Deserialize<'de> for ChatCompletionResponseMessage<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        if let Value::Object(mut map) = value {
            let content = map
                .remove("content")
                .ok_or_else(|| de::Error::missing_field("content"))?;

            let content = match content {
                Value::String(s) => serde_json::from_str(&s).map_err(de::Error::custom)?,
                _ => content,
            };

            let content: T = T::deserialize(content).map_err(de::Error::custom)?;

            let refusal = map
                .remove("refusal")
                .and_then(|v| v.as_str().map(String::from));

            Ok(ChatCompletionResponseMessage { content, refusal })
        } else {
            Err(de::Error::custom("expected an object"))
        }
    }
}

impl<T> From<ChatCompletionResponseMessage<T>> for OpenAIResult<T> {
    fn from(value: ChatCompletionResponseMessage<T>) -> Self {
        match value.refusal {
            Some(refusal) => Err(OpenAIError::Refusal(refusal)),
            None => Ok(value.content),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatCompletionFinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
}

pub(crate) async fn create_chat_completion<'a, Auth, S>(
    openai: &OpenAI<Auth>,
    req: &ChatCompletionRequest<'a, S>,
) -> OpenAIResult<ChatCompletion<S>>
where
    Auth: auth::AuthTokenProvider,
    S: for<'de> Deserialize<'de>,
{
    openai
        .post(format!("{API_BASE_URL}/chat/completions"), req)
        .await?
        .openai_response_json()
        .await
}
