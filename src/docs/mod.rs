use crate::models::search::{CctvImageData, SearchRequest, SearchResult};
use utoipa::OpenApi;

// Re-export SwaggerUi for use in main.rs
pub use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::search_vehicles,
        crate::handlers::insert_image,
    ),
    components(
        schemas(
            SearchRequest,
            SearchResult,
            CctvImageData,
        )
    ),
    tags(
        (name = "Search API", description = "Vehicle search endpoints"),
        (name = "Insertion API", description = "Image insertion endpoints")
    )
)]
pub struct ApiDoc;
