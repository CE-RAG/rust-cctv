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

/// Request to fetch training data from CCTV metadata API
#[derive(Serialize, Debug)]
pub struct CctvMetadataRequest {
    pub cctv_id: String,
    pub date_start: String,
    pub date_stop: String,
    pub limit: u32,
}

/// Response wrapper from CCTV metadata API
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct CctvMetadataResponse {
    pub success: bool,
    pub count: u32,
    pub data: Vec<CctvImageData>,
}

/// Individual CCTV image metadata
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct CctvImageData {
    pub id: u32,
    pub cctv_id: String,
    pub date: String,
    pub time: String,
    pub frame: u32,
    pub vehicle_type: u32,
    pub yolo_id: u32,
    pub filename: String,
    pub file_path: String,
    pub ai_label: Option<AiLabel>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// AI label information
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct AiLabel {
    pub class_name: String,
    pub confidence: f32,
}

