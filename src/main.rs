use actix_web::{web, App, HttpServer, HttpResponse, Responder, post};
use serde::{Deserialize, Serialize};
use qdrant_client::prelude::*;
use qdrant_client::qdrant::{SearchPoints, PointStruct, Value};
use std::collections::HashMap;

// --- Config ---
const QDRANT_URL: &str = "http://localhost:6334"; // Or your Cloud URL
const AI_SERVICE_URL: &str = "http://localhost:8000";
const COLLECTION_NAME: &str = "nt_cctv_vehicles";

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
    metadata: HashMap<String, String>, // Simplified metadata
}

struct AppState {
    qdrant: QdrantClient,
    http_client: reqwest::Client,
}

// --- Helper: Call Python AI Service ---
async fn get_text_embedding(client: &reqwest::Client, text: &str) -> Result<Vec<f32>, String> {
    let res = client
        .post(format!("{}/embed_text", AI_SERVICE_URL))
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
    
    // 1. Get Embedding from Python Service
    let vector = match get_text_embedding(&state.http_client, &payload.query).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Embedding failed: {}", e)),
    };

    println!("Generated embedding for '{}': len={}", payload.query, vector.len());

    // 2. Search Qdrant
    let search_points = SearchPoints {
        collection_name: COLLECTION_NAME.to_string(),
        vector: vector,
        limit: payload.top_k.unwrap_or(5),
        with_payload: Some(true.into()),
        ..Default::default()
    };

    let result = state.qdrant.search_points(&search_points).await;

    match result {
        Ok(response) => {
            // 3. Map Qdrant results to clean JSON
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
                    metadata: HashMap::new(), // You can map other fields (color, brand) here
                }
            }).collect();

            HttpResponse::Ok().json(hits)
        },
        Err(e) => HttpResponse::InternalServerError().body(format!("Qdrant Error: {}", e)),
    }
}

// --- Handler: Index (Example trigger) ---
// This assumes you send the image URL or Path to be indexed
#[derive(Deserialize)]
struct IndexRequest {
    filename: String,
    image_path: String,
    metadata: serde_json::Value, 
}

#[post("/index")]
async fn index_image(
    state: web::Data<AppState>,
    payload: web::Json<IndexRequest>,
) -> impl Responder {
    // Note: In a real app, you would read the file here and send 
    // multipart data to the Python /embed_image endpoint.
    // For brevity, this is a placeholder structure.
    
    HttpResponse::Ok().body("To implement: Read file -> Call Python /embed_image -> Qdrant Upsert")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize Qdrant Client
    // Note: For Qdrant Cloud, use QdrantClient::from_url("...").with_api_key("...")
    let qdrant_client = QdrantClient::from_url(QDRANT_URL)
        .build()
        .expect("Failed to create Qdrant client");

    let http_client = reqwest::Client::new();

    println!("Server starting at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                qdrant: qdrant_client.clone(),
                http_client: http_client.clone(),
            }))
            .service(search_vehicles)
            .service(index_image)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}