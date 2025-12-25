//! Background Scheduler
//!
//! Handles scheduled tasks for fetching and processing CCTV images.

use crate::config::Config;
use crate::models::search::CctvImageData;
use crate::services::{fetch_cctv_training_data, get_image_embedding, api_datetime_to_rfc3339, PayloadBuilder};
use chrono::Duration;
use chrono_tz::Asia::Bangkok;
use qdrant_client::qdrant::{PointStruct, UpsertPoints};
use qdrant_client::Qdrant;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

/// Scheduler context containing shared resources
#[derive(Clone)]
pub struct SchedulerContext {
    pub qdrant: Arc<Qdrant>,
    pub http_client: reqwest::Client,
    pub config: Config,
}

impl SchedulerContext {
    pub fn new(qdrant: Arc<Qdrant>, http_client: reqwest::Client, config: Config) -> Self {
        Self {
            qdrant,
            http_client,
            config,
        }
    }
}

/// Start the background scheduler for CCTV image fetching
pub async fn start_scheduler(ctx: SchedulerContext) {
    tokio::spawn(async move {
        let sched = JobScheduler::new()
            .await
            .expect("Failed to create scheduler");

        // Build cron expression dynamically based on FETCH_EVERY_TIME
        let cron_expr = format!("0 */{} * * * *", ctx.config.fetch_every_time);
        
        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _l| {
            let ctx = ctx.clone();
            Box::pin(async move {
                run_fetch_task(&ctx).await;
            })
        })
        .expect("Failed to create scheduled job");

        sched.add(job).await.expect("Failed to add job");
        sched.start().await.expect("Failed to start scheduler");

        println!("✅ Background scheduler started (every {} minutes)", ctx.config.fetch_every_time);

        // Keep scheduler running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    });
}

/// Run the CCTV image fetch and processing task
async fn run_fetch_task(ctx: &SchedulerContext) {
    println!("\n⏰ Running scheduled CCTV image fetch...");

    // Calculate time range in Thailand timezone
    let now = chrono::Utc::now().with_timezone(&Bangkok);
    let date_stop = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let date_start = (now - Duration::days(ctx.config.fetch_days_range))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    // Fetch images from CCTV API
    match fetch_cctv_training_data(
        &ctx.config.cctv_api_url,
        &ctx.config.cctv_auth_token,
        &ctx.config.cctv_id,
        &date_start,
        &date_stop,
        ctx.config.fetch_limit,
    )
    .await
    {
        Ok(images) => {
            println!("📥 Processing {} images...", images.len());
            process_images(ctx, &images).await;
            println!("✅ Scheduled task completed\n");
        }
        Err(e) => {
            println!("❌ Failed to fetch CCTV images: {}\n", e);
        }
    }
}

/// Process a batch of images using batch embedding
async fn process_images(ctx: &SchedulerContext, images: &[CctvImageData]) {
    if images.is_empty() {
        return;
    }

    println!("   🚀 Getting batch embeddings for {} images...", images.len());

    // Collect all image paths
    let image_paths: Vec<String> = images.iter()
        .map(|img| img.file_path.clone())
        .collect();

    // Get batch embeddings
    let batch_result = match get_image_embedding(
        &ctx.http_client,
        &ctx.config.ai_service_url,
        image_paths.clone()
    ).await {
        Ok(result) => result,
        Err(e) => {
            println!("   ❌ Failed to get batch embeddings: {}", e);
            return;
        }
    };

    println!("   ✅ Received {} embedding results", batch_result.results.len());

    // Process each result and store in Qdrant
    for (idx, result) in batch_result.results.iter().enumerate() {
        // Find the corresponding image data
        let image = match images.iter().find(|img| img.file_path == result.path) {
            Some(img) => img,
            None => {
                println!("   ⚠️  Could not find image data for path: {}", result.path);
                continue;
            }
        };

        println!(
            "   [{}/{}] Processing: {}",
            idx + 1,
            batch_result.results.len(),
            image.filename
        );

        // Check if this result has an error
        if let Some(ref error) = result.error {
            println!("      ❌ {}", error);
            continue;
        }

        // Check if embedding is present
        let vector = match &result.embedding {
            Some(v) => v.clone(),
            None => {
                println!("      ❌ No embedding in result");
                continue;
            }
        };

        // Build payload and store in Qdrant
        if let Err(e) = store_image_in_qdrant(ctx, image, vector).await {
            println!("      ❌ {}", e);
        } else {
            println!("      ✅ Inserted successfully");
        }
    }
}

/// Store a single image with its embedding in Qdrant
async fn store_image_in_qdrant(
    ctx: &SchedulerContext,
    image: &CctvImageData,
    vector: Vec<f32>
) -> Result<(), String> {
    // Build payload using the builder
    let datetime_rfc3339 = api_datetime_to_rfc3339(&image.date, &image.time);
    
    // Use provided created_at or generate current timestamp
    let created_at = image.created_at.clone().unwrap_or_else(|| {
        chrono::Utc::now().to_rfc3339()
    });

    let mut payload_builder = PayloadBuilder::new()
        .string("image", &image.file_path)
        .string("filename", &image.filename)
        .string("camera_id", &image.cctv_id)
        .string("datetime", datetime_rfc3339)
        .integer("frame", image.frame as i64)
        .integer("vehicle_type", image.vehicle_type as i64)
        .integer("yolo_id", image.yolo_id as i64)
        .string("created_at", &created_at);

    // Add AI label if present
    if let Some(ai_label) = &image.ai_label {
        payload_builder = payload_builder
            .string("vehicle_class", &ai_label.class_name)
            .double("confidence", ai_label.confidence as f64);
    }

    let payload_map = payload_builder.build();

    // Create and upsert point
    let point = PointStruct::new(image.id as u64, vector, payload_map);

    let upsert = UpsertPoints {
        collection_name: ctx.config.collection_name.clone(),
        wait: Some(true),
        points: vec![point],
        ..Default::default()
    };

    ctx.qdrant
        .upsert_points(upsert)
        .await
        .map_err(|e| format!("Failed to insert: {}", e))?;

    Ok(())
}
