use crate::medium::Medium;
use crate::MediaConfig;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ego_tree::NodeRef;
use reqwest::Client;
use scraper::{Html, Selector};

#[derive(serde::Serialize)]
struct User {
    _csrf_token: String,
    _username: String,
    _password: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Trend {
    pub username: String,
    pub password: String,
    #[serde(default = "logged_default")]
    pub logged: bool,
}
fn logged_default() -> bool {
    false
}

impl From<MediaConfig> for Trend {
    fn from(config: MediaConfig) -> Self {
        config.trend.unwrap_or_default()
    }
}
impl Default for Trend {
    fn default() -> Self {
        Self {
            username: "".to_string(),
            password: "".to_string(),
            logged: false,
        }
    }
}
#[async_trait]
impl Medium for Trend {
    async fn get_article(&self, client: &Client, url: &reqwest::Url) -> Result<String> {
        let res = client.get(url.to_owned()).send().await?;
        let res = res.text().await?;
        Ok(res)
    }

    async fn login(&mut self, client: &Client) -> Result<()> {
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
        self.logged = true;
        Ok(())
    }

    async fn html_to_markdown(&self, content: &str) -> Result<(String, String)> {
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
        let (markdown, title) = Self::handle_article(document.clone(), &selectors)?;

        Ok((markdown, title))
    }
    async fn logged(&self) -> bool {
        self.logged.clone()
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

    fn handle_article(
        document: Html,
        selectors: &Vec<(&Selector, &str)>,
    ) -> Result<(String, String)> {
        let mut title = String::new();
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
                "title" => {
                    title = part.to_lowercase().replace(" ", "_");
                    format!("---\ntitle: {part}\n")
                }
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
                        handle_node(i, "", &mut text);
                    }
                    text
                }
                _ => format!(""),
            };
            markdown.push_str(&part);
        }
        Ok((markdown, title))
    }
}

fn handle_node(node: NodeRef<'_, scraper::Node>, e_name: &str, mut_text: &mut String) {
    for i in node.children() {
        let value = i.value();

        if value.is_element() {
            let mut bad_class = false;
            let el = value.as_element().unwrap();
            let classes = el.classes();
            for i in classes {
                if i == "article-related" || i == "unlock-subscription" || i == "attribution" {
                    bad_class = true;
                    continue;
                }
            }
            if bad_class {
                continue;
            }
            let name = el.name();
            handle_node(i, name, mut_text);
        }

        if value.is_text() {
            let txt = value.as_text().unwrap().text.clone();
            let txt = match e_name {
                "h2" | "strong" | "dt" => format!("\n\n## {}\n", txt.trim()),
                "itm-params" | "figcaption" => "".to_owned(),
                "dd" => format!("\n{}\n", txt.trim()),
                _ => format!("{}", txt.trim()),
            };
            mut_text.push_str(&txt);
        }
    }
}
