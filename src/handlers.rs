use crate::models::search::{InsertImageRequest, SearchRequest, SearchResult};
use crate::services::{
    filename_to_rfc3339, get_image_embedding, get_text_embedding, parse_cctv_filename,
    rfc3339_to_timestamp,
};
use actix_web::{HttpResponse, Responder, post, web};
use qdrant_client::qdrant::{
    Condition, DatetimeRange, Filter, PointStruct, SearchPoints, UpsertPoints,
};
use rand::Rng;
use std::collections::HashMap;

/// Application state shared across all web workers
pub struct AppState {
    pub qdrant: std::sync::Arc<qdrant_client::Qdrant>,
    pub http_client: reqwest::Client,
    pub ai_service_url: String,
    pub collection_name: String,
}

/// Handler for searching vehicles with optional datetime filtering
#[post("/search")]
pub async fn search_vehicles(
    state: web::Data<AppState>,
    payload: web::Json<SearchRequest>,
) -> impl Responder {
    // Get text embedding from AI service
    let vector =
        match get_text_embedding(&state.http_client, &state.ai_service_url, &payload.query).await {
            Ok(v) => v,
            Err(e) => return HttpResponse::InternalServerError().body(e),
        };

    // Prepare search for Qdrant
    let mut search_points = SearchPoints {
        collection_name: state.collection_name.clone(),
        vector,
        vector_name: None,
        limit: payload.top_k.unwrap_or(5),
        with_payload: Some(true.into()),
        ..Default::default()
    };

    // Add datetime filter if provided
    if payload.start_date.is_some() || payload.end_date.is_some() {
        let mut datetime_range = DatetimeRange::default();

        if let Some(start) = &payload.start_date {
            // Skip empty string
            if !start.is_empty() {
                match rfc3339_to_timestamp(start) {
                    Ok(timestamp) => datetime_range.gt = Some(timestamp),
                    Err(e) => {
                        return HttpResponse::BadRequest()
                            .body(format!("Invalid start_date format: {}", e));
                    }
                }
            }
        }

        if let Some(end) = &payload.end_date {
            // Skip empty string
            if !end.is_empty() {
                match rfc3339_to_timestamp(end) {
                    Ok(timestamp) => datetime_range.lte = Some(timestamp),
                    Err(e) => {
                        return HttpResponse::BadRequest()
                            .body(format!("Invalid end_date format: {}", e));
                    }
                }
            }
        }

        // Add datetime filter to search query
        search_points.filter = Some(Filter {
            must: vec![Condition::datetime_range("datetime", datetime_range)],
            ..Default::default()
        });
    }

    // Execute search
    let result = state.qdrant.search_points(search_points).await;

    match result {
        Ok(response) => {
            // Map results to JSON
            let hits: Vec<SearchResult> = response
                .result
                .into_iter()
                .map(|point| {
                    let payload = point.payload;

                    // Helper to extract string from Qdrant payload
                    let get_str = |key: &str| -> String {
                        payload
                            .get(key)
                            .and_then(|v| v.kind.as_ref())
                            .and_then(|k| match k {
                                qdrant_client::qdrant::value::Kind::StringValue(s) => {
                                    Some(s.clone())
                                }
                                _ => None,
                            })
                            .unwrap_or_default()
                    };

                    SearchResult {
                        filename: get_str("image"),
                        caption: get_str("caption"),
                        score: point.score,
                        datetime: Some(get_str("datetime")),
                    }
                })
                .collect();

            HttpResponse::Ok().json(hits)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Qdrant search error: {}", e)),
    }
}

/// Handler for inserting a new image with metadata
#[post("/insert_image")]
pub async fn insert_image(
    state: web::Data<AppState>,
    payload: web::Json<InsertImageRequest>,
) -> impl Responder {
    // Parse filename to extract metadata
    let parsed_filename = match parse_cctv_filename(&payload.image) {
        Ok(parsed) => parsed,
        Err(e) => {
            return HttpResponse::BadRequest().body(format!("Invalid filename format: {}", e));
        }
    };

    // Convert to RFC 3339 format for storage
    let datetime_rfc3339 = filename_to_rfc3339(&parsed_filename);

    // Get image embedding from AI service
    let vector = match get_image_embedding(
        &state.http_client,
        &state.ai_service_url,
        &payload.image,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(e),
    };

    // Build Qdrant point
    let mut rng = rand::thread_rng();
    let point_id: u64 = rng.r#gen();

    // Create payload with image metadata
    let mut payload_map: HashMap<String, qdrant_client::qdrant::Value> = HashMap::new();

    // Store image URL/filename
    payload_map.insert(
        "image".to_string(),
        qdrant_client::qdrant::Value {
            kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                payload.image.clone(),
            )),
        },
    );

    // Store camera ID
    payload_map.insert(
        "camera_id".to_string(),
        qdrant_client::qdrant::Value {
            kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                parsed_filename.camera_id.clone(),
            )),
        },
    );

    // Store datetime in RFC 3339 format
    payload_map.insert(
        "datetime".to_string(),
        qdrant_client::qdrant::Value {
            kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                datetime_rfc3339.clone(),
            )),
        },
    );

    let point = PointStruct::new(point_id, vector.clone(), payload_map);

    // Upsert point to Qdrant
    let upsert = UpsertPoints {
        collection_name: state.collection_name.clone(),
        wait: Some(true),
        points: vec![point],
        ..Default::default()
    };

    match state.qdrant.upsert_points(upsert).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "point_id": point_id,
            "type": "image_embedding",
            "embedding": vector,
        })),
        Err(e) => HttpResponse::InternalServerError().body(format!("Qdrant upsert error: {}", e)),
    }
}
