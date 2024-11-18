use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{auth, error::OpenAIAPIError, OpenAI, OpenAIResult};

pub mod chat;
pub mod chat_reasoning;
pub mod embeddings;

const API_BASE_URL: &str = "https://api.openai.com/v1";

// this enum and the struct below it are hacks to deal with openai's weird response format
// where they will return either a single error field or the success payload.
#[derive(Deserialize)]
#[serde(untagged)]
enum GenericOpenAIResponse<T> {
    Success(T),
    Error(ResponseDeserializableOpenAIAPIError),
}

#[derive(Deserialize)]
struct ResponseDeserializableOpenAIAPIError {
    error: OpenAIAPIError,
}

impl<T> From<GenericOpenAIResponse<T>> for OpenAIResult<T> {
    fn from(value: GenericOpenAIResponse<T>) -> Self {
        match value {
            GenericOpenAIResponse::Success(success) => Ok(success),
            GenericOpenAIResponse::Error(error) => Err(crate::OpenAIError::API(error.error)),
        }
    }
}

pub(super) async fn send_request<Auth, R>(
    openai: &OpenAI<Auth>,
    request: &R,
) -> OpenAIResult<R::Response>
where
    Auth: auth::AuthTokenProvider,
    R: OpenAIRequestProvider,
{
    let bearer_token = openai
        .auth
        .resolve()
        .await
        .ok_or(crate::error::OpenAIError::MissingAuthToken)?;

    openai
        .client
        .request(
            R::METHOD,
            format!("{API_BASE_URL}{}", R::path_with_leading_slash()),
        )
        .header("Authorization", format!("Bearer {bearer_token}"))
        // TODO: support a way to omit the body during a get request if the time comes
        .json(request)
        .send()
        .await?
        .json::<GenericOpenAIResponse<R::Response>>()
        .await?
        .into()
}

mod private {
    pub trait Sealed {}
}

/// Any type that can be sent to the client's `req` method.
pub trait OpenAIRequestProvider: Serialize + private::Sealed {
    type Response: for<'de> Deserialize<'de>;
    const METHOD: Method;

    fn path_with_leading_slash() -> String;
}
