use serde::{Deserialize, Serialize};

use crate::{error::OpenAIResponseExt, AuthTokenProvider, OpenAI, OpenAIResult};

use super::API_BASE_URL;

#[derive(Serialize, Debug, Clone, Copy)]
pub enum EmbeddingsModel {
    #[serde(rename = "text-embedding-3-large")]
    TextEmbedding3Large,
}

#[derive(Serialize, Debug, Clone)]
pub struct CreateEmbeddingsRequest<'a> {
    model: EmbeddingsModel,
    input: &'a str,
}

impl<'a> CreateEmbeddingsRequest<'a> {
    pub fn new(model: EmbeddingsModel, input: &'a str) -> Self {
        Self { model, input }
    }
}

#[derive(Deserialize)]
pub struct CreateEmbeddingsResponse {
    data: Vec<EmbeddingsData>,
}

impl CreateEmbeddingsResponse {
    pub fn embedding(&self) -> &[f32] {
        &self.data[0].embedding
    }
}

#[derive(Deserialize)]
struct EmbeddingsData {
    embedding: Vec<f32>,
}

pub async fn create_embeddings<Auth>(
    openai: &OpenAI<Auth>,
    req: &CreateEmbeddingsRequest<'_>,
) -> OpenAIResult<CreateEmbeddingsResponse>
where
    Auth: AuthTokenProvider,
{
    openai
        .post(format!("{API_BASE_URL}/embeddings"), req)
        .await?
        .openai_response_json()
        .await
}
