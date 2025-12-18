//! Filename Utilities
//! 
//! Functions for parsing CCTV filenames and datetime conversions.

use crate::models::search::ParsedFilename;
use chrono::{DateTime, Datelike, Timelike, Utc};

/// Parse CCTV filename to extract metadata
/// 
/// Supports formats:
/// - Underscore: `cctv08_2025-10-08_06-32_4.jpg`
/// - Dash: `cctv08-2025-10-08-06-32-4.jpg`
/// - Full URLs with filename at the end
pub fn parse_cctv_filename(filename: &str) -> Result<ParsedFilename, String> {
    // Extract filename from URL path if needed
    let filename = if filename.contains('/') {
        filename.split('/').last().unwrap_or(filename)
    } else {
        filename
    };

    // Check if it's the mixed dash format (camera-date-time-sequence)
    if filename.contains("cctv") && filename.contains('-') && !filename.contains('_') {
        parse_dash_format(filename)
    } else {
        parse_underscore_format(filename)
    }
}

/// Parse dash format: cctv08-2025-10-08-06-32-4.jpg
fn parse_dash_format(filename: &str) -> Result<ParsedFilename, String> {
    let dash_pos = filename
        .find('-')
        .ok_or("Invalid dash format filename - missing camera ID")?;

    let camera_id = filename[..dash_pos].to_string();
    let remainder = &filename[dash_pos + 1..];

    // Format: YYYY-MM-DD-HH-MM-sequence.ext
    let parts: Vec<&str> = remainder.split('-').collect();

    if parts.len() < 6 {
        return Err("Invalid dash format filename".to_string());
    }

    Ok(ParsedFilename {
        camera_id,
        date: format!("{}-{}-{}", parts[0], parts[1], parts[2]),
        time: format!("{}-{}", parts[3], parts[4]),
        sequence: parts[5].split('.').next().unwrap_or("0").to_string(),
    })
}

/// Parse underscore format: cctv08_2025-10-08_06-32_4.jpg
fn parse_underscore_format(filename: &str) -> Result<ParsedFilename, String> {
    let parts: Vec<&str> = filename.split('_').collect();

    if parts.len() < 4 {
        return Err("Invalid filename format".to_string());
    }

    Ok(ParsedFilename {
        camera_id: parts[0].to_string(),
        date: parts[1].to_string(),
        time: parts[2].to_string(),
        sequence: parts[3].split('.').next().unwrap_or("0").to_string(),
    })
}

/// Convert parsed filename datetime to RFC 3339 format
pub fn filename_to_rfc3339(parsed: &ParsedFilename) -> String {
    let time_with_minutes = parsed.time.replace('-', ":");
    format!("{}T{}:00Z", parsed.date, time_with_minutes)
}

/// Parse RFC 3339 datetime string to Qdrant Timestamp
pub fn rfc3339_to_timestamp(rfc3339_str: &str) -> Result<qdrant_client::qdrant::Timestamp, String> {
    let dt = DateTime::parse_from_rfc3339(rfc3339_str)
        .map_err(|e| format!("Failed to parse RFC 3339 datetime: {}", e))?;

    let dt_utc = dt.with_timezone(&Utc);

    qdrant_client::qdrant::Timestamp::date_time(
        dt_utc.year() as i64,
        dt_utc.month() as u8,
        dt_utc.day() as u8,
        dt_utc.hour() as u8,
        dt_utc.minute() as u8,
        dt_utc.second() as u8,
    )
    .map_err(|e| format!("Failed to create timestamp: {}", e))
}
