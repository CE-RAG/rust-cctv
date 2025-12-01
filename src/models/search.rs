use serde::{Deserialize, Serialize};

/// Request for searching images with optional datetime filtering
#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub top_k: Option<u64>,
    pub start_date: Option<String>, // RFC 3339 format
    pub end_date: Option<String>,   // RFC 3339 format
}

/// Request to insert a new image
#[derive(Deserialize)]
pub struct InsertImageRequest {
    pub image: String, // URL or filename
}

/// Result from image search
#[derive(Serialize)]
pub struct SearchResult {
    pub filename: String,
    pub caption: String,
    pub score: f32,
    pub datetime: Option<String>,
}

/// Response from AI embedding service
#[derive(Serialize, Deserialize, Debug)]
pub struct EmbedResponse {
    #[serde(alias = "embedding")]
    pub vector: Vec<f32>,
}

/// Parsed components from CCTV filename
#[derive(Debug)]
pub struct ParsedFilename {
    pub camera_id: String,
    pub date: String,
    pub time: String,
    #[allow(dead_code)]
    pub sequence: String,
}
