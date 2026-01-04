use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CctvListResponse {
    pub success: bool,
    pub data: Vec<CctvItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CctvItem {
    pub cctv_id: String,
}
