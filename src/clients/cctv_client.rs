use std::io::Error;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;

use super::api_client::ApiClient;
use super::base_client::BaseApiClient;
use crate::models::token::GetTokenRequest;
use reqwest::Client;

pub trait CctvApiClient: ApiClient {
    async fn auth_header(&self) -> Result<String, Error>;
}

#[derive(Clone)]
pub struct CctvApi {
    pub base_url: String,
    pub client: Client,
    base_client: BaseApiClient,
    token: Arc<Mutex<Option<(String, SystemTime)>>>,
    token_request: GetTokenRequest,
}

impl CctvApi {
    pub fn new(
        base_url: impl Into<String>,
        authorize_code: impl Into<String>,
        user_auth: impl Into<String>,
        client_id: impl Into<String>,
    ) -> Self {
        let base_url_str = base_url.into();
        let base_client = BaseApiClient::new(base_url_str.clone());

        let token_request = GetTokenRequest {
            authorize_code: authorize_code.into(),
            user_auth: user_auth.into(),
            client_id: client_id.into(),
            scope: vec!["client".to_string()], // Default scope
        };

        Self {
            base_url: base_url_str,
            client: Client::new(),
            base_client,
            token: Arc::new(Mutex::new(None)),
            token_request,
        }
    }

    async fn get_or_refresh_token(&self) -> Result<String, Error> {
        let mut token_guard = self.token.lock().await;

        // Check if we have a valid token that hasn't expired
        if let Some((ref token, expiry)) = *token_guard {
            if SystemTime::now() < expiry {
                return Ok(token.clone());
            }
        }

        // Token is expired or doesn't exist, fetch a new one
        let new_token = self
            .base_client
            .get_token(&self.token_request)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Set expiry to 2 hours from now (with 5 min buffer)
        let expiry = SystemTime::now() + Duration::from_secs(2 * 60 * 60 - (5 * 60));
        *token_guard = Some((new_token.clone(), expiry));

        Ok(new_token)
    }
}

impl ApiClient for CctvApi {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn client(&self) -> &Client {
        &self.client
    }
}

impl CctvApiClient for CctvApi {
    async fn auth_header(&self) -> Result<String, Error> {
        let token = self.get_or_refresh_token().await?;
        Ok(format!("Bearer {}", token))
    }
}
