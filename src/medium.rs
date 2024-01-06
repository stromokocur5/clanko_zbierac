use crate::{trend::Trend, MediaConfig, Result};
use async_trait::async_trait;
use reqwest::{Client, Url};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};

#[async_trait]
pub trait Medium {
    async fn login(&self, client: &Client) -> Result<()>;
    async fn get_article(&self, client: &Client, url: &Url) -> Result<String>;
    async fn html_to_markdown(&self, content: &str) -> Result<(String, String)>;
}

pub struct MediumClient {
    client: Client,
    config: MediaConfig,
}

impl MediumClient {
    pub async fn new(config: MediaConfig) -> Self {
        let cookie_store = CookieStore::new(None);
        let cookie_store = CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);
        let client = reqwest::ClientBuilder::new()
            .cookie_store(true)
            .cookie_provider(std::sync::Arc::clone(&cookie_store))
            .build()
            .unwrap();
        MediumClient { client, config }
    }
    pub async fn get_article(&self, url: &Url) -> Result<(String, String)> {
        let medium = self.which_medium(&url).await;
        medium.login(&self.client).await?;
        let article = medium.get_article(&self.client, &url).await?;
        let (article, title) = medium.html_to_markdown(&article).await?;
        Ok((article, title))
    }
    pub async fn which_medium(&self, url: &Url) -> Box<dyn Medium> {
        let domain = url.domain().unwrap_or_else(|| "");
        match domain {
            _ => Box::new(Trend::from(self.config.clone())),
        }
    }
}
