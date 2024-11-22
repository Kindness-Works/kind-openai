use serde::Deserialize;

pub type OpenAIResult<T> = std::result::Result<T, OpenAIError>;

/// Any error that can be produced during the facilitation of an OpenAI request.
#[derive(Debug, thiserror::Error)]
pub enum OpenAIError {
    /// Error that occured at the HTTP / request level.
    #[error("http error: {0}")]
    Reqwest(reqwest::Error),
    /// Malformed response from the OpenAI API.
    #[error("failed to deserialize api response {0} with error: {1}")]
    Serde(String, serde_json::Error),
    /// The auth token was not provided.
    #[error("missing auth token")]
    MissingAuthToken,
    /// An API error occurred.
    #[error("OpenAI API error: {0}")]
    API(OpenAIAPIError),
    /// The model refused to generate a response or could not conform to a particular structured output.
    #[error("OpenAI refused to generate response: {0}")]
    Refusal(String),
}

impl From<reqwest::Error> for OpenAIError {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

#[derive(Debug, Deserialize, Clone, thiserror::Error)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIAPIError {
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

/// The payload of an OpenAI API error.
#[derive(Debug, Deserialize, Clone)]
pub struct OpenAIAPIErrorData {
    /// The message of the error.
    pub message: String,
    /// Any associated data with the error.
    pub param: Option<String>,
    /// The code of the error.
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
