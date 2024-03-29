use crate::trend::Trend;
use crate::MediaConfig;
use crate::Result;
use async_trait::async_trait;
use reqwest::{Client, Url};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use std::collections::HashMap;

#[async_trait]
pub trait Medium {
    async fn login(&mut self, client: &Client) -> Result<()>;
    async fn get_article(&self, client: &Client, url: &Url) -> Result<String>;
    async fn html_to_markdown(&self, content: &str) -> Result<(String, String)>;
    async fn logged(&self) -> bool;
}
pub type MediaStore = HashMap<String, Box<dyn Medium + Send + Sync>>;

pub struct MediumClient {
    client: Client,
    config: MediaConfig,
    media: MediaStore,
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
        let mut media: MediaStore = HashMap::new();
        media.insert("trend".to_owned(), Box::new(Trend::from(config.clone())));
        MediumClient {
            client,
            config,
            media,
        }
    }
    pub async fn get_article(&mut self, url: &Url) -> Result<(String, String)> {
        let medium = Self::which_medium(&url);
        let medium = self.media.get_mut(medium).unwrap();
        if !medium.logged().await {
            medium.login(&self.client).await?;
        }
        let article = medium.get_article(&self.client, &url).await?;
        let (article, title) = medium.html_to_markdown(&article).await?;
        Ok((article, title))
    }
    pub fn which_medium(url: &Url) -> &str {
        let domain = url.domain().unwrap_or_else(|| "");
        match domain {
            _ => "trend",
        }
    }
}
