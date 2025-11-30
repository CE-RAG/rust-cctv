use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub top_k: Option<u64>,
}

#[derive(Deserialize)]
pub struct InsertImageRequest {
    pub image: String, // image URL
}

#[derive(Serialize)]
pub struct SearchResult {
    pub filename: String,
    pub caption: String,
    pub score: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmbedResponse {
    #[serde(alias = "embedding")]
    pub vector: Vec<f32>,
}
