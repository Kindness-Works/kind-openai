/// Any type that can provide a bearer auth token.
pub trait AuthTokenProvider: Clone {
    async fn resolve(&self) -> Option<String>;
}

/// Auth token provided that takes the auth token from the environment variable `OPENAI_API_KEY`.
#[derive(Clone)]
pub struct EnvironmentAuthTokenProvider;

impl EnvironmentAuthTokenProvider {
    const ENV_VAR: &'static str = "OPENAI_API_KEY";
}

impl AuthTokenProvider for EnvironmentAuthTokenProvider {
    async fn resolve(&self) -> Option<String> {
        std::env::var(Self::ENV_VAR).ok()
    }
}
