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
    #[error("model context length exceeded: {0}")]
    ContextLengthExceeded(OpenAIAPIErrorData),
    #[error("cloudflare service unavailable: {0}")]
    CfServiceUnavailable(OpenAIAPIErrorData),
    #[error("transient server error: {0}")]
    ServerError(OpenAIAPIErrorData),
    #[error("cloudflare bad gateway: {0}")]
    CfBadGateway(OpenAIAPIErrorData),
    #[error("quota exceeded: {0}")]
    QuotaExceeded(OpenAIAPIErrorData),
    #[error("internal error: {0}")]
    InternalError(OpenAIAPIErrorData),
    #[error("invalid request error: {0}")]
    InvalidRequestError(OpenAIAPIErrorData),
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAIAPIErrorData {
    pub message: String,
    pub param: Option<String>,
    pub code: Option<String>,
}

impl std::fmt::Display for OpenAIAPIErrorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "message: {}", self.message)?;
        if let Some(param) = &self.param {
            write!(f, ", param: {}", param)?;
        }
        if let Some(code) = &self.code {
            write!(f, ", code: {}", code)?;
        }
        Ok(())
    }
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
