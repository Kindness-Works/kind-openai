use bon::Builder;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::OpenAIRequestProvider;

/// The model used to create text embeddings.
#[derive(Serialize, Debug, Clone, Copy)]
pub enum EmbeddingsModel {
    #[serde(rename = "text-embedding-3-large")]
    TextEmbedding3Large,
}

/// A text embeddings creation request.
///
/// Construct with `Embeddings::model`
#[derive(Serialize, Debug, Clone, Builder)]
#[builder(start_fn = model)]
pub struct Embeddings<'a> {
    #[builder(start_fn)]
    model: EmbeddingsModel,
    input: &'a str,
}

impl OpenAIRequestProvider for Embeddings<'_> {
    type Response = EmbeddingsResponse;

    const METHOD: reqwest::Method = Method::POST;

    fn path_with_leading_slash() -> String {
        "/embeddings".to_string()
    }
}

impl super::private::Sealed for Embeddings<'_> {}

#[derive(Deserialize)]
pub struct EmbeddingsResponse {
    data: Vec<EmbeddingsData>,
}

impl EmbeddingsResponse {
    /// Consumes the response and gives the embeddings.
    pub fn embedding(self) -> Vec<f32> {
        self.data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .unwrap_or_default()
    }

    /// Gives a reference to the generated embeddings.
    pub fn embedding_ref(&self) -> &[f32] {
        &self.data[0].embedding
    }
}

#[derive(Deserialize)]
struct EmbeddingsData {
    embedding: Vec<f32>,
}
