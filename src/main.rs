use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use qdrant_client::Qdrant;
use std::env;
use std::sync::Arc;

mod handlers;
mod models;
mod services;

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
            .app_data(web::Data::new(handlers::AppState {
                qdrant: qdrant_arc.clone(),
                http_client: http_client.clone(),
                ai_service_url: ai_service_url.clone(),
                collection_name: collection_name.clone(),
            }))
            .service(handlers::search_vehicles)
            .service(handlers::insert_image)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
