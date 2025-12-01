use crate::models::search::{InsertImageRequest, SearchRequest, SearchResult};
use crate::services;
use actix_web::{HttpResponse, Responder, post, web};
use qdrant_client::qdrant::{PointStruct, SearchPoints, UpsertPoints};
use rand::Rng;
use std::collections::HashMap;

// Global State shared across all web workers
pub struct AppState {
    pub qdrant: std::sync::Arc<qdrant_client::Qdrant>,
    pub http_client: reqwest::Client,
    pub ai_service_url: String,
    pub collection_name: String,
}

// --- Handler: Search ---
#[post("/search")]
pub async fn search_vehicles(
    state: web::Data<AppState>,
    payload: web::Json<SearchRequest>,
) -> impl Responder {
    // 1. Get Embedding from Python
    let vector = match services::get_text_embedding(
        &state.http_client,
        &state.ai_service_url,
        &payload.query,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(e),
    };

    // 2. Prepare Search for Qdrant
    let search_points = SearchPoints {
        collection_name: state.collection_name.clone(),
        vector: vector,
        vector_name: Some("".to_string()),
        limit: payload.top_k.unwrap_or(5),
        with_payload: Some(true.into()),
        ..Default::default()
    };

    // 3. Execute Search
    // Note: We use the Arc pointer directly
    let result = state.qdrant.search_points(search_points).await;

    match result {
        Ok(response) => {
            // 4. Map results to clean JSON
            let hits: Vec<SearchResult> = response
                .result
                .into_iter()
                .map(|point| {
                    let payload = point.payload;

                    // Helper to safely extract string from Qdrant Payload
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
                        filename: get_str("filename"),
                        caption: get_str("caption"),
                        score: point.score,
                    }
                })
                .collect();

            HttpResponse::Ok().json(hits)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Qdrant Search Error: {}", e)),
    }
}

#[post("/insert_image")]
pub async fn insert_image(
    state: web::Data<AppState>,
    payload: web::Json<InsertImageRequest>,
) -> impl Responder {
    // 1. Get image embedding from Python AI Service
    let vector = match services::get_image_embedding(
        &state.http_client,
        &state.ai_service_url,
        &payload.image,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(e),
    };

    // 2. Build a Qdrant point
    // Use a random u64 as point ID
    let mut rng = rand::thread_rng();
    let point_id: u64 = rng.r#gen();

    // Payload: we at least store image URL; you can add more fields later
    let mut payload_map: HashMap<String, qdrant_client::qdrant::Value> = HashMap::new();
    payload_map.insert(
        "image".to_string(),
        qdrant_client::qdrant::Value {
            kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                payload.image.clone(),
            )),
        },
    );

    let point = PointStruct::new(point_id, vector.clone(), payload_map);

    // 3. Upsert into Qdrant
    let upsert = UpsertPoints {
        collection_name: state.collection_name.clone(),
        wait: Some(true),
        points: vec![point],
        ..Default::default()
    };

    match state.qdrant.upsert_points(upsert).await {
        Ok(_) => {
            // Optionally echo the embedding (if you really want that API shape),
            // but be careful: it can be huge. Here I return type + embedding to match your spec.
            HttpResponse::Ok().json(serde_json::json!({
                "status": "ok",
                "point_id": point_id,
                "type": "image_embedding",
                "embedding": vector,
            }))
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Qdrant Upsert Error: {}", e)),
    }
}
