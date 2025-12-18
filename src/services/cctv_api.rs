//! CCTV Metadata API Service
//! 
//! Functions for fetching training data from the CCTV metadata API.

use crate::models::search::{CctvImageData, CctvMetadataRequest, CctvMetadataResponse};
use std::time::Duration;

/// Fetch training images from CCTV metadata API
/// 
/// # Arguments
/// * `api_url` - The CCTV metadata API endpoint
/// * `auth_token` - Bearer authentication token
/// * `cctv_id` - Camera ID to fetch images from
/// * `date_start` - Start date in "YYYY-MM-DD HH:MM:SS" format
/// * `date_stop` - End date in "YYYY-MM-DD HH:MM:SS" format
/// * `limit` - Maximum number of images to fetch
pub async fn fetch_cctv_training_data(
    api_url: &str,
    auth_token: &str,
    cctv_id: &str,
    date_start: &str,
    date_stop: &str,
    limit: u32,
) -> Result<Vec<CctvImageData>, String> {
    // Create a client with timeout configuration
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let request = CctvMetadataRequest {
        cctv_id: cctv_id.to_string(),
        date_start: date_start.to_string(),
        date_stop: date_stop.to_string(),
        limit,
    };

    println!("📡 Fetching CCTV training data from API...");
    println!("   -> CCTV ID: {}", cctv_id);
    println!("   -> Date Range: {} to {}", date_start, date_stop);
    println!("   -> Limit: {}", limit);

    let res = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .header("accept", "*/*")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "Connection timed out - API server may be unreachable".to_string()
            } else if e.is_connect() {
                format!("Connection failed - check network or API URL: {}", e)
            } else {
                format!("Failed to connect to CCTV Metadata API: {}", e)
            }
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let error_body = res.text().await.unwrap_or_default();
        return Err(format!(
            "CCTV Metadata API returned error: {} - {}",
            status, error_body
        ));
    }

    let response: CctvMetadataResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse CCTV Metadata API response: {}", e))?;

    if !response.success {
        return Err("API returned success=false".to_string());
    }

    println!("✅ Successfully fetched {} images from CCTV API", response.data.len());

    Ok(response.data)
}
