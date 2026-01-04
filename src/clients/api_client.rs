use reqwest::Client;

pub trait ApiClient {
    fn base_url(&self) -> &str;

    fn client(&self) -> &Client;
}
