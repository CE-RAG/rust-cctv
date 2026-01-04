use std::io::Error;
use std::sync::Arc;
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
    token: Arc<Mutex<Option<String>>>,
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
            scope: vec!["read".to_string()], // Default scope
        };

        Self {
            base_url: base_url_str,
            client: Client::new(),
            base_client,
            token: Arc::new(Mutex::new(None)),
            token_request,
        }
    }

    // Get or refresh the auth token
    async fn get_or_refresh_token(&self) -> Result<String, Error> {
        let mut token_guard = self.token.lock().await;

        // If we already have a token, return it
        if let Some(ref token) = *token_guard {
            return Ok(token.clone());
        }

        // Otherwise fetch a new token
        let new_token = self
            .base_client
            .get_token(&self.token_request)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        *token_guard = Some(new_token.clone());

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
