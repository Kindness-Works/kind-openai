use serde::Deserialize;

pub type OpenAIResult<T> = std::result::Result<T, OpenAIError>;

#[derive(Debug, thiserror::Error)]
pub enum OpenAIError {
    #[error("http error: {0}")]
    Reqwest(reqwest::Error),
    #[error("failed to deserialize api response: {0}")]
    Serde(serde_json::Error),
    #[error("missing auth token")]
    MissingAuthToken,
    #[error("OpenAI API error: {0}")]
    API(OpenAIAPIError),
    #[error("OpenAI refused to generate response: {0}")]
    Refusal(String),
}

impl From<reqwest::Error> for OpenAIError {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

impl From<serde_json::Error> for OpenAIError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serde(err)
    }
}

#[derive(Debug, Deserialize, Clone, thiserror::Error)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIAPIError {
    #[error("no choices")]
    #[serde(skip)]
    NoChoices,
    #[error("model context length exceeded")]
    ContextLengthExceeded(OpenAIAPIErrorData),
    #[error("cloudflare service unavailable")]
    CfServiceUnavailable(OpenAIAPIErrorData),
    #[error("transient server error")]
    ServerError(OpenAIAPIErrorData),
    #[error("cloudflare bad gateway")]
    CfBadGateway(OpenAIAPIErrorData),
    #[error("quota exceeded")]
    QuotaExceeded(OpenAIAPIErrorData),
    #[error("internal error")]
    InternalError(OpenAIAPIErrorData),
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAIAPIErrorData {
    pub message: String,
    pub param: Option<String>,
    pub code: Option<String>,
}

pub(crate) trait OpenAIResponseExt {
    async fn openai_response_json<T>(self) -> OpenAIResult<T>
    where
        T: for<'de> Deserialize<'de>;
}

impl OpenAIResponseExt for reqwest::Response {
    async fn openai_response_json<T>(self) -> OpenAIResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let json = self.json::<serde_json::Value>().await?;
        match json.get("error") {
            Some(err) => {
                let api_err = serde_json::from_value(err.clone())?;
                Err(OpenAIError::API(api_err))
            }
            None => serde_json::from_value(json).map_err(Into::into),
        }
    }
}
