use actix_web::{App, HttpResponse, HttpServer, Responder, post, web};
use dotenv::dotenv;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{PointStruct, SearchPoints, UpsertPoints};
use rand::Rng;
use std::collections::HashMap;
use std::env;
use std::sync::Arc; // for random u64 point IDs

use crate::models::search::{EmbedResponse, InsertImageRequest, SearchRequest, SearchResult};
mod models;
// --- Data Structures ---

// Global State shared across all web workers
struct AppState {
    qdrant: Arc<Qdrant>,
    http_client: reqwest::Client,
    ai_service_url: String,
    collection_name: String,
}

// --- Helper: Call Python AI Service ---
async fn get_text_embedding(
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
        format!("Failed to parse AI response. Ensure Python returns 'vector' or 'embedding' key. Error: {}", e)
    })?;

    Ok(data.vector)
}

// --- Helper: Call Python AI Service for IMAGE embedding ---
async fn get_image_embedding(
    client: &reqwest::Client,
    base_url: &str,
    image_path: &str,
) -> Result<Vec<f32>, String> {
    // Assumes Python exposes POST {base_url}/predict
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

    // Response is: { "type": "image_embedding", "embedding": [...] }
    let data: EmbedResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse AI image response: {}", e))?;

    Ok(data.vector)
}

// --- Handler: Search ---
#[post("/search")]
async fn search_vehicles(
    state: web::Data<AppState>,
    payload: web::Json<SearchRequest>,
) -> impl Responder {
    // 1. Get Embedding from Python
    let vector =
        match get_text_embedding(&state.http_client, &state.ai_service_url, &payload.query).await {
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
async fn insert_image(
    state: web::Data<AppState>,
    payload: web::Json<InsertImageRequest>,
) -> impl Responder {
    // 1. Get image embedding from Python AI Service
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

// --- Handler: Health Check ---
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file
    dotenv().ok();

    // 1. Load Configuration from Environment Variables
    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let ai_service_url =
        env::var("AI_SERVICE_URL").unwrap_or_else(|_| "http://localhost:5090".to_string());
    let collection_name =
        env::var("COLLECTION_NAME").unwrap_or_else(|_| "ntcctvvehicles".to_string());
    let qdrant_api_key =
        env::var("QDRANT_API_KEY").unwrap_or_else(|_| "your_api_key_here".to_string());

    println!("========================================");
    println!("🚀 Starting CCTV Search Backend");
    println!("   -> Qdrant URL : {}", qdrant_url);
    println!("   -> AI Service : {}", ai_service_url);
    println!("   -> Collection : {}", collection_name);
    println!("========================================");

    // 2. Configure Qdrant Client for Cloud gRPC
    let client = Qdrant::from_url(&qdrant_url)
        .api_key(qdrant_api_key) // <-- no &
        .build()
        .expect("Failed to initialize Qdrant Client");

    // 3. Create Shared State (Arc)
    let qdrant_arc = Arc::new(client);
    let http_client = reqwest::Client::new();

    // 4. Start HTTP Server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                qdrant: qdrant_arc.clone(),
                http_client: http_client.clone(),
                ai_service_url: ai_service_url.clone(),
                collection_name: collection_name.clone(),
            }))
            .service(search_vehicles)
            .service(insert_image)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
