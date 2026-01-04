//! Filename Utilities
//!
//! Functions for datetime conversions.

use chrono::{DateTime, Datelike, Timelike, Utc};

/// Convert API date and time fields directly to RFC 3339 format
///
/// Takes date in format "2025-10-02" and time in format "13:11:00"
/// Returns RFC 3339 format: "2025-10-02T13:11:00Z"
#[inline]
pub fn api_datetime_to_rfc3339(date: &str, time: &str) -> String {
    format!("{}T{}Z", date, time)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_datetime_to_rfc3339() {
        let result = api_datetime_to_rfc3339("2025-10-02", "13:11:00");
        assert_eq!(result, "2025-10-02T13:11:00Z");
    }

    #[test]
    fn test_rfc3339_to_timestamp() {
        let result = rfc3339_to_timestamp("2025-10-02T13:11:00Z");
        assert!(result.is_ok());
    }
}
