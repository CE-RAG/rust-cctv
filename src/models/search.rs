//! Data Models
//!
//! Request/Response structures for the API and external services.

use serde::{Deserialize, Serialize};

// =============================================================================
// Search API Models
// =============================================================================

/// Request for searching images with optional datetime filtering
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default)]
    pub top_k: Option<u64>,
    /// Start date filter in RFC 3339 format
    pub start_date: Option<String>,
    /// End date filter in RFC 3339 format
    pub end_date: Option<String>,
}

/// Result from image search
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub filename: String,
    pub id: String,
    pub score: f32,
    pub datetime: String,
}

// =============================================================================
// AI Service Models
// =============================================================================

/// Response from AI embedding service
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedResponse {
    #[serde(alias = "embedding")]
    pub vector: Vec<f32>,
}

/// AI label classification result
#[derive(Debug, Deserialize)]
pub struct AiLabel {
    pub class_name: String,
    pub confidence: f32,
}

// =============================================================================
// CCTV Metadata API Models
// =============================================================================

/// Request to fetch training data from CCTV metadata API
#[derive(Debug, Serialize)]
pub struct CctvMetadataRequest {
    pub cctv_id: String,
    pub date_start: String,
    pub date_stop: String,
    pub limit: u32,
}

/// Response wrapper from CCTV metadata API
#[derive(Debug, Deserialize)]
pub struct CctvMetadataResponse {
    pub success: bool,
    pub count: u32,
    pub data: Vec<CctvImageData>,
}

/// Individual CCTV image metadata
#[derive(Debug, Deserialize)]
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
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
}
