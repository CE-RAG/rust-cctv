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
    // Load environment variables
    dotenv().ok();

    // Load configuration from environment variables with defaults
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

    // Configure Qdrant client
    let client = Qdrant::from_url(&qdrant_url)
        .api_key(qdrant_api_key)
        .build()
        .expect("Failed to initialize Qdrant client");

    // Create shared state
    let qdrant_arc = Arc::new(client);
    let http_client = reqwest::Client::new();

    // Ensure collection exists and create datetime field index
    println!("Setting up collection...");
    match services::ensure_collection_exists(&qdrant_arc, &collection_name, 768).await {
        Ok(_) => println!("✅ Collection is ready"),
        Err(e) => println!("⚠️  Warning: {}", e),
    }

    println!("Creating datetime field index...");
    match services::create_datetime_index(&qdrant_arc, &collection_name).await {
        Ok(_) => println!("✅ Datetime field index created successfully"),
        Err(e) => println!("⚠️  Warning: {}", e),
    }
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
