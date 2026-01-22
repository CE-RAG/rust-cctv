//! Application Configuration
//!
//! Centralized configuration loading with sensible defaults.

use std::env;

/// Default application constants
pub mod defaults {
    pub const QDRANT_URL: &str = "http://localhost:6334";
    pub const AI_SERVICE_URL: &str = "http://localhost:5090";
    pub const COLLECTION_NAME: &str = "nt-cctv-vehicles";
    pub const CCTV_API_URL: &str = "https://ntvideo.totbb.net";
    pub const CCTV_AUTHORIZE_CODE: &str = "your_authorize_code_here";
    pub const CCTV_USER_AUTH: &str = "your_user_auth_here";
    pub const CCTV_CLIENT_ID: &str = "rust-cctv-client";
    pub const SERVER_PORT: u16 = 8080;
    pub const FETCH_LIMIT: u32 = 20;
    pub const FETCH_DAYS_RANGE: i64 = 2;
    pub const FETCH_EVERY_TIME: i64 = 1;
}

/// Technical constants (should not be changed without model retraining)
pub mod technical {
    /// Vector embedding size - must match AI model output
    pub const VECTOR_SIZE: usize = 1152;
}

/// Application configuration loaded from environment
#[derive(Clone)]
pub struct Config {
    pub qdrant_url: String,
    pub qdrant_api_key: String,
    pub ai_service_url: String,
    pub collection_name: String,
    pub cctv_api_url: String,
    pub cctv_authorize_code: String,
    pub cctv_user_auth: String,
    pub cctv_client_id: String,
    pub server_port: u16,
    pub fetch_limit: u32,
    pub fetch_days_range: i64,
    pub fetch_every_time: i64,
}

impl Config {
    /// Load configuration from environment variables with defaults
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            qdrant_url: env::var("QDRANT_URL").unwrap_or_else(|_| defaults::QDRANT_URL.to_string()),
            qdrant_api_key: env::var("QDRANT_API_KEY")
                .unwrap_or_else(|_| "your_api_key_here".to_string()),
            ai_service_url: env::var("AI_SERVICE_URL")
                .unwrap_or_else(|_| defaults::AI_SERVICE_URL.to_string()),
            collection_name: env::var("COLLECTION_NAME")
                .unwrap_or_else(|_| defaults::COLLECTION_NAME.to_string()),
            cctv_api_url: env::var("CCTV_API_URL")
                .unwrap_or_else(|_| defaults::CCTV_API_URL.to_string()),
            cctv_authorize_code: env::var("CCTV_AUTHORIZE_CODE")
                .unwrap_or_else(|_| defaults::CCTV_AUTHORIZE_CODE.to_string()),
            cctv_user_auth: env::var("CCTV_USER_AUTH")
                .unwrap_or_else(|_| defaults::CCTV_USER_AUTH.to_string()),
            cctv_client_id: env::var("CCTV_CLIENT_ID")
                .unwrap_or_else(|_| defaults::CCTV_CLIENT_ID.to_string()),
            server_port: Self::parse_env("SERVER_PORT", defaults::SERVER_PORT)?,
            fetch_limit: Self::parse_env("FETCH_LIMIT", defaults::FETCH_LIMIT)?,
            fetch_days_range: Self::parse_env("FETCH_DAYS_RANGE", defaults::FETCH_DAYS_RANGE)?,
            fetch_every_time: Self::parse_env("FETCH_EVERY_TIME", defaults::FETCH_EVERY_TIME)?,
        })
    }

    /// Helper function to parse environment variables with type conversion
    fn parse_env<T: std::str::FromStr>(key: &str, default: T) -> Result<T, String>
    where
        T::Err: std::fmt::Display,
    {
        match env::var(key) {
            Ok(val) => val
                .parse::<T>()
                .map_err(|e| format!("Failed to parse {}: {} (value: '{}')", key, e, val)),
            Err(_) => Ok(default),
        }
    }

    /// Print configuration summary
    pub fn print_summary(&self) {
        println!("========================================");
        println!("🚀 Starting CCTV Search Backend");
        println!("   -> Server Port : {}", self.server_port);
        println!("   -> Qdrant URL  : {}", self.qdrant_url);
        println!("   -> AI Service  : {}", self.ai_service_url);
        println!("   -> Collection  : {}", self.collection_name);
        println!("   -> Fetch Limit : {} images", self.fetch_limit);
        println!("   -> Fetch Range : {} days", self.fetch_days_range);
        println!("   -> Fetch Every : {} minutes", self.fetch_every_time);
        println!("========================================");
    }
}
