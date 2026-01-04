use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
pub struct GetTokenRequest {
    pub authorize_code: String,
    pub user_auth: String,
    pub client_id: String,
    pub scope: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TokenData {
    pub token_type: String,
    pub access_token: String,
    pub status: bool,
}

#[derive(Debug, Deserialize)]
pub struct GetTokenResponse {
    pub Code: u32,
    pub Message: String,
    pub Data: TokenData,
}
