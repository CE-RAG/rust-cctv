use super::api_client::ApiClient;
use crate::models::token::{GetTokenRequest, GetTokenResponse};
use reqwest::{Client, Error};

#[derive(Clone)]
pub struct BaseApiClient {
    pub base_url: String,
    pub client: Client,
}

impl BaseApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    pub async fn get_token(&self, request_body: &GetTokenRequest) -> Result<String, Error> {
        let url = format!("{}/get-token", self.base_url);

        let resp = self
            .client
            .post(url)
            .json(request_body)
            .send()
            .await?
            .json::<GetTokenResponse>()
            .await?;

        Ok(resp.Data.access_token)
    }
}

impl ApiClient for BaseApiClient {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn client(&self) -> &Client {
        &self.client
    }
}
