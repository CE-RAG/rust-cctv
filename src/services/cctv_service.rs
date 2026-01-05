use std::io::Error;

use crate::clients::cctv_client::CctvApiClient;
use crate::models::cctv::CctvListResponse;
use crate::models::search::{CctvImageData, CctvMetadataRequest, CctvMetadataResponse};

pub struct CctvService<T: CctvApiClient> {
    client: T,
}

impl<T: CctvApiClient + Clone> Clone for CctvService<T> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}

impl<T: CctvApiClient> CctvService<T> {
    pub fn new(client: T) -> Self {
        Self { client }
    }

    pub async fn list_cctv(&self) -> Result<Vec<String>, Error> {
        let url = format!("{}/video-metadata/list-cctv", self.client.base_url());

        let auth_header = self
            .client
            .auth_header()
            .await
            .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

        let response = self
            .client
            .client()
            .get(url)
            .header("Authorization", auth_header)
            .send()
            .await
            .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

        let resp = response
            .json::<CctvListResponse>()
            .await
            .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

        Ok(resp.data.into_iter().map(|c| c.cctv_id).collect())
    }

    pub async fn fetch_train_data(
        &self,
        request_body: &CctvMetadataRequest,
    ) -> Result<Vec<CctvImageData>, Error> {
        let url = format!(
            "{}/video-metadata/train-data-condition",
            self.client.base_url()
        );

        let auth_header = self
            .client
            .auth_header()
            .await
            .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

        let response = self
            .client
            .client()
            .post(url)
            .header("Authorization", auth_header)
            .json(request_body)
            .send()
            .await
            .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

        let response_data = response
            .json::<CctvMetadataResponse>()
            .await
            .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

        if !response_data.success {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "API returned success=false",
            ));
        }

        Ok(response_data.data)
    }
}
