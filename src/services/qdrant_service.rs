//! Qdrant Service
//! 
//! Functions for interacting with Qdrant vector database.

use qdrant_client::qdrant::{CreateCollection, CreateFieldIndexCollectionBuilder, Distance, FieldType, VectorParams};
use qdrant_client::Qdrant;

/// Ensure collection exists, create if not
pub async fn ensure_collection_exists(
    qdrant: &Qdrant,
    collection_name: &str,
    vector_size: usize,
) -> Result<(), String> {
    let vector_params = VectorParams {
        size: vector_size as u64,
        distance: Distance::Cosine.into(),
        ..Default::default()
    };

    let create_collection = CreateCollection {
        collection_name: collection_name.to_string(),
        vectors_config: Some(vector_params.into()),
        ..Default::default()
    };

    match qdrant.create_collection(create_collection).await {
        Ok(_) => println!("✅ Collection '{}' created successfully", collection_name),
        Err(e) => {
            let error_msg = format!("{}", e);
            if error_msg.contains("already exists") {
                println!("✅ Collection '{}' already exists", collection_name);
            } else {
                return Err(format!("Failed to create collection: {}", e));
            }
        }
    }

    Ok(())
}

/// Create datetime field index for filtering
pub async fn create_datetime_index(
    qdrant: &Qdrant,
    collection_name: &str,
) -> Result<(), String> {
    // Check if collection exists
    let collections = qdrant
        .list_collections()
        .await
        .map_err(|e| format!("Failed to list collections: {}", e))?;

    let collection_exists = collections
        .collections
        .iter()
        .any(|c| c.name == collection_name);

    if !collection_exists {
        return Err(format!("Collection '{}' does not exist", collection_name));
    }

    // Create datetime field index
    qdrant
        .create_field_index(
            CreateFieldIndexCollectionBuilder::new(collection_name, "datetime", FieldType::Datetime)
                .wait(true),
        )
        .await
        .map_err(|e| format!("Failed to create datetime index: {}", e))?;

    Ok(())
}
