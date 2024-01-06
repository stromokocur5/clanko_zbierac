use crate::medium::Medium;
use crate::MediaConfig;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};

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
        let trend = config.trend;
        Self {
            username: trend.username,
            password: trend.password,
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
        let document = Html::parse_document(content);

        let title = Selector::parse(r#"h1[data-don="article_title"]"#)
            .map_err(|_| anyhow!("no article_title"))?;

        let author = Selector::parse(r#"div[data-don="article_author"]"#)
            .map_err(|_| anyhow!("no article_author"))?;

        let perex = Selector::parse(r#"p[data-don="article_perex"]"#)
            .map_err(|_| anyhow!("no article_perex"))?;

        let day_month = Selector::parse(r#"span[class="datetime-day-month"]"#)
            .map_err(|_| anyhow!("no daytime"))?;

        let year =
            Selector::parse(r#"span[class="datetime-year"]"#).map_err(|_| anyhow!("no daytime"))?;

        let time =
            Selector::parse(r#"span[class="datetime-time"]"#).map_err(|_| anyhow!("no daytime"))?;

        let body = Selector::parse(r#"div[data-don="article_body"]"#)
            .map_err(|_| anyhow!("no article_body"))?;

        let selectors: Vec<(&Selector, &str)> = vec![
            (&title, "title"),
            (&author, "author"),
            (&day_month, "day_month"),
            (&year, "year"),
            (&time, "time"),
            (&perex, "perex"),
            (&body, "body"),
        ];
        let markdown = Self::handle_article(document.clone(), &selectors)?;

        let markdown = markdown.to_string();

        Ok(markdown)
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

        let document = Html::parse_document(&content);

        let csrf_selector = Selector::parse(r#"input[name="_csrf_token"]"#)
            .map_err(|_| anyhow!("nenasiel sa csrf token"))?;

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

    fn handle_article(document: Html, selectors: &Vec<(&Selector, &str)>) -> Result<String> {
        let mut day_month = String::new();
        let mut year = String::new();
        let mut markdown = String::new();
        for (s, m) in selectors {
            let part = document.select(s).next();
            if let None = part {
                continue;
            }
            let orig_part = part.unwrap();
            let part = orig_part.inner_html();
            let part = part.trim();
            let part = match m.to_owned() {
                "title" => format!("---\ntitle: {part}\n"),
                "author" => format!("author: {part}\n"),
                "day_month" => {
                    day_month = part.into();
                    "".to_string()
                }
                "year" => {
                    year = part.into();
                    "".to_string()
                }
                "time" => {
                    let time = part;
                    format!("date: {day_month}{year} {time}\n---\n")
                }
                "perex" => {
                    let text = orig_part.text();
                    let text = text.last().ok_or_else(|| anyhow!(""))?.trim();
                    format!("\n# {}\n", text)
                }
                "body" => {
                    let mut text = String::new();
                    for i in orig_part.children() {
                        let value = i.value();

                        let name = value.as_element();
                        if !value.is_element() {
                            continue;
                        }
                        let name = name.unwrap().name();
                        if !i.has_children() {
                            continue;
                        }
                        for j in i.children() {
                            let value = j.value();

                            if value.is_element() {
                                let name = value.as_element().unwrap();
                                let name = name.name();
                                if name == "em" || name == "p" || name == "a" {
                                    for k in j.children() {
                                        let value = k.value();
                                        if !value.is_text() {
                                            continue;
                                        }
                                        text.push_str(&format!(
                                            " {}",
                                            &value.as_text().unwrap().text
                                        ));
                                    }
                                }
                            }
                            if !value.is_text() {
                                continue;
                            }
                            if name == "p" {
                                text.push_str(&format!(
                                    "{}",
                                    &value.as_text().unwrap().text.trim()
                                ));
                            }
                            if name == "h2" {
                                text.push_str(&format!(
                                    "\n\n## {}\n",
                                    &value.as_text().unwrap().text.trim()
                                ));
                            }
                        }
                    }

                    format!("{}", text)
                }
                _ => format!(""),
            };
            markdown.push_str(&part);
        }
        Ok(markdown)
    }
}
