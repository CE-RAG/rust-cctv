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

    // Load CCTV Metadata API configuration
    let cctv_api_url = env::var("CCTV_API_URL").unwrap_or_else(|_| {
        "https://ntvideo.totbb.net/video-metadata/train-data-condition".to_string()
    });
    let cctv_auth_token = env::var("CCTV_AUTH_TOKEN").expect("CCTV_AUTH_TOKEN must be set");
    let cctv_id = env::var("CCTV_ID").unwrap_or_else(|_| "cctv01".to_string());

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

    // Start background scheduler for CCTV image fetching
    let scheduler_qdrant = qdrant_arc.clone();
    let scheduler_http_client = http_client.clone();
    let scheduler_ai_service = ai_service_url.clone();
    let scheduler_collection = collection_name.clone();
    let scheduler_api_url = cctv_api_url.clone();
    let scheduler_auth_token = cctv_auth_token.clone();
    let scheduler_cctv_id = cctv_id.clone();

    tokio::spawn(async move {
        use tokio_cron_scheduler::{Job, JobScheduler};

        let sched = JobScheduler::new()
            .await
            .expect("Failed to create scheduler");

        // Schedule to run every 10 minutes
        let job = Job::new_async("0 */1 * * * *", move |_uuid, _l| {
            let qdrant = scheduler_qdrant.clone();
            let http_client = scheduler_http_client.clone();
            let ai_service_url = scheduler_ai_service.clone();
            let collection_name = scheduler_collection.clone();
            let api_url = scheduler_api_url.clone();
            let auth_token = scheduler_auth_token.clone();
            let cctv_id = scheduler_cctv_id.clone();

            Box::pin(async move {
                println!("\n⏰ Running scheduled CCTV image fetch...");

                // Calculate time range (last 2 days) in Thailand timezone
                use chrono_tz::Asia::Bangkok;
                let now = chrono::Utc::now().with_timezone(&Bangkok);
                let date_stop = now.format("%Y-%m-%d %H:%M:%S").to_string();
                let date_start = (now - chrono::Duration::days(2))
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();

                // Fetch images from CCTV API
                match services::fetch_cctv_training_data(
                    &api_url,
                    &auth_token,
                    &cctv_id,
                    &date_start,
                    &date_stop,
                    20, // Limit to 20 images
                )
                .await
                {
                    Ok(images) => {
                        println!("📥 Processing {} images...", images.len());

                        for (idx, image_data) in images.iter().enumerate() {
                            let file_path = &image_data.file_path;
                            let filename = &image_data.filename;
                            
                            println!("   [{}/{}] Processing: {}", idx + 1, images.len(), filename);

                            // Get image embedding using file_path
                            match services::get_image_embedding(
                                &http_client,
                                &ai_service_url,
                                file_path,
                            )
                            .await
                            {
                                Ok(vector) => {
                                    // Parse filename to extract metadata
                                    match services::parse_cctv_filename(filename) {
                                        Ok(parsed) => {
                                            let datetime_rfc3339 =
                                                services::filename_to_rfc3339(&parsed);

                                            // Create payload
                                            let mut payload_map = std::collections::HashMap::new();
                                            
                                            // Store the file_path URL
                                            payload_map.insert(
                                                "image".to_string(),
                                                qdrant_client::qdrant::Value {
                                                    kind: Some(
                                                        qdrant_client::qdrant::value::Kind::StringValue(
                                                            file_path.clone(),
                                                        ),
                                                    ),
                                                },
                                            );
                                            
                                            // Store filename
                                            payload_map.insert(
                                                "filename".to_string(),
                                                qdrant_client::qdrant::Value {
                                                    kind: Some(
                                                        qdrant_client::qdrant::value::Kind::StringValue(
                                                            filename.clone(),
                                                        ),
                                                    ),
                                                },
                                            );
                                            
                                            payload_map.insert(
                                                "camera_id".to_string(),
                                                qdrant_client::qdrant::Value {
                                                    kind: Some(
                                                        qdrant_client::qdrant::value::Kind::StringValue(
                                                            image_data.cctv_id.clone(),
                                                        ),
                                                    ),
                                                },
                                            );
                                            
                                            payload_map.insert(
                                                "datetime".to_string(),
                                                qdrant_client::qdrant::Value {
                                                    kind: Some(
                                                        qdrant_client::qdrant::value::Kind::StringValue(
                                                            datetime_rfc3339,
                                                        ),
                                                    ),
                                                },
                                            );
                                            
                                            // Store vehicle type and AI label if available
                                            if let Some(ai_label) = &image_data.ai_label {
                                                payload_map.insert(
                                                    "vehicle_class".to_string(),
                                                    qdrant_client::qdrant::Value {
                                                        kind: Some(
                                                            qdrant_client::qdrant::value::Kind::StringValue(
                                                                ai_label.class_name.clone(),
                                                            ),
                                                        ),
                                                    },
                                                );
                                            }

                                            // Insert into Qdrant using the API's image ID
                                            let point_id: u64 = image_data.id as u64;

                                            let point = qdrant_client::qdrant::PointStruct::new(
                                                point_id,
                                                vector,
                                                payload_map,
                                            );

                                            let upsert = qdrant_client::qdrant::UpsertPoints {
                                                collection_name: collection_name.clone(),
                                                wait: Some(true),
                                                points: vec![point],
                                                ..Default::default()
                                            };

                                            match qdrant.upsert_points(upsert).await {
                                                Ok(_) => {
                                                    println!("      ✅ Inserted successfully")
                                                }
                                                Err(e) => {
                                                    println!(
                                                        "      ❌ Failed to insert: {}",
                                                        e
                                                    )
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            println!("      ⚠️  Failed to parse filename: {}", e)
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("      ❌ Failed to get embedding: {}", e)
                                }
                            }
                        }

                        println!("✅ Scheduled task completed\n");
                    }
                    Err(e) => {
                        println!("❌ Failed to fetch CCTV images: {}\n", e);
                    }
                }
            })
        })
        .expect("Failed to create scheduled job");

        sched.add(job).await.expect("Failed to add job");
        sched.start().await.expect("Failed to start scheduler");

        println!("✅ Background scheduler started (every 10 minutes)");

        // Keep the scheduler running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    });

    // Give scheduler time to initialize
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
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
