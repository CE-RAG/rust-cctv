use crate::models::search::EmbedResponse;

// --- Helper: Call Python AI Service ---
pub async fn get_text_embedding(
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
        format!(
            "Failed to parse AI response. Ensure Python returns 'vector' or 'embedding' key. Error: {}",
            e
        )
    })?;

    Ok(data.vector)
}

// --- Helper: Call Python AI Service for IMAGE embedding ---
pub async fn get_image_embedding(
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
