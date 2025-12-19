//! Application Configuration
//!
//! Centralized configuration loading with sensible defaults.

use std::env;

/// Default application constants
pub mod defaults {
    pub const QDRANT_URL: &str = "http://localhost:6334";
    pub const AI_SERVICE_URL: &str = "http://localhost:5090";
    pub const COLLECTION_NAME: &str = "ntcctvvehicles";
    pub const CCTV_API_URL: &str = "https://ntvideo.totbb.net/video-metadata/train-data-condition";
    pub const CCTV_ID: &str = "cctv01";
    pub const VECTOR_SIZE: usize = 768;
    pub const SERVER_PORT: u16 = 8080;
    pub const FETCH_LIMIT: u32 = 20;
    pub const FETCH_DAYS_RANGE: i64 = 2;
}

/// Application configuration loaded from environment
#[derive(Clone)]
pub struct Config {
    pub qdrant_url: String,
    pub qdrant_api_key: String,
    pub ai_service_url: String,
    pub collection_name: String,
    pub cctv_api_url: String,
    pub cctv_auth_token: String,
    pub cctv_id: String,
}

impl Config {
    /// Load configuration from environment variables with defaults
    pub fn from_env() -> Result<Self, String> {
        let cctv_auth_token = env::var("CCTV_AUTH_TOKEN")
            .map_err(|_| "CCTV_AUTH_TOKEN must be set")?;

        Ok(Self {
            qdrant_url: env::var("QDRANT_URL")
                .unwrap_or_else(|_| defaults::QDRANT_URL.to_string()),
            qdrant_api_key: env::var("QDRANT_API_KEY")
                .unwrap_or_else(|_| "your_api_key_here".to_string()),
            ai_service_url: env::var("AI_SERVICE_URL")
                .unwrap_or_else(|_| defaults::AI_SERVICE_URL.to_string()),
            collection_name: env::var("COLLECTION_NAME")
                .unwrap_or_else(|_| defaults::COLLECTION_NAME.to_string()),
            cctv_api_url: env::var("CCTV_API_URL")
                .unwrap_or_else(|_| defaults::CCTV_API_URL.to_string()),
            cctv_auth_token,
            cctv_id: env::var("CCTV_ID")
                .unwrap_or_else(|_| defaults::CCTV_ID.to_string()),
        })
    }

    /// Print configuration summary
    pub fn print_summary(&self) {
        println!("========================================");
        println!("🚀 Starting CCTV Search Backend");
        println!("   -> Qdrant URL : {}", self.qdrant_url);
        println!("   -> AI Service : {}", self.ai_service_url);
        println!("   -> Collection : {}", self.collection_name);
        println!("========================================");
    }
}
