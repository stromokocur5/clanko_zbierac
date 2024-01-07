use std::collections::HashMap;

use crate::{trend::Trend, MediaConfig, Result};
use async_trait::async_trait;
use reqwest::{Client, Url};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};

#[async_trait]
pub trait Medium {
    async fn login(&mut self, client: &Client) -> Result<()>;
    async fn get_article(&self, client: &Client, url: &Url) -> Result<String>;
    async fn html_to_markdown(&self, content: &str) -> Result<(String, String)>;
    async fn logged(&self) -> bool;
}

#[derive(Clone)]
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
    pub async fn get_article<'a>(
        &self,
        url: &Url,
        media: &'a mut HashMap<String, Box<dyn Medium>>,
    ) -> Result<(String, String)> {
        let medium: &mut Box<dyn Medium> = self.which_medium(&url, media);
        if medium.logged().await == false {
            medium.login(&self.client).await?;
        }
        let article = medium.get_article(&self.client, &url).await?;
        let (article, title) = medium.html_to_markdown(&article).await?;
        Ok((article, title))
    }
    pub fn which_medium<'a>(
        &self,
        url: &Url,
        media: &'a mut HashMap<String, Box<dyn Medium>>,
    ) -> &'a mut Box<dyn Medium> {
        let domain = url.domain().unwrap_or_else(|| "");
        match domain {
            _ => media.get_mut("trend").unwrap(),
        }
    }
}
