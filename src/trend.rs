use crate::medium::Medium;
use crate::MediaConfig;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;

#[derive(serde::Serialize)]
struct User {
    _csrf_token: String,
    _username: String,
    _password: String,
}

#[derive(serde::Deserialize)]
pub struct Trend {
    pub username: String,
    pub password: String,
}

impl From<MediaConfig> for Trend {
    fn from(config: MediaConfig) -> Self {
        Self {
            username: config.trend.username,
            password: config.trend.password,
        }
    }
}
#[async_trait]
impl Medium for Trend {
    async fn get_article(client: &Client, url: reqwest::Url) -> Result<String> {
        let res = client.get(url).send().await?;
        let res = res.text().await?;
        Ok(res)
    }

    async fn login(&self, client: &Client) -> Result<()> {
        let csrf_token = Self::get_csrf_token(&client).await?;
        let user = User {
            _csrf_token: csrf_token,
            _username: self.username.to_owned(),
            _password: self.password.to_owned(),
        };
        let _ = client
            .post("https://sso.newsandmedia.sk/login")
            .form(&user)
            .send()
            .await?;
        Ok(())
    }

    async fn html_to_markdown(content: &str) -> Result<String> {
        use scraper::{Html, Selector};
        let document = Html::parse_document(content);
        let mut markdown = String::new();

        let article_title = Selector::parse(r#"h1[data-don="article_title"]"#)
            .map_err(|_| anyhow!("no article_title"))?;

        let article_author = Selector::parse(r#"div[data-don="article_author"]"#)
            .map_err(|_| anyhow!("no article_author"))?;

        let article_perex = Selector::parse(r#"p[data-don="article_perex"]"#)
            .map_err(|_| anyhow!("no article_perex"))?;

        let datetime =
            Selector::parse(r#"span[class="datetime"]"#).map_err(|_| anyhow!("no daytime"))?;

        let article_body = Selector::parse(r#"div[data-don="article_body"]"#)
            .map_err(|_| anyhow!("no article_body"))?;

        Ok(document
            .select(&article_title)
            .next()
            .unwrap()
            .inner_html()
            .trim_start()
            .trim_end()
            .to_string())
    }
}
impl Trend {
    async fn get_csrf_token(client: &Client) -> Result<String> {
        let content = client
            .get("https://sso.newsandmedia.sk/login")
            .send()
            .await?
            .text()
            .await?;

        let document = scraper::Html::parse_document(&content);

        let csrf_selector = scraper::Selector::parse(r#"input[name="_csrf_token"]"#).unwrap();

        let csrf_element = document
            .select(&csrf_selector)
            .next()
            .expect("csrf token sa nepodarilo ziskat");
        let csrf_token = csrf_element
            .value()
            .attr("value")
            .expect("nepodarilo sa ziskat hodnotu csrf tokenu");
        Ok(csrf_token.to_string())
    }
}
