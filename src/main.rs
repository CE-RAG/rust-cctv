//! CCTV Search Backend
//!
//! A high-performance REST API for vehicle image search using vector embeddings.

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use qdrant_client::Qdrant;
use std::sync::Arc;

mod config;
mod handlers;
mod models;
mod scheduler;
mod services;

use config::{technical, Config};
use scheduler::{start_scheduler, SchedulerContext};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    config.print_summary();

    // Initialize Qdrant client
    let qdrant = Qdrant::from_url(&config.qdrant_url)
        .api_key(config.qdrant_api_key.clone())
        .build()
        .expect("Failed to initialize Qdrant client");

    let qdrant = Arc::new(qdrant);
    let http_client = reqwest::Client::new();

    // Setup Qdrant collection
    setup_qdrant(&qdrant, &config.collection_name).await;

    // Start background scheduler
    let scheduler_ctx = SchedulerContext::new(
        qdrant.clone(),
        http_client.clone(),
        config.clone(),
    );
    start_scheduler(scheduler_ctx).await;

    // Give scheduler time to initialize
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Start HTTP server
    let ai_service_url = config.ai_service_url.clone();
    let collection_name = config.collection_name.clone();
    let server_port = config.server_port;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(handlers::AppState {
                qdrant: qdrant.clone(),
                http_client: http_client.clone(),
                ai_service_url: ai_service_url.clone(),
                collection_name: collection_name.clone(),
            }))
            .service(handlers::search_vehicles)
            .service(handlers::insert_image)
    })
    .bind(("0.0.0.0", server_port))?
    .run()
    .await
}

/// Setup Qdrant collection and indices
async fn setup_qdrant(qdrant: &Arc<Qdrant>, collection_name: &str) {
    println!("Setting up collection...");

    match services::ensure_collection_exists(qdrant, collection_name, technical::VECTOR_SIZE).await {
        Ok(_) => println!("✅ Collection is ready"),
        Err(e) => println!("⚠️  Warning: {}", e),
    }

    println!("Creating datetime field index...");

    match services::create_datetime_index(qdrant, collection_name).await {
        Ok(_) => println!("✅ Datetime field index created successfully"),
        Err(e) => println!("⚠️  Warning: {}", e),
    }
}
