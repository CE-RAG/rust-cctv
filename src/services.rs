use crate::models::search::{EmbedResponse, ParsedFilename};
use chrono::{DateTime, Datelike, Timelike, Utc};

// --- Helper: Call Python AI Service ---
pub async fn get_text_embedding(
    client: &reqwest::Client,
    base_url: &str,
    text: &str,
) -> Result<Vec<f32>, String> {
    let url = format!("{}/predict", base_url);

    // Send request to Python
    let res = client
        .post(&url)
        .json(&serde_json::json!({ "text": text }))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to AI Service: {}", e))?;

    // Check status
    if !res.status().is_success() {
        return Err(format!("AI Service returned error: {}", res.status()));
    }

    // Parse JSON
    let data: EmbedResponse = res.json().await.map_err(|e| {
        format!(
            "Failed to parse AI response. Ensure Python returns 'vector' or 'embedding' key. Error: {}",
            e
        )
    })?;

    Ok(data.vector)
}

// --- AI Service: Get image embedding ---
pub async fn get_image_embedding(
    client: &reqwest::Client,
    base_url: &str,
    image_path: &str,
) -> Result<Vec<f32>, String> {
    let url = format!("{}/predict", base_url);

    let res = client
        .post(&url)
        .json(&serde_json::json!({ "image_path": image_path }))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to AI Image Service: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("AI Image Service returned error: {}", res.status()));
    }

    let data: EmbedResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse AI image response: {}", e))?;

    Ok(data.vector)
}

// --- Utility: Parse CCTV Filename ---
pub fn parse_cctv_filename(filename: &str) -> Result<ParsedFilename, String> {
    // Extract filename from URL path if needed
    let filename = if filename.contains('/') {
        filename.split('/').last().unwrap_or(filename)
    } else {
        filename
    };

    // First check if it's the mixed dash format (camera-date-time-sequence)
    if filename.contains("cctv") && filename.contains('-') && !filename.contains('_') {
        // Parse format: cctv08-2025-10-08-06-32-4.jpg
        // Need to handle this carefully since date itself contains dashes

        // First extract camera ID (before first dash)
        if let Some(dash_pos) = filename.find('-') {
            let camera_id = filename[..dash_pos].to_string();

            // Remove camera ID and first dash
            let remainder = &filename[dash_pos + 1..];

            // Now parse the remainder: 2025-10-08-06-32-4.jpg
            // Format is: YYYY-MM-DD-HH-MM-sequence.ext
            let parts: Vec<&str> = remainder.split('-').collect();

            if parts.len() < 6 {
                return Err("Invalid dash format filename".to_string());
            }

            // Extract date (YYYY-MM-DD)
            let date = format!("{}-{}-{}", parts[0], parts[1], parts[2]);

            // Extract time (HH-MM)
            let time = format!("{}-{}", parts[3], parts[4]);

            // Extract sequence (e.g., "4")
            let sequence = parts[5].split('.').next().unwrap_or("0").to_string();

            Ok(ParsedFilename {
                camera_id,
                date,
                time,
                sequence,
            })
        } else {
            return Err("Invalid dash format filename - missing camera ID".to_string());
        }
    } else {
        // Handle underscore format
        let parts: Vec<&str> = filename.split('_').collect();

        if parts.len() < 4 {
            return Err("Invalid filename format".to_string());
        }

        // Extract camera ID (e.g., "cctv08")
        let camera_id = parts[0].to_string();

        // Extract date (e.g., "2025-10-08")
        let date = parts[1].to_string();

        // Extract time (e.g., "06-32")
        let time = parts[2].to_string();

        // Extract sequence (e.g., "4")
        let sequence = parts[3].split('.').next().unwrap_or("0").to_string();

        Ok(ParsedFilename {
            camera_id,
            date,
            time,
            sequence,
        })
    }
}

// --- Utility: Convert filename datetime to RFC 3339 format ---
pub fn filename_to_rfc3339(parsed: &ParsedFilename) -> String {
    // Convert time from "06-32" to "06:32"
    let time_with_minutes = parsed.time.replace('-', ":");
    format!("{}T{}:00Z", parsed.date, time_with_minutes)
}

// --- Utility: Parse RFC 3339 datetime to Qdrant Timestamp ---
pub fn rfc3339_to_timestamp(rfc3339_str: &str) -> Result<qdrant_client::qdrant::Timestamp, String> {
    // Chrono traits are already imported above

    // Parse RFC 3339 string
    let dt = DateTime::parse_from_rfc3339(rfc3339_str)
        .map_err(|e| format!("Failed to parse RFC 3339 datetime: {}", e))?;

    // Convert to UTC
    let dt_utc = dt.with_timezone(&Utc);

    // Create Qdrant Timestamp
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

// --- Qdrant: Create collection if it doesn't exist ---
pub async fn ensure_collection_exists(
    qdrant: &qdrant_client::Qdrant,
    collection_name: &str,
    vector_size: usize,
) -> Result<(), String> {
    use qdrant_client::qdrant::{CreateCollection, Distance, VectorParams};

    // Just try to create the collection, ignore if it already exists
    let vector_params = VectorParams {
        size: vector_size as u64,
        distance: Distance::Cosine.into(),
        ..Default::default()
    };

    let create_collection = CreateCollection {
        collection_name: collection_name.to_string(),
        vectors_config: Some(vector_params.into()),
        ..Default::default()
    };

    match qdrant.create_collection(create_collection).await {
        Ok(_) => println!("✅ Collection '{}' created successfully", collection_name),
        Err(e) => {
            // Check if error is because collection already exists
            let error_msg = format!("{}", e);
            if error_msg.contains("already exists") {
                println!("✅ Collection '{}' already exists", collection_name);
            } else {
                return Err(format!("Failed to create collection: {}", e));
            }
        }
    }

    Ok(())
}

// --- Qdrant: Create datetime field index ---
pub async fn create_datetime_index(
    qdrant: &qdrant_client::Qdrant,
    collection_name: &str,
) -> Result<(), String> {
    use qdrant_client::qdrant::{CreateFieldIndexCollectionBuilder, FieldType};

    // Check if collection exists first
    let collections = qdrant
        .list_collections()
        .await
        .map_err(|e| format!("Failed to list collections: {}", e))?;

    let collection_exists = collections
        .collections
        .iter()
        .any(|c| c.name == collection_name);

    if !collection_exists {
        return Err(format!("Collection '{}' does not exist", collection_name));
    }

    // Create index for datetime field
    qdrant
        .create_field_index(
            CreateFieldIndexCollectionBuilder::new(
                collection_name,
                "datetime",
                FieldType::Datetime,
            )
            .wait(true),
        )
        .await
        .map_err(|e| format!("Failed to create datetime index: {}", e))?;

    Ok(())
}
