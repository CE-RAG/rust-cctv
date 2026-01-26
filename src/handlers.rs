//! HTTP Request Handlers
//!
//! Handlers for the REST API endpoints.

use crate::models::search::{CctvImageData, SearchRequest, SearchResult};
use crate::services::{
    PayloadBuilder, api_datetime_to_rfc3339, extract_string, get_image_embedding,
    get_text_embedding, rfc3339_to_timestamp,
};
use actix_web::{HttpResponse, Responder, post, web};
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    Condition, DatetimeRange, Filter, PointStruct, SearchPoints, UpsertPoints,
};
use utoipa;

use std::sync::Arc;

/// Application state shared across all web workers
pub struct AppState {
    pub qdrant: Arc<Qdrant>,
    pub http_client: reqwest::Client,
    pub ai_service_url: String,
    pub collection_name: String,
}

/// Convert PointId to String
fn point_id_to_string(point_id: &qdrant_client::qdrant::PointId) -> String {
    if let Some(kind) = &point_id.point_id_options {
        match kind {
            qdrant_client::qdrant::point_id::PointIdOptions::Num(n) => n.to_string(),
            qdrant_client::qdrant::point_id::PointIdOptions::Uuid(u) => u.clone(),
        }
    } else {
        String::new()
    }
}

/// Handler for searching vehicles with optional datetime filtering
#[utoipa::path(
    post,
    path = "/search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search completed successfully", body = [SearchResult]),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Search API"
)]
#[post("/search")]
pub async fn search_vehicles(
    state: web::Data<AppState>,
    payload: web::Json<SearchRequest>,
) -> impl Responder {
    // Log search request
    let start_time = chrono::Utc::now();
    let datetime_range = match (&payload.start_date, &payload.end_date) {
        (None, None) => "all time".to_string(),
        (Some(s), None) => format!("from {}", s),
        (None, Some(e)) => format!("to {}", e),
        (Some(s), Some(e)) => format!("{} to {}", s, e),
    };
    let top_k = payload.top_k.unwrap_or(5);
    println!(
        "[SEARCH] Query: {}, Range: {}, Top-K: {}, Time: {}",
        &payload.query,
        datetime_range,
        top_k,
        start_time.to_rfc3339()
    );

    // Get text embedding from AI service
    let vector =
        match get_text_embedding(&state.http_client, &state.ai_service_url, &payload.query).await {
            Ok(v) => v,
            Err(e) => return HttpResponse::InternalServerError().body(e),
        };

    // Build search request
    let filter = match build_datetime_filter(&payload) {
        Ok(f) => f,
        Err(e) => return HttpResponse::BadRequest().body(e),
    };

    let search_points = SearchPoints {
        collection_name: state.collection_name.clone(),
        vector,
        vector_name: None,
        limit: payload.top_k.unwrap_or(5),
        with_payload: Some(true.into()),
        filter,
        ..Default::default()
    };

    // Execute search and map results
    match state.qdrant.search_points(search_points).await {
        Ok(response) => {
            let hit_count = response.result.len();
            let elapsed_ms = start_time.signed_duration_since(chrono::Utc::now()).num_milliseconds().abs();
            println!(
                "[SEARCH] Completed: {} results in {}ms",
                hit_count, elapsed_ms
            );

            let hits: Vec<SearchResult> = response
                .result
                .into_iter()
                .map(|point| SearchResult {
                    filename: extract_string(&point.payload, "filename"),
                    id: point
                        .id
                        .as_ref()
                        .map(point_id_to_string)
                        .unwrap_or_default(),
                    score: point.score,
                    datetime: extract_string(&point.payload, "datetime"),
                })
                .collect();
            HttpResponse::Ok().json(hits)
        }
        Err(e) => {
            let elapsed_ms = start_time.signed_duration_since(chrono::Utc::now()).num_milliseconds().abs();
            println!("[SEARCH] Failed after {}ms: {}", elapsed_ms, e);
            HttpResponse::InternalServerError().body(format!("Qdrant search error: {}", e))
        }
    }
}

/// Build datetime filter from search request
fn build_datetime_filter(payload: &SearchRequest) -> Result<Option<Filter>, String> {
    let has_start = payload.start_date.as_ref().map_or(false, |s| !s.is_empty());
    let has_end = payload.end_date.as_ref().map_or(false, |s| !s.is_empty());

    if !has_start && !has_end {
        return Ok(None);
    }

    let mut datetime_range = DatetimeRange::default();

    if let Some(start) = &payload.start_date {
        if !start.is_empty() {
            datetime_range.gt = Some(
                rfc3339_to_timestamp(start)
                    .map_err(|e| format!("Invalid start_date format: {}", e))?,
            );
        }
    }

    if let Some(end) = &payload.end_date {
        if !end.is_empty() {
            datetime_range.lte = Some(
                rfc3339_to_timestamp(end).map_err(|e| format!("Invalid end_date format: {}", e))?,
            );
        }
    }

    Ok(Some(Filter {
        must: vec![Condition::datetime_range("datetime", datetime_range)],
        ..Default::default()
    }))
}

/// Handler for inserting a new image with metadata
#[utoipa::path(
    post,
    path = "/insert_image",
    request_body = CctvImageData,
    responses(
        (status = 200, description = "Image inserted successfully", body = Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "Insertion API"
)]
#[post("/insert_image")]
pub async fn insert_image(
    state: web::Data<AppState>,
    payload: web::Json<CctvImageData>,
) -> impl Responder {
    // Convert date and time to RFC3339 format
    let datetime_rfc3339 = api_datetime_to_rfc3339(&payload.date, &payload.time);

    // Auto-generate createdAt if not provided
    let created_at = payload
        .created_at
        .clone()
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    // Get image embedding from AI service (using file_path)
    let batch_result = match get_image_embedding(
        &state.http_client,
        &state.ai_service_url,
        vec![payload.file_path.clone()]
    ).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(e),
    };

    // Extract the first result
    let result = match batch_result.results.into_iter().next() {
        Some(r) => r,
        None => return HttpResponse::InternalServerError().body("No results returned from AI service"),
    };

    // Check for errors in the result
    if let Some(error) = result.error {
        return HttpResponse::InternalServerError().body(format!("AI Image Service error: {}", error));
    }

    // Get the embedding
    let vector = match result.embedding {
        Some(v) => v,
        None => return HttpResponse::InternalServerError().body("No embedding returned from AI service"),
    };

    // Build payload using the builder pattern
    let mut payload_builder = PayloadBuilder::new()
        .string("image", &payload.file_path)
        .string("filename", &payload.filename)
        .string("camera_id", &payload.cctv_id)
        .string("datetime", &datetime_rfc3339)
        .integer("frame", payload.frame as i64)
        .integer("vehicle_type", payload.vehicle_type as i64)
        .integer("yolo_id", payload.yolo_id as i64)
        .string("created_at", &created_at);

    // Add AI label if present
    if let Some(ref ai_label) = payload.ai_label {
        payload_builder = payload_builder
            .string("vehicle_class", &ai_label.class_name)
            .double("confidence", ai_label.confidence as f64);
    }

    let payload_map = payload_builder.build();

    // Use the API's image ID as point ID
    let point_id: u64 = payload.id as u64;
    let point = PointStruct::new(point_id, vector.clone(), payload_map);

    // Upsert to Qdrant
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
