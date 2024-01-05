use crate::{MediaConfig, Result};
use async_trait::async_trait;
use reqwest::{Client, Url};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};

#[async_trait]
pub trait Medium: From<MediaConfig> {
    async fn login(&self, client: &Client) -> Result<()>;
    async fn get_article(client: &Client, url: Url) -> Result<String>;
    async fn html_to_markdown(content: &str) -> Result<String>;
}

pub struct MediumClient(Client);

impl MediumClient {
    pub async fn new() -> Self {
        let cookie_store = CookieStore::new(None);
        let cookie_store = CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);
        let client = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .cookie_provider(std::sync::Arc::clone(&cookie_store))
            .build()
            .unwrap();
        MediumClient(client)
    }
    pub async fn get_article<T: Medium>(&self, medium: T, url: Url) -> Result<String> {
        medium.login(&self.0).await?;
        let article = T::get_article(&self.0, url).await?;
        let article = T::html_to_markdown(&article).await?;
        Ok(article)
    }
}
