use actix_web::{web, App, HttpServer, HttpResponse, Responder, post, get};
use serde::{Deserialize, Serialize};
use qdrant_client::prelude::*;
use qdrant_client::qdrant::{SearchPoints, Value};
use std::collections::HashMap;
use std::env;

// --- Data Structures ---
#[derive(Deserialize)]
struct SearchRequest {
    query: String,
    top_k: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct EmbedResponse {
    vector: Vec<f32>,
}

#[derive(Serialize)]
struct SearchResult {
    filename: String,
    caption: String,
    score: f32,
}

struct AppState {
    qdrant: QdrantClient,
    http_client: reqwest::Client,
    ai_service_url: String,
    collection_name: String,
}

// --- Helper: Call Python AI Service ---
async fn get_text_embedding(
    client: &reqwest::Client,
    base_url: &str,
    text: &str
) -> Result<Vec<f32>, String> {
    let url = format!("{}/embed_text", base_url);
    let res = client.post(&url)
        .json(&serde_json::json!({ "text": text }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("AI Service Error: {}", res.status()));
    }

    let data: EmbedResponse = res.json().await.map_err(|e| e.to_string())?;
    Ok(data.vector)
}

// --- Handler: Search ---
#[post("/search")]
async fn search_vehicles(
    state: web::Data<AppState>,
    payload: web::Json<SearchRequest>,
) -> impl Responder {

    // 1. Get Embedding
    let vector = match get_text_embedding(&state.http_client, &state.ai_service_url, &payload.query).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Embedding failed: {}", e)),
    };

    // 2. Search Qdrant
    let search_points = SearchPoints {
        collection_name: state.collection_name.clone(),
        vector: vector,
        limit: payload.top_k.unwrap_or(5),
        with_payload: Some(true.into()),
        ..Default::default()
    };

    let result = state.qdrant.search_points(&search_points).await;

    match result {
        Ok(response) => {
            let hits: Vec<SearchResult> = response.result.into_iter().map(|point| {
                let payload = point.payload;

                // Helper to extract string from Qdrant Value
                let get_str = |key: &str| -> String {
                    payload.get(key)
                        .and_then(|v| v.kind.as_ref())
                        .and_then(|k| match k {
                            qdrant_client::qdrant::value::Kind::StringValue(s) => Some(s.clone()),
                            _ => None
                        })
                        .unwrap_or_default()
                };

                SearchResult {
                    filename: get_str("filename"),
                    caption: get_str("caption"),
                    score: point.score,
                }
            }).collect();

            HttpResponse::Ok().json(hits)
        },
        Err(e) => HttpResponse::InternalServerError().body(format!("Qdrant Error: {}", e)),
    }
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("Rust Backend is Running")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Read Config from ENV (with defaults for local dev)
    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "https://38c16abc-090f-4840-b89d-f9f0500793e1.europe-west3-0.gcp.cloud.qdrant.io:6333".to_string());
    let ai_service_url = env::var("AI_SERVICE_URL").unwrap_or_else(|_| "http://192.168.248.177:5090/predict".to_string());
    let collection_name = env::var("COLLECTION_NAME").unwrap_or_else(|_| "nt_cctv_vehicles".to_string());

    println!("Starting Server...");
    println!(" -> Qdrant: {}", qdrant_url);
    println!(" -> AI Service: {}", ai_service_url);

    let qdrant_client = QdrantClient::from_url(&qdrant_url)
        .build()
        .expect("Failed to create Qdrant client");

    let http_client = reqwest::Client::new();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                qdrant: qdrant_client.clone(),
                http_client: http_client.clone(),
                ai_service_url: ai_service_url.clone(),
                collection_name: collection_name.clone(),
            }))
            .service(search_vehicles)
            .service(health)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
