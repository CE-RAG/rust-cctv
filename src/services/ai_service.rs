//! AI Embedding Service
//! 
//! Functions to get text and image embeddings from the AI service.

use crate::models::search::{EmbedResponse, BatchImageEmbeddingResponse};

/// Get text embedding from AI service
pub async fn get_text_embedding(
    client: &reqwest::Client,
    base_url: &str,
    text: &str,
) -> Result<Vec<f32>, String> {
    let url = format!("{}/predict", base_url);

    let res = client
        .post(&url)
        .json(&serde_json::json!({ "text": text }))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to AI Service: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("AI Service returned error: {}", res.status()));
    }

    let data: EmbedResponse = res.json().await.map_err(|e| {
        format!(
            "Failed to parse AI response. Ensure Python returns 'vector' or 'embedding' key. Error: {}",
            e
        )
    })?;

    Ok(data.vector)
}

/// Get image embedding(s) from AI service
/// 
/// Supports both single and batch image embedding requests.
/// Pass a single image path or multiple image paths in the vector.
/// 
/// # Examples
/// 
/// Single image:
/// ```
/// let result = get_image_embedding(&client, &url, vec!["image.jpg".to_string()]).await?;
/// ```
/// 
/// Batch images:
/// ```
/// let result = get_image_embedding(&client, &url, vec!["img1.jpg".to_string(), "img2.jpg".to_string()]).await?;
/// ```
pub async fn get_image_embedding(
    client: &reqwest::Client,
    base_url: &str,
    image_paths: Vec<String>,
) -> Result<BatchImageEmbeddingResponse, String> {
    if image_paths.is_empty() {
        return Err("No image paths provided".to_string());
    }

    let url = format!("{}/predict", base_url);

    let res = client
        .post(&url)
        .json(&serde_json::json!({ "image_paths": image_paths }))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to AI Image Service: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("AI Image Service returned error: {}", res.status()));
    }

    let data: BatchImageEmbeddingResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse AI image response: {}", e))?;

    Ok(data)
}
