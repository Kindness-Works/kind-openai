//! An opinionated wrapper around the [OpenAI API](https://platform.openai.com/docs/api-reference).
//! This does not support all endpoints, and is not automatically generated.

#![allow(async_fn_in_trait)]

mod auth;
pub mod endpoints;
pub mod error;
mod util;

pub use auth::{AuthTokenProvider, EnvironmentAuthTokenProvider};
use endpoints::chat::ChatCompletion;
pub use error::{OpenAIError, OpenAIResult};
pub use kind_openai_schema::*;
use reqwest::{IntoUrl, Method};
pub use util::UnstructuredString;

/// A handle to OpenAI.
#[derive(Clone)]
pub struct OpenAI<Auth> {
    client: reqwest::Client,
    auth: Auth,
}

impl<Auth> OpenAI<Auth>
where
    Auth: AuthTokenProvider,
{
    /// Creates a new instance of OpenAI with the provided auth.
    pub fn new(auth: Auth) -> Self {
        Self {
            client: reqwest::Client::new(),
            auth,
        }
    }

    async fn post(
        &self,
        url: impl IntoUrl,
        json: &impl serde::Serialize,
    ) -> OpenAIResult<reqwest::Response> {
        Ok(self
            .authed_request(Method::POST, url)
            .await?
            .json(json)
            .send()
            .await?)
    }

    #[allow(dead_code)]
    async fn get(&self, url: impl IntoUrl) -> OpenAIResult<reqwest::Response> {
        Ok(self.authed_request(Method::GET, url).await?.send().await?)
    }

    async fn authed_request(
        &self,
        method: Method,
        url: impl IntoUrl,
    ) -> OpenAIResult<reqwest::RequestBuilder>
    where
        Auth: auth::AuthTokenProvider,
    {
        let bearer_token = self
            .auth
            .resolve()
            .await
            .ok_or(error::OpenAIError::MissingAuthToken)?;

        Ok(self
            .client
            .request(method, url)
            .header("Authorization", format!("Bearer {bearer_token}")))
    }

    /// Creates a structured chat completion on any type that implements
    /// `OpenAISchema`.
    pub async fn create_chat_completion<'a, S>(
        &self,
        req: &endpoints::chat::ChatCompletionRequest<'a, S>,
    ) -> OpenAIResult<ChatCompletion<S>>
    where
        S: OpenAISchema + for<'de> serde::Deserialize<'de>,
    {
        endpoints::chat::create_chat_completion(self, req).await
    }

    pub async fn create_embeddings(
        &self,
        req: &endpoints::embeddings::CreateEmbeddingsRequest<'_>,
    ) -> OpenAIResult<endpoints::embeddings::CreateEmbeddingsResponse> {
        endpoints::embeddings::create_embeddings(self, req).await
    }
}
