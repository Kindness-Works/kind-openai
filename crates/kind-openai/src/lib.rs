//! An opinionated wrapper around the [OpenAI API](https://platform.openai.com/docs/api-reference).
//! This does not support all endpoints, and is not automatically generated.

#![allow(async_fn_in_trait)]

mod auth;
pub mod endpoints;
pub mod error;

pub use auth::{AuthTokenProvider, EnvironmentAuthTokenProvider};
use endpoints::OpenAIRequestProvider;
pub use error::{OpenAIError, OpenAIResult};
pub use kind_openai_schema::*;
use serde::Deserialize;

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

    /// Sends a request to the OpenAI API.
    pub async fn req<R: OpenAIRequestProvider>(&self, r: &R) -> OpenAIResult<R::Response> {
        endpoints::send_request(self, r).await
    }
}

/// The token usage of a request.
#[derive(Deserialize, Clone, Copy, Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
